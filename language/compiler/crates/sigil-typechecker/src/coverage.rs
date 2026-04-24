//! Match exhaustiveness and coverage analysis for Sigil.
//!
//! Defines the MatchSpace type system and all coverage checking functions.
//! Invoked by synthesize_match and check_match in bidirectional.rs.

use crate::bidirectional::{
    constructor_display_name, create_constructor_type_with_result_name, lookup_constructor_type,
    match_arm_refinement, scrutinee_proof_context,
    scrutinee_symbolic_value, sorted_record_field_types, split_qualified_constructor_name,
};
use crate::environment::{TypeEnvironment, TypeInfo};
use crate::errors::{format_type, TypeError};
use crate::proof_context::{
    solve_exact_single_var, solve_single_var_interval, ProofContext, SymbolicCollection,
    SymbolicRecord, SymbolicValue, MATCH_SCRUTINEE_BINDING,
};
use crate::types::{apply_subst, unify, InferenceType, TConstructor, TPrimitive};
use sigil_ast::{Expr, LiteralValue, PrimitiveName, TypeDef};
use sigil_diagnostics::codes;
use sigil_solver::{
    prove_formula, Atom, ComparisonOp, Formula, SolverOutcome, SymbolPath,
    SymbolPathStep,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum LiteralAtom {
    Float(u64),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PrimitiveEqKind {
    Float,
    String,
    Char,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PrimitiveSpace {
    Bool {
        allow_true: bool,
        allow_false: bool,
    },
    Unit {
        present: bool,
    },
    Int(IntRangeSet),
    EqAny {
        kind: PrimitiveEqKind,
        excluded: std::collections::BTreeSet<LiteralAtom>,
    },
    EqFinite {
        kind: PrimitiveEqKind,
        values: std::collections::BTreeSet<LiteralAtom>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VariantSpace {
    owner: String,
    name: String,
    fields: Vec<MatchSpace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordSpace {
    fields: Vec<(String, MatchSpace)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ListSpace {
    Any(Box<MatchSpace>),
    Nil,
    Cons {
        head: Box<MatchSpace>,
        tail: Box<MatchSpace>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MatchSpace {
    Empty,
    Primitive(PrimitiveSpace),
    Variant(VariantSpace),
    Record(RecordSpace),
    Tuple(Vec<MatchSpace>),
    List(ListSpace),
    Union(Vec<MatchSpace>),
    Opaque(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntInterval {
    start: Option<i64>,
    end: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntRangeSet {
    intervals: Vec<IntInterval>,
}

impl IntRangeSet {
    fn all() -> Self {
        Self {
            intervals: vec![IntInterval {
                start: None,
                end: None,
            }],
        }
    }

    fn singleton(value: i64) -> Self {
        Self {
            intervals: vec![IntInterval {
                start: Some(value),
                end: Some(value),
            }],
        }
    }

    fn empty() -> Self {
        Self { intervals: vec![] }
    }

    fn greater_eq(value: i64) -> Self {
        Self {
            intervals: vec![IntInterval {
                start: Some(value),
                end: None,
            }],
        }
    }

    fn less_eq(value: i64) -> Self {
        Self {
            intervals: vec![IntInterval {
                start: None,
                end: Some(value),
            }],
        }
    }

    fn union(&self, other: &Self) -> Self {
        let mut intervals = self.intervals.clone();
        intervals.extend(other.intervals.clone());
        normalize_int_ranges(intervals)
    }

    fn intersect(&self, other: &Self) -> Self {
        let mut result = Vec::new();
        for left in &self.intervals {
            for right in &other.intervals {
                if let Some(interval) = intersect_interval(left, right) {
                    result.push(interval);
                }
            }
        }
        normalize_int_ranges(result)
    }

    fn difference(&self, other: &Self) -> Self {
        let mut current = self.clone();
        for interval in &other.intervals {
            current = current.subtract_interval(interval);
            if current.is_empty() {
                break;
            }
        }
        current
    }

    fn subtract_interval(&self, remove: &IntInterval) -> Self {
        let mut result = Vec::new();
        for interval in &self.intervals {
            result.extend(subtract_interval(interval, remove));
        }
        normalize_int_ranges(result)
    }

    fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }
}

fn normalize_int_ranges(mut intervals: Vec<IntInterval>) -> IntRangeSet {
    intervals.retain(|interval| interval_valid(interval));
    intervals.sort_by(compare_interval_start);

    let mut merged: Vec<IntInterval> = Vec::new();
    for interval in intervals {
        if let Some(last) = merged.last_mut() {
            if intervals_touch_or_overlap(last, &interval) {
                last.end = max_end(last.end, interval.end);
                continue;
            }
        }
        merged.push(interval);
    }

    IntRangeSet { intervals: merged }
}

fn compare_interval_start(left: &IntInterval, right: &IntInterval) -> std::cmp::Ordering {
    match (left.start, right.start) {
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (Some(_), None) => std::cmp::Ordering::Greater,
        (Some(left), Some(right)) => left.cmp(&right),
    }
}

fn interval_valid(interval: &IntInterval) -> bool {
    match (interval.start, interval.end) {
        (Some(start), Some(end)) => start <= end,
        _ => true,
    }
}

fn intersect_interval(left: &IntInterval, right: &IntInterval) -> Option<IntInterval> {
    let start = max_start(left.start, right.start);
    let end = min_end(left.end, right.end);
    let interval = IntInterval { start, end };
    interval_valid(&interval).then_some(interval)
}

fn subtract_interval(base: &IntInterval, remove: &IntInterval) -> Vec<IntInterval> {
    let Some(overlap) = intersect_interval(base, remove) else {
        return vec![base.clone()];
    };

    let mut result = Vec::new();

    if overlap.start != base.start {
        let left_end = overlap.start.and_then(|start| start.checked_sub(1));
        let left = IntInterval {
            start: base.start,
            end: left_end,
        };
        if interval_valid(&left) {
            result.push(left);
        }
    }

    if overlap.end != base.end {
        let right_start = overlap.end.and_then(|end| end.checked_add(1));
        let right = IntInterval {
            start: right_start,
            end: base.end,
        };
        if interval_valid(&right) {
            result.push(right);
        }
    }

    result
}

fn max_start(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn min_end(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn max_end(left: Option<i64>, right: Option<i64>) -> Option<i64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (None, _) | (_, None) => None,
    }
}

fn intervals_touch_or_overlap(left: &IntInterval, right: &IntInterval) -> bool {
    match (left.end, right.start) {
        (Some(left_end), Some(right_start)) => left_end
            .checked_add(1)
            .is_some_and(|next| next >= right_start),
        (None, _) | (_, None) => true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ValuePathStep {
    Field(String),
    VariantField(usize),
    TupleIndex(usize),
    ListHead,
    ListTail,
}

type ValuePath = Vec<ValuePathStep>;

#[derive(Debug, Clone)]
struct ArmProof {
    space: MatchSpace,
    guard_supported: bool,
    facts: Vec<String>,
    unsupported_facts: Vec<String>,
    condition_formula: Option<Formula>,
}

pub(crate) fn analyze_match_coverage(
    env: &TypeEnvironment,
    proof_context: &ProofContext,
    scrutinee_type: &InferenceType,
    match_expr: &sigil_ast::MatchExpr,
) -> Result<(), TypeError> {
    let scrutinee_space = total_space_for_type(env, scrutinee_type)?;
    let mut remaining = scrutinee_space.clone();
    let mut previous_facts: Vec<serde_json::Value> = Vec::new();
    let mut unsupported_facts = Vec::new();
    let mut fallthrough_context =
        scrutinee_proof_context(env, proof_context, &match_expr.scrutinee);

    for (index, arm) in match_expr.arms.iter().enumerate() {
        let arm_proof = arm_proof(
            env,
            &fallthrough_context,
            &match_expr.scrutinee,
            scrutinee_type,
            arm,
        )?;
        if !arm_proof.unsupported_facts.is_empty() {
            unsupported_facts.extend(arm_proof.unsupported_facts.clone());
        }

        if space_is_empty(&remaining) {
            return Err(
                TypeError::new("Unreachable match arm".to_string(), Some(arm.location))
                    .with_code(codes::typecheck::MATCH_UNREACHABLE_ARM)
                    .with_detail("armIndex", index)
                    .with_detail(
                        "scrutineeType",
                        format_type(&env.normalize_type(scrutinee_type)),
                    )
                    .with_detail("coveredByArm", index.saturating_sub(1))
                    .with_detail("coveredBy", previous_facts.clone())
                    .with_detail("proofFragment", proof_fragment())
                    .with_detail("unsupportedFacts", unsupported_facts.clone()),
            );
        }

        let useful_space = space_intersection(&remaining, &arm_proof.space);
        if space_is_empty(&useful_space) {
            return Err(TypeError::new(
                "Redundant pattern in match expression".to_string(),
                Some(arm.location),
            )
            .with_code(codes::typecheck::MATCH_REDUNDANT_PATTERN)
            .with_detail("armIndex", index)
            .with_detail(
                "scrutineeType",
                format_type(&env.normalize_type(scrutinee_type)),
            )
            .with_detail("coveredBy", previous_facts.clone())
            .with_detail("knownFacts", arm_proof.facts.clone())
            .with_detail("remainingBeforeArm", space_to_case_summaries(&remaining, 8))
            .with_detail("armCases", space_to_case_summaries(&arm_proof.space, 8))
            .with_detail("proofFragment", proof_fragment())
            .with_detail("unsupportedFacts", unsupported_facts.clone()));
        }

        previous_facts.push(serde_json::json!({
            "armIndex": index,
            "facts": arm_proof.facts,
            "guardSupported": arm_proof.guard_supported,
            "pattern": pattern_summary(&arm.pattern),
        }));

        if arm_proof.guard_supported {
            remaining = space_difference(&remaining, &arm_proof.space);
        }
        if let Some(condition_formula) = arm_proof.condition_formula {
            fallthrough_context =
                fallthrough_context.with_assumption(Formula::Not(Box::new(condition_formula)));
        }
    }

    if !space_is_empty(&remaining) {
        let uncovered_cases = space_to_case_summaries(&remaining, 8);
        let suggested_arms = space_to_case_summaries(&remaining, 4);
        return Err(TypeError::new(
            "Non-exhaustive match expression".to_string(),
            Some(match_expr.location),
        )
        .with_code(codes::typecheck::MATCH_NON_EXHAUSTIVE)
        .with_detail(
            "scrutineeType",
            format_type(&env.normalize_type(scrutinee_type)),
        )
        .with_detail("matchLocation", match_expr.location.start.line)
        .with_detail("uncoveredCases", uncovered_cases)
        .with_detail("suggestedMissingArms", suggested_arms)
        .with_detail("coveredBy", previous_facts)
        .with_detail("proofFragment", proof_fragment())
        .with_detail("unsupportedFacts", unsupported_facts));
    }

    Ok(())
}

fn proof_fragment() -> serde_json::Value {
    serde_json::json!({
        "constructs": [
            "constructors",
            "bool_literals",
            "unit_literal",
            "list_shapes",
            "tuple_shapes",
            "int_literal_equality",
            "int_literal_order",
            "bool_and_or_not",
            "refinement_bool_aliases",
            "match_guard_refinement_facts"
        ]
    })
}

fn arm_proof(
    env: &TypeEnvironment,
    proof_context: &ProofContext,
    scrutinee: &Expr,
    scrutinee_type: &InferenceType,
    arm: &sigil_ast::MatchArm,
) -> Result<ArmProof, TypeError> {
    let mut bindings = HashMap::new();
    let mut visiting = std::collections::BTreeSet::new();
    let mut base_space = pattern_to_space(
        env,
        scrutinee_type,
        &arm.pattern,
        &mut bindings,
        &vec![],
        &mut visiting,
    )?;
    let scrutinee_roots = coverage_scrutinee_roots(env, proof_context, scrutinee, scrutinee_type);
    let arm_refinement = match_arm_refinement(env, proof_context, scrutinee, scrutinee_type, arm)?;
    let mut facts = vec![pattern_summary(&arm.pattern)];
    if let Some(guard) = &arm.guard {
        facts.push(expr_summary(guard));
    }

    if let Some(condition_formula) = &arm_refinement.condition_formula {
        if let Some(guard_space) =
            formula_to_space_subset(&base_space, condition_formula, &scrutinee_roots)
        {
            base_space = space_intersection(&base_space, &guard_space);
        } else if arm.guard.is_some() {
            return Ok(ArmProof {
                space: base_space,
                guard_supported: false,
                facts,
                unsupported_facts: arm_refinement.unsupported_facts,
                condition_formula: None,
            });
        }
    }

    Ok(ArmProof {
        space: base_space,
        guard_supported: arm_refinement.guard_supported,
        facts,
        unsupported_facts: arm_refinement.unsupported_facts,
        condition_formula: arm_refinement.condition_formula,
    })
}

fn coverage_scrutinee_roots(
    env: &TypeEnvironment,
    proof_context: &ProofContext,
    scrutinee: &Expr,
    scrutinee_type: &InferenceType,
) -> Vec<SymbolPath> {
    let Some(symbolic) = scrutinee_symbolic_value(env, proof_context, scrutinee, scrutinee_type)
    else {
        return vec![];
    };

    match symbolic {
        SymbolicValue::Int(linear) if linear.constant == 0 => linear
            .form
            .single_term()
            .filter(|(_, coeff)| *coeff == 1)
            .map(|(path, _)| vec![path.clone()])
            .unwrap_or_default(),
        SymbolicValue::Bool(Formula::Atom(Atom::BoolEq { path, value: true })) => vec![path],
        SymbolicValue::Collection(SymbolicCollection::Path(path)) => vec![path],
        SymbolicValue::Record(SymbolicRecord::Path { base, .. }) => vec![base],
        _ => vec![],
    }
}

fn symbol_path_to_value_path(
    path: &SymbolPath,
    scrutinee_roots: &[SymbolPath],
) -> Option<ValuePath> {
    let mut parts = path.0.iter();
    let first = parts.next()?;
    match first {
        SymbolPathStep::Binding(name) if name == MATCH_SCRUTINEE_BINDING => {}
        SymbolPathStep::Binding(_) => {
            let matched_root = scrutinee_roots
                .iter()
                .find(|candidate| candidate.0.first() == Some(first))?;
            if path.0.len() < matched_root.0.len()
                || path.0[..matched_root.0.len()] != matched_root.0[..]
            {
                return None;
            }
            let mut value_path = Vec::new();
            for step in &path.0[matched_root.0.len()..] {
                match step {
                    SymbolPathStep::Field(name) => {
                        value_path.push(ValuePathStep::Field(name.clone()))
                    }
                    SymbolPathStep::VariantField(index) => {
                        value_path.push(ValuePathStep::VariantField(*index))
                    }
                    SymbolPathStep::TupleIndex(index) => {
                        value_path.push(ValuePathStep::TupleIndex(*index))
                    }
                    SymbolPathStep::ListHead => value_path.push(ValuePathStep::ListHead),
                    SymbolPathStep::ListTail => value_path.push(ValuePathStep::ListTail),
                    SymbolPathStep::Binding(_) | SymbolPathStep::Length => return None,
                }
            }
            return Some(value_path);
        }
        _ => return None,
    }

    let mut value_path = Vec::new();
    for step in parts {
        match step {
            SymbolPathStep::Field(name) => value_path.push(ValuePathStep::Field(name.clone())),
            SymbolPathStep::VariantField(index) => {
                value_path.push(ValuePathStep::VariantField(*index))
            }
            SymbolPathStep::TupleIndex(index) => value_path.push(ValuePathStep::TupleIndex(*index)),
            SymbolPathStep::ListHead => value_path.push(ValuePathStep::ListHead),
            SymbolPathStep::ListTail => value_path.push(ValuePathStep::ListTail),
            SymbolPathStep::Binding(_) | SymbolPathStep::Length => return None,
        }
    }

    Some(value_path)
}

fn formula_to_space_subset(
    base_space: &MatchSpace,
    formula: &Formula,
    scrutinee_roots: &[SymbolPath],
) -> Option<MatchSpace> {
    match formula {
        Formula::True => Some(base_space.clone()),
        Formula::False => Some(MatchSpace::Empty),
        Formula::Atom(atom) => atom_to_space_subset(base_space, atom, scrutinee_roots),
        Formula::And(parts) => {
            let mut current = base_space.clone();
            for part in parts {
                current = space_intersection(
                    &current,
                    &formula_to_space_subset(&current, part, scrutinee_roots)?,
                );
            }
            Some(current)
        }
        Formula::Or(parts) => {
            let mut spaces = Vec::new();
            for part in parts {
                spaces.push(formula_to_space_subset(base_space, part, scrutinee_roots)?);
            }
            Some(normalize_space(MatchSpace::Union(spaces)))
        }
        Formula::Not(part) => {
            let inner = formula_to_space_subset(base_space, part, scrutinee_roots)?;
            Some(space_difference(base_space, &inner))
        }
    }
}

fn atom_to_space_subset(
    base_space: &MatchSpace,
    atom: &Atom,
    scrutinee_roots: &[SymbolPath],
) -> Option<MatchSpace> {
    match atom {
        Atom::BoolEq { path, value } => {
            let value_path = symbol_path_to_value_path(path, scrutinee_roots)?;
            let constraint = MatchSpace::Primitive(PrimitiveSpace::Bool {
                allow_true: *value,
                allow_false: !*value,
            });
            refine_space_at_path(base_space, &value_path, &constraint)
        }
        Atom::IntCmp { form, op, rhs } => {
            if form.terms.is_empty() {
                return Some(
                    if matches!(
                        prove_formula(&[], &Formula::Atom(atom.clone())).outcome,
                        SolverOutcome::Proved
                    ) {
                        base_space.clone()
                    } else {
                        MatchSpace::Empty
                    },
                );
            }

            let (path, coeff) = form.single_term()?;
            let value_path = symbol_path_to_value_path(path, scrutinee_roots)?;
            let constraint = int_constraint_space(coeff, *op, *rhs)?;
            refine_space_at_path(base_space, &value_path, &constraint)
        }
        Atom::StateEq { .. } => None,
    }
}

fn int_constraint_space(coeff: i64, op: ComparisonOp, rhs: i64) -> Option<MatchSpace> {
    let ranges = match op {
        ComparisonOp::Eq => IntRangeSet::singleton(solve_exact_single_var(coeff, rhs)?),
        ComparisonOp::Ne => {
            let value = solve_exact_single_var(coeff, rhs)?;
            IntRangeSet::all().difference(&IntRangeSet::singleton(value))
        }
        ComparisonOp::Lt | ComparisonOp::Le | ComparisonOp::Gt | ComparisonOp::Ge => {
            let (lower, upper) = solve_single_var_interval(coeff, op, rhs)?;
            match (lower, upper) {
                (None, None) => IntRangeSet::all(),
                (Some(lower), None) => IntRangeSet::greater_eq(lower),
                (None, Some(upper)) => IntRangeSet::less_eq(upper),
                (Some(lower), Some(upper)) if lower <= upper => IntRangeSet {
                    intervals: vec![IntInterval {
                        start: Some(lower),
                        end: Some(upper),
                    }],
                },
                (Some(_), Some(_)) => IntRangeSet::empty(),
            }
        }
    };

    Some(MatchSpace::Primitive(PrimitiveSpace::Int(ranges)))
}

pub(crate) fn total_space_for_type(
    env: &TypeEnvironment,
    typ: &InferenceType,
) -> Result<MatchSpace, TypeError> {
    total_space_for_type_inner(env, typ, &mut std::collections::BTreeSet::new())
}

fn total_space_for_type_inner(
    env: &TypeEnvironment,
    typ: &InferenceType,
    visiting: &mut std::collections::BTreeSet<String>,
) -> Result<MatchSpace, TypeError> {
    let normalized = env.normalize_type(typ);
    let recursion_key = match &normalized {
        InferenceType::Constructor(constructor) => Some(format_type(&InferenceType::Constructor(
            constructor.clone(),
        ))),
        _ => None,
    };

    if let Some(key) = &recursion_key {
        if !visiting.insert(key.clone()) {
            return Ok(MatchSpace::Opaque(key.clone()));
        }
    }

    let result = match normalized {
        InferenceType::Primitive(TPrimitive {
            name: PrimitiveName::Bool,
        }) => Ok(MatchSpace::Primitive(PrimitiveSpace::Bool {
            allow_true: true,
            allow_false: true,
        })),
        InferenceType::Primitive(TPrimitive {
            name: PrimitiveName::Unit,
        }) => Ok(MatchSpace::Primitive(PrimitiveSpace::Unit {
            present: true,
        })),
        InferenceType::Primitive(TPrimitive {
            name: PrimitiveName::Int,
        }) => Ok(MatchSpace::Primitive(PrimitiveSpace::Int(
            IntRangeSet::all(),
        ))),
        InferenceType::Primitive(TPrimitive {
            name: PrimitiveName::String,
        }) => Ok(MatchSpace::Primitive(PrimitiveSpace::EqAny {
            kind: PrimitiveEqKind::String,
            excluded: std::collections::BTreeSet::new(),
        })),
        InferenceType::Primitive(TPrimitive {
            name: PrimitiveName::Char,
        }) => Ok(MatchSpace::Primitive(PrimitiveSpace::EqAny {
            kind: PrimitiveEqKind::Char,
            excluded: std::collections::BTreeSet::new(),
        })),
        InferenceType::Primitive(TPrimitive {
            name: PrimitiveName::Float,
        }) => Ok(MatchSpace::Primitive(PrimitiveSpace::EqAny {
            kind: PrimitiveEqKind::Float,
            excluded: std::collections::BTreeSet::new(),
        })),
        InferenceType::Primitive(TPrimitive {
            name: PrimitiveName::Never,
        }) => Ok(MatchSpace::Empty),
        InferenceType::Tuple(tuple) => Ok(MatchSpace::Tuple(
            tuple
                .types
                .iter()
                .map(|item| total_space_for_type_inner(env, item, visiting))
                .collect::<Result<Vec<_>, _>>()?,
        )),
        InferenceType::Record(record) => Ok(MatchSpace::Record(RecordSpace {
            fields: sorted_record_field_types(&record)
                .into_iter()
                .map(|(name, field_type)| {
                    total_space_for_type_inner(env, &field_type, visiting)
                        .map(|space| (name, space))
                })
                .collect::<Result<Vec<_>, _>>()?,
        })),
        InferenceType::List(list) => Ok(MatchSpace::List(ListSpace::Any(Box::new(
            total_space_for_type_inner(env, &list.element_type, visiting)?,
        )))),
        InferenceType::Constructor(constructor) => {
            total_space_for_constructor_inner(env, &constructor, visiting)
        }
        InferenceType::Function(_)
        | InferenceType::Any
        | InferenceType::Var(_)
        | InferenceType::Map(_)
        | InferenceType::Owned(_)
        | InferenceType::Borrowed(_) => Ok(MatchSpace::Opaque(format_type(&normalized))),
    };

    if let Some(key) = &recursion_key {
        visiting.remove(key);
    }

    result
}

fn total_space_for_constructor_inner(
    env: &TypeEnvironment,
    constructor: &TConstructor,
    visiting: &mut std::collections::BTreeSet<String>,
) -> Result<MatchSpace, TypeError> {
    let Some((result_name, info)) = lookup_type_info_for_constructor(env, constructor) else {
        return Ok(MatchSpace::Opaque(format_type(
            &InferenceType::Constructor(constructor.clone()),
        )));
    };

    let TypeDef::Sum(sum_type) = info.definition else {
        return Ok(MatchSpace::Opaque(result_name));
    };

    let mut variants = Vec::new();
    for variant in &sum_type.variants {
        let field_spaces = instantiate_variant_field_spaces(
            env,
            variant,
            &info.type_params,
            &result_name,
            &constructor.type_args,
            visiting,
        )?;
        variants.push(MatchSpace::Variant(VariantSpace {
            owner: result_name.clone(),
            name: variant.name.clone(),
            fields: field_spaces,
        }));
    }

    Ok(normalize_space(MatchSpace::Union(variants)))
}

fn lookup_type_info_for_constructor(
    env: &TypeEnvironment,
    constructor: &TConstructor,
) -> Option<(String, TypeInfo)> {
    if let Some((module_path, type_name)) = split_qualified_constructor_name(&constructor.name) {
        env.lookup_qualified_type(&module_path, &type_name)
            .map(|info| (constructor.name.clone(), info))
    } else {
        env.lookup_type(&constructor.name)
            .map(|info| (constructor.name.clone(), info))
    }
}

fn instantiate_variant_field_spaces(
    env: &TypeEnvironment,
    variant: &sigil_ast::Variant,
    type_params: &[String],
    result_name: &str,
    result_type_args: &[InferenceType],
    visiting: &mut std::collections::BTreeSet<String>,
) -> Result<Vec<MatchSpace>, TypeError> {
    let ctor_type =
        create_constructor_type_with_result_name(env, variant, type_params, result_name)?;
    let InferenceType::Function(ctor_fn) = ctor_type else {
        return Ok(vec![]);
    };
    let expected_result = InferenceType::Constructor(TConstructor {
        name: result_name.to_string(),
        type_args: result_type_args.to_vec(),
    });
    let subst = unify(&ctor_fn.return_type, &expected_result).map_err(|message| {
        TypeError::new(
            format!(
                "Could not instantiate variant '{}' for '{}': {}",
                variant.name, result_name, message
            ),
            Some(variant.location),
        )
    })?;

    ctor_fn
        .params
        .iter()
        .map(|param| total_space_for_type_inner(env, &apply_subst(&subst, param), visiting))
        .collect()
}

pub(crate) fn pattern_to_space(
    env: &TypeEnvironment,
    scrutinee_type: &InferenceType,
    pattern: &sigil_ast::Pattern,
    bindings: &mut HashMap<String, ValuePath>,
    path: &ValuePath,
    visiting: &mut std::collections::BTreeSet<String>,
) -> Result<MatchSpace, TypeError> {
    use sigil_ast::{Pattern, PatternLiteralType, PatternLiteralValue};
    let normalized_scrutinee_type = env.normalize_type(scrutinee_type);

    match pattern {
        Pattern::Wildcard(_) => {
            total_space_for_type_inner(env, &normalized_scrutinee_type, visiting)
        }
        Pattern::Identifier(identifier) => {
            bindings.insert(identifier.name.clone(), path.clone());
            total_space_for_type_inner(env, &normalized_scrutinee_type, visiting)
        }
        Pattern::Literal(literal) => Ok(match literal.literal_type {
            PatternLiteralType::Bool => MatchSpace::Primitive(PrimitiveSpace::Bool {
                allow_true: matches!(literal.value, PatternLiteralValue::Bool(true)),
                allow_false: matches!(literal.value, PatternLiteralValue::Bool(false)),
            }),
            PatternLiteralType::Unit => {
                MatchSpace::Primitive(PrimitiveSpace::Unit { present: true })
            }
            PatternLiteralType::Int => MatchSpace::Primitive(PrimitiveSpace::Int(
                IntRangeSet::singleton(match literal.value {
                    PatternLiteralValue::Int(value) => value,
                    _ => 0,
                }),
            )),
            PatternLiteralType::String => MatchSpace::Primitive(PrimitiveSpace::EqFinite {
                kind: PrimitiveEqKind::String,
                values: std::collections::BTreeSet::from([LiteralAtom::String(
                    match &literal.value {
                        PatternLiteralValue::String(value) => value.clone(),
                        _ => String::new(),
                    },
                )]),
            }),
            PatternLiteralType::Char => MatchSpace::Primitive(PrimitiveSpace::EqFinite {
                kind: PrimitiveEqKind::Char,
                values: std::collections::BTreeSet::from([LiteralAtom::Char(
                    match literal.value {
                        PatternLiteralValue::Char(value) => value,
                        _ => '\0',
                    },
                )]),
            }),
            PatternLiteralType::Float => MatchSpace::Primitive(PrimitiveSpace::EqFinite {
                kind: PrimitiveEqKind::Float,
                values: std::collections::BTreeSet::from([LiteralAtom::Float(
                    match literal.value {
                        PatternLiteralValue::Float(value) => value.to_bits(),
                        _ => 0.0f64.to_bits(),
                    },
                )]),
            }),
        }),
        Pattern::Tuple(tuple_pattern) => {
            let InferenceType::Tuple(tuple_type) = &normalized_scrutinee_type else {
                return Ok(MatchSpace::Empty);
            };
            let mut items = Vec::new();
            for (index, (item_pattern, item_type)) in tuple_pattern
                .patterns
                .iter()
                .zip(tuple_type.types.iter())
                .enumerate()
            {
                let mut item_path = path.clone();
                item_path.push(ValuePathStep::TupleIndex(index));
                items.push(pattern_to_space(
                    env,
                    item_type,
                    item_pattern,
                    bindings,
                    &item_path,
                    visiting,
                )?);
            }
            Ok(MatchSpace::Tuple(items))
        }
        Pattern::List(list_pattern) => {
            let InferenceType::List(list_type) = &normalized_scrutinee_type else {
                return Ok(MatchSpace::Empty);
            };
            list_pattern_to_space(
                env,
                &list_type.element_type,
                &list_pattern.patterns,
                list_pattern.rest.as_ref(),
                bindings,
                path,
                visiting,
            )
        }
        Pattern::Constructor(constructor_pattern) => {
            let constructor_type = lookup_constructor_type(
                env,
                &constructor_pattern.module_path,
                &constructor_pattern.name,
            )?
            .ok_or_else(|| {
                TypeError::new(
                    format!(
                        "Unknown constructor '{}'",
                        constructor_display_name(
                            &constructor_pattern.module_path,
                            &constructor_pattern.name
                        )
                    ),
                    Some(constructor_pattern.location),
                )
            })?;
            let InferenceType::Function(ctor_fn) = constructor_type else {
                return Ok(MatchSpace::Empty);
            };
            let subst =
                unify(&ctor_fn.return_type, &normalized_scrutinee_type).map_err(|message| {
                    TypeError::new(
                        format!(
                            "Constructor '{}' does not match scrutinee type {} ({})",
                            constructor_display_name(
                                &constructor_pattern.module_path,
                                &constructor_pattern.name
                            ),
                            format_type(&normalized_scrutinee_type),
                            message
                        ),
                        Some(constructor_pattern.location),
                    )
                })?;
            let owner = match &ctor_fn.return_type {
                InferenceType::Constructor(return_ctor) => return_ctor.name.clone(),
                _ => format_type(&normalized_scrutinee_type),
            };
            let recursion_key = format_type(&normalized_scrutinee_type);
            let inserted = visiting.insert(recursion_key.clone());
            let mut fields = Vec::new();
            for (index, (pattern, field_type)) in constructor_pattern
                .patterns
                .iter()
                .zip(ctor_fn.params.iter())
                .enumerate()
            {
                let mut field_path = path.clone();
                field_path.push(ValuePathStep::VariantField(index));
                fields.push(pattern_to_space(
                    env,
                    &apply_subst(&subst, field_type),
                    pattern,
                    bindings,
                    &field_path,
                    visiting,
                )?);
            }
            if inserted {
                visiting.remove(&recursion_key);
            }
            Ok(MatchSpace::Variant(VariantSpace {
                owner,
                name: constructor_pattern.name.clone(),
                fields,
            }))
        }
        Pattern::Record(record_pattern) => {
            let InferenceType::Record(record_type) = &normalized_scrutinee_type else {
                return Ok(MatchSpace::Empty);
            };
            let expected_fields = sorted_record_field_types(record_type);
            if record_pattern.fields.len() != expected_fields.len() {
                return Ok(MatchSpace::Empty);
            }

            let mut fields = Vec::new();
            for (field_name, field_type) in expected_fields {
                let Some(field_pattern) = record_pattern
                    .fields
                    .iter()
                    .find(|field| field.name == field_name)
                else {
                    return Ok(MatchSpace::Empty);
                };
                let mut field_path = path.clone();
                field_path.push(ValuePathStep::Field(field_name.clone()));
                let field_space = if let Some(pattern) = &field_pattern.pattern {
                    pattern_to_space(env, &field_type, pattern, bindings, &field_path, visiting)?
                } else {
                    bindings.insert(field_name.clone(), field_path);
                    total_space_for_type_inner(env, &field_type, visiting)?
                };
                fields.push((field_name, field_space));
            }
            Ok(MatchSpace::Record(RecordSpace { fields }))
        }
    }
}

fn list_pattern_to_space(
    env: &TypeEnvironment,
    element_type: &InferenceType,
    patterns: &[sigil_ast::Pattern],
    rest: Option<&String>,
    bindings: &mut HashMap<String, ValuePath>,
    path: &ValuePath,
    visiting: &mut std::collections::BTreeSet<String>,
) -> Result<MatchSpace, TypeError> {
    if patterns.is_empty() {
        if let Some(rest_name) = rest {
            bindings.insert(rest_name.clone(), path.clone());
            return Ok(MatchSpace::List(ListSpace::Any(Box::new(
                total_space_for_type_inner(env, element_type, visiting)?,
            ))));
        }
        return Ok(MatchSpace::List(ListSpace::Nil));
    }

    let mut head_path = path.clone();
    head_path.push(ValuePathStep::ListHead);
    let mut tail_path = path.clone();
    tail_path.push(ValuePathStep::ListTail);

    let head = pattern_to_space(
        env,
        element_type,
        &patterns[0],
        bindings,
        &head_path,
        visiting,
    )?;
    let tail = list_pattern_to_space(
        env,
        element_type,
        &patterns[1..],
        rest,
        bindings,
        &tail_path,
        visiting,
    )?;
    Ok(MatchSpace::List(ListSpace::Cons {
        head: Box::new(head),
        tail: Box::new(tail),
    }))
}

pub(crate) fn space_is_empty(space: &MatchSpace) -> bool {
    match space {
        MatchSpace::Empty => true,
        MatchSpace::Primitive(primitive) => match primitive {
            PrimitiveSpace::Bool {
                allow_true,
                allow_false,
            } => !allow_true && !allow_false,
            PrimitiveSpace::Unit { present } => !present,
            PrimitiveSpace::Int(ranges) => ranges.is_empty(),
            PrimitiveSpace::EqAny { .. } => false,
            PrimitiveSpace::EqFinite { values, .. } => values.is_empty(),
        },
        MatchSpace::Variant(variant) => variant.fields.iter().any(space_is_empty),
        MatchSpace::Record(record) => record.fields.iter().any(|(_, field)| space_is_empty(field)),
        MatchSpace::Tuple(items) => items.iter().any(space_is_empty),
        MatchSpace::List(list) => match list {
            ListSpace::Any(_) => false,
            ListSpace::Nil => false,
            ListSpace::Cons { head, tail } => space_is_empty(head) || space_is_empty(tail),
        },
        MatchSpace::Union(items) => items.iter().all(space_is_empty),
        MatchSpace::Opaque(_) => false,
    }
}

fn normalize_space(space: MatchSpace) -> MatchSpace {
    match space {
        MatchSpace::Empty => MatchSpace::Empty,
        MatchSpace::Primitive(primitive) => normalize_primitive_space(primitive),
        MatchSpace::Variant(variant) => {
            let fields = variant
                .fields
                .into_iter()
                .map(normalize_space)
                .collect::<Vec<_>>();
            if fields.iter().any(space_is_empty) {
                MatchSpace::Empty
            } else {
                MatchSpace::Variant(VariantSpace {
                    owner: variant.owner,
                    name: variant.name,
                    fields,
                })
            }
        }
        MatchSpace::Record(record) => {
            let fields = record
                .fields
                .into_iter()
                .map(|(name, field)| (name, normalize_space(field)))
                .collect::<Vec<_>>();
            if fields.iter().any(|(_, field)| space_is_empty(field)) {
                MatchSpace::Empty
            } else {
                MatchSpace::Record(RecordSpace { fields })
            }
        }
        MatchSpace::Tuple(items) => {
            let items = items.into_iter().map(normalize_space).collect::<Vec<_>>();
            if items.iter().any(space_is_empty) {
                MatchSpace::Empty
            } else {
                MatchSpace::Tuple(items)
            }
        }
        MatchSpace::List(list) => normalize_list_space(list),
        MatchSpace::Union(items) => normalize_union(items),
        MatchSpace::Opaque(name) => MatchSpace::Opaque(name),
    }
}

fn normalize_primitive_space(space: PrimitiveSpace) -> MatchSpace {
    match space {
        PrimitiveSpace::Bool {
            allow_true,
            allow_false,
        } if !allow_true && !allow_false => MatchSpace::Empty,
        PrimitiveSpace::Unit { present: false } => MatchSpace::Empty,
        PrimitiveSpace::Int(ranges) if ranges.is_empty() => MatchSpace::Empty,
        PrimitiveSpace::EqFinite { values, .. } if values.is_empty() => MatchSpace::Empty,
        primitive => MatchSpace::Primitive(primitive),
    }
}

fn normalize_list_space(list: ListSpace) -> MatchSpace {
    match list {
        ListSpace::Any(element) => {
            let element = normalize_space(*element);
            if space_is_empty(&element) {
                MatchSpace::List(ListSpace::Nil)
            } else {
                MatchSpace::List(ListSpace::Any(Box::new(element)))
            }
        }
        ListSpace::Nil => MatchSpace::List(ListSpace::Nil),
        ListSpace::Cons { head, tail } => {
            let head = normalize_space(*head);
            let tail = normalize_space(*tail);
            if space_is_empty(&head) || space_is_empty(&tail) {
                MatchSpace::Empty
            } else {
                MatchSpace::List(ListSpace::Cons {
                    head: Box::new(head),
                    tail: Box::new(tail),
                })
            }
        }
    }
}

fn normalize_union(items: Vec<MatchSpace>) -> MatchSpace {
    let mut flattened = Vec::new();
    for item in items {
        match normalize_space(item) {
            MatchSpace::Empty => {}
            MatchSpace::Union(nested) => flattened.extend(nested),
            other => flattened.push(other),
        }
    }

    if flattened.is_empty() {
        return MatchSpace::Empty;
    }

    let mut merged: Vec<MatchSpace> = Vec::new();
    for item in flattened {
        if let Some(existing) = merged
            .iter_mut()
            .find(|existing| can_merge_union(existing, &item))
        {
            *existing = merge_union_item(existing.clone(), item);
        } else if !merged.contains(&item) {
            merged.push(item);
        }
    }

    if merged.len() == 1 {
        merged.into_iter().next().unwrap()
    } else {
        MatchSpace::Union(merged)
    }
}

fn can_merge_union(left: &MatchSpace, right: &MatchSpace) -> bool {
    match (left, right) {
        (MatchSpace::Primitive(left), MatchSpace::Primitive(right)) => {
            primitive_same_kind(left, right)
        }
        _ => false,
    }
}

fn merge_union_item(left: MatchSpace, right: MatchSpace) -> MatchSpace {
    match (left, right) {
        (MatchSpace::Primitive(left), MatchSpace::Primitive(right)) => {
            normalize_primitive_space(primitive_union(&left, &right))
        }
        (left, _) => left,
    }
}

fn primitive_same_kind(left: &PrimitiveSpace, right: &PrimitiveSpace) -> bool {
    match (left, right) {
        (PrimitiveSpace::Bool { .. }, PrimitiveSpace::Bool { .. }) => true,
        (PrimitiveSpace::Unit { .. }, PrimitiveSpace::Unit { .. }) => true,
        (PrimitiveSpace::Int(_), PrimitiveSpace::Int(_)) => true,
        (PrimitiveSpace::EqAny { kind: left, .. }, PrimitiveSpace::EqAny { kind: right, .. }) => {
            left == right
        }
        (
            PrimitiveSpace::EqAny { kind: left, .. },
            PrimitiveSpace::EqFinite { kind: right, .. },
        ) => left == right,
        (
            PrimitiveSpace::EqFinite { kind: left, .. },
            PrimitiveSpace::EqAny { kind: right, .. },
        ) => left == right,
        (
            PrimitiveSpace::EqFinite { kind: left, .. },
            PrimitiveSpace::EqFinite { kind: right, .. },
        ) => left == right,
        _ => false,
    }
}

fn primitive_union(left: &PrimitiveSpace, right: &PrimitiveSpace) -> PrimitiveSpace {
    match (left, right) {
        (
            PrimitiveSpace::Bool {
                allow_true: left_true,
                allow_false: left_false,
            },
            PrimitiveSpace::Bool {
                allow_true: right_true,
                allow_false: right_false,
            },
        ) => PrimitiveSpace::Bool {
            allow_true: *left_true || *right_true,
            allow_false: *left_false || *right_false,
        },
        (PrimitiveSpace::Unit { present: left }, PrimitiveSpace::Unit { present: right }) => {
            PrimitiveSpace::Unit {
                present: *left || *right,
            }
        }
        (PrimitiveSpace::Int(left), PrimitiveSpace::Int(right)) => {
            PrimitiveSpace::Int(left.union(right))
        }
        (
            PrimitiveSpace::EqFinite {
                kind,
                values: left_values,
            },
            PrimitiveSpace::EqFinite {
                values: right_values,
                ..
            },
        ) => {
            let mut values = left_values.clone();
            values.extend(right_values.iter().cloned());
            PrimitiveSpace::EqFinite {
                kind: kind.clone(),
                values,
            }
        }
        (
            PrimitiveSpace::EqAny {
                kind,
                excluded: left_excluded,
            },
            PrimitiveSpace::EqAny {
                excluded: right_excluded,
                ..
            },
        ) => PrimitiveSpace::EqAny {
            kind: kind.clone(),
            excluded: left_excluded
                .intersection(right_excluded)
                .cloned()
                .collect(),
        },
        (PrimitiveSpace::EqAny { kind, excluded }, PrimitiveSpace::EqFinite { values, .. })
        | (PrimitiveSpace::EqFinite { values, .. }, PrimitiveSpace::EqAny { kind, excluded }) => {
            let mut next_excluded = excluded.clone();
            for value in values {
                next_excluded.remove(value);
            }
            PrimitiveSpace::EqAny {
                kind: kind.clone(),
                excluded: next_excluded,
            }
        }
        _ => left.clone(),
    }
}

fn primitive_intersection(left: &PrimitiveSpace, right: &PrimitiveSpace) -> MatchSpace {
    if !primitive_same_kind(left, right) {
        return MatchSpace::Empty;
    }

    normalize_primitive_space(match (left, right) {
        (
            PrimitiveSpace::Bool {
                allow_true: left_true,
                allow_false: left_false,
            },
            PrimitiveSpace::Bool {
                allow_true: right_true,
                allow_false: right_false,
            },
        ) => PrimitiveSpace::Bool {
            allow_true: *left_true && *right_true,
            allow_false: *left_false && *right_false,
        },
        (PrimitiveSpace::Unit { present: left }, PrimitiveSpace::Unit { present: right }) => {
            PrimitiveSpace::Unit {
                present: *left && *right,
            }
        }
        (PrimitiveSpace::Int(left), PrimitiveSpace::Int(right)) => {
            PrimitiveSpace::Int(left.intersect(right))
        }
        (
            PrimitiveSpace::EqFinite {
                kind,
                values: left_values,
            },
            PrimitiveSpace::EqFinite {
                values: right_values,
                ..
            },
        ) => PrimitiveSpace::EqFinite {
            kind: kind.clone(),
            values: left_values.intersection(right_values).cloned().collect(),
        },
        (PrimitiveSpace::EqAny { kind, excluded }, PrimitiveSpace::EqFinite { values, .. })
        | (PrimitiveSpace::EqFinite { values, .. }, PrimitiveSpace::EqAny { kind, excluded }) => {
            PrimitiveSpace::EqFinite {
                kind: kind.clone(),
                values: values
                    .iter()
                    .filter(|value| !excluded.contains(*value))
                    .cloned()
                    .collect(),
            }
        }
        (
            PrimitiveSpace::EqAny {
                kind,
                excluded: left_excluded,
            },
            PrimitiveSpace::EqAny {
                excluded: right_excluded,
                ..
            },
        ) => PrimitiveSpace::EqAny {
            kind: kind.clone(),
            excluded: left_excluded.union(right_excluded).cloned().collect(),
        },
        _ => return MatchSpace::Empty,
    })
}

fn primitive_difference(left: &PrimitiveSpace, right: &PrimitiveSpace) -> MatchSpace {
    if !primitive_same_kind(left, right) {
        return MatchSpace::Primitive(left.clone());
    }

    normalize_primitive_space(match (left, right) {
        (
            PrimitiveSpace::Bool {
                allow_true: left_true,
                allow_false: left_false,
            },
            PrimitiveSpace::Bool {
                allow_true: right_true,
                allow_false: right_false,
            },
        ) => PrimitiveSpace::Bool {
            allow_true: *left_true && !*right_true,
            allow_false: *left_false && !*right_false,
        },
        (PrimitiveSpace::Unit { present: left }, PrimitiveSpace::Unit { present: right }) => {
            PrimitiveSpace::Unit {
                present: *left && !*right,
            }
        }
        (PrimitiveSpace::Int(left), PrimitiveSpace::Int(right)) => {
            PrimitiveSpace::Int(left.difference(right))
        }
        (
            PrimitiveSpace::EqFinite {
                kind,
                values: left_values,
            },
            PrimitiveSpace::EqFinite {
                values: right_values,
                ..
            },
        ) => PrimitiveSpace::EqFinite {
            kind: kind.clone(),
            values: left_values.difference(right_values).cloned().collect(),
        },
        (
            PrimitiveSpace::EqFinite {
                kind,
                values: left_values,
            },
            PrimitiveSpace::EqAny { excluded, .. },
        ) => PrimitiveSpace::EqFinite {
            kind: kind.clone(),
            values: left_values.intersection(excluded).cloned().collect(),
        },
        (
            PrimitiveSpace::EqAny {
                kind,
                excluded: left_excluded,
            },
            PrimitiveSpace::EqFinite { values, .. },
        ) => {
            let mut next = left_excluded.clone();
            next.extend(values.iter().cloned());
            PrimitiveSpace::EqAny {
                kind: kind.clone(),
                excluded: next,
            }
        }
        (
            PrimitiveSpace::EqAny {
                kind,
                excluded: left_excluded,
            },
            PrimitiveSpace::EqAny {
                excluded: right_excluded,
                ..
            },
        ) => PrimitiveSpace::EqFinite {
            kind: kind.clone(),
            values: right_excluded.difference(left_excluded).cloned().collect(),
        },
        _ => return MatchSpace::Primitive(left.clone()),
    })
}

pub(crate) fn space_intersection(left: &MatchSpace, right: &MatchSpace) -> MatchSpace {
    match (left, right) {
        (MatchSpace::Empty, _) | (_, MatchSpace::Empty) => MatchSpace::Empty,
        (MatchSpace::Union(items), other) | (other, MatchSpace::Union(items)) => {
            normalize_space(MatchSpace::Union(
                items
                    .iter()
                    .map(|item| space_intersection(item, other))
                    .collect(),
            ))
        }
        (MatchSpace::Primitive(left), MatchSpace::Primitive(right)) => {
            primitive_intersection(left, right)
        }
        (MatchSpace::Variant(left), MatchSpace::Variant(right))
            if left.owner == right.owner
                && left.name == right.name
                && left.fields.len() == right.fields.len() =>
        {
            normalize_space(MatchSpace::Variant(VariantSpace {
                owner: left.owner.clone(),
                name: left.name.clone(),
                fields: left
                    .fields
                    .iter()
                    .zip(right.fields.iter())
                    .map(|(left, right)| space_intersection(left, right))
                    .collect(),
            }))
        }
        (MatchSpace::Record(left), MatchSpace::Record(right))
            if left.fields.len() == right.fields.len()
                && left
                    .fields
                    .iter()
                    .zip(right.fields.iter())
                    .all(|((left_name, _), (right_name, _))| left_name == right_name) =>
        {
            normalize_space(MatchSpace::Record(RecordSpace {
                fields: left
                    .fields
                    .iter()
                    .zip(right.fields.iter())
                    .map(|((name, left), (_, right))| {
                        (name.clone(), space_intersection(left, right))
                    })
                    .collect(),
            }))
        }
        (MatchSpace::Tuple(left), MatchSpace::Tuple(right)) if left.len() == right.len() => {
            normalize_space(MatchSpace::Tuple(
                left.iter()
                    .zip(right.iter())
                    .map(|(left, right)| space_intersection(left, right))
                    .collect(),
            ))
        }
        (MatchSpace::List(left), MatchSpace::List(right)) => {
            normalize_space(MatchSpace::List(list_intersection(left, right)))
        }
        (MatchSpace::Opaque(left), MatchSpace::Opaque(right)) if left == right => {
            MatchSpace::Opaque(left.clone())
        }
        _ => MatchSpace::Empty,
    }
}

fn list_intersection(left: &ListSpace, right: &ListSpace) -> ListSpace {
    match (left, right) {
        (ListSpace::Nil, ListSpace::Nil) => ListSpace::Nil,
        (ListSpace::Nil, ListSpace::Any(_)) | (ListSpace::Any(_), ListSpace::Nil) => ListSpace::Nil,
        (ListSpace::Nil, ListSpace::Cons { .. }) | (ListSpace::Cons { .. }, ListSpace::Nil) => {
            ListSpace::Cons {
                head: Box::new(MatchSpace::Empty),
                tail: Box::new(MatchSpace::Empty),
            }
        }
        (ListSpace::Any(left_element), ListSpace::Any(right_element)) => {
            ListSpace::Any(Box::new(space_intersection(left_element, right_element)))
        }
        (ListSpace::Any(element), ListSpace::Cons { head, tail })
        | (ListSpace::Cons { head, tail }, ListSpace::Any(element)) => ListSpace::Cons {
            head: Box::new(space_intersection(element, head)),
            tail: Box::new(space_intersection(
                &MatchSpace::List(ListSpace::Any(element.clone())),
                tail,
            )),
        },
        (
            ListSpace::Cons {
                head: left_head,
                tail: left_tail,
            },
            ListSpace::Cons {
                head: right_head,
                tail: right_tail,
            },
        ) => ListSpace::Cons {
            head: Box::new(space_intersection(left_head, right_head)),
            tail: Box::new(space_intersection(left_tail, right_tail)),
        },
    }
}

fn space_difference(base: &MatchSpace, remove: &MatchSpace) -> MatchSpace {
    match (base, remove) {
        (MatchSpace::Empty, _) => MatchSpace::Empty,
        (_, MatchSpace::Empty) => base.clone(),
        (MatchSpace::Union(items), other) => normalize_space(MatchSpace::Union(
            items
                .iter()
                .map(|item| space_difference(item, other))
                .collect(),
        )),
        (other, MatchSpace::Union(items)) => items.iter().fold(other.clone(), |current, item| {
            space_difference(&current, item)
        }),
        _ if space_subset_of(base, remove) => MatchSpace::Empty,
        (MatchSpace::Primitive(left), MatchSpace::Primitive(right)) => {
            primitive_difference(left, right)
        }
        (MatchSpace::Variant(left), MatchSpace::Variant(right))
            if left.owner == right.owner
                && left.name == right.name
                && left.fields.len() == right.fields.len() =>
        {
            difference_variant(left, right)
        }
        (MatchSpace::Variant(_), MatchSpace::Variant(_)) => base.clone(),
        (MatchSpace::Record(left), MatchSpace::Record(right))
            if left.fields.len() == right.fields.len()
                && left
                    .fields
                    .iter()
                    .zip(right.fields.iter())
                    .all(|((left_name, _), (right_name, _))| left_name == right_name) =>
        {
            difference_record(left, right)
        }
        (MatchSpace::Record(_), MatchSpace::Record(_)) => base.clone(),
        (MatchSpace::Tuple(left), MatchSpace::Tuple(right)) if left.len() == right.len() => {
            difference_tuple(left, right)
        }
        (MatchSpace::Tuple(_), MatchSpace::Tuple(_)) => base.clone(),
        (MatchSpace::List(left), MatchSpace::List(right)) => {
            normalize_space(list_difference(left, right))
        }
        (MatchSpace::Opaque(left), MatchSpace::Opaque(right)) if left == right => MatchSpace::Empty,
        (MatchSpace::Opaque(_), MatchSpace::Opaque(_)) => base.clone(),
        _ => base.clone(),
    }
}

fn space_subset_of(left: &MatchSpace, right: &MatchSpace) -> bool {
    if left == right {
        return true;
    }

    match (left, right) {
        (MatchSpace::Empty, _) => true,
        (MatchSpace::Union(items), other) => items.iter().all(|item| space_subset_of(item, other)),
        (_, MatchSpace::Union(items)) => items.iter().any(|item| space_subset_of(left, item)),
        (MatchSpace::Primitive(left), MatchSpace::Primitive(right)) => {
            space_is_empty(&primitive_difference(left, right))
        }
        (MatchSpace::Variant(left), MatchSpace::Variant(right)) => {
            left.owner == right.owner
                && left.name == right.name
                && left.fields.len() == right.fields.len()
                && left
                    .fields
                    .iter()
                    .zip(right.fields.iter())
                    .all(|(left, right)| space_subset_of(left, right))
        }
        (MatchSpace::Record(left), MatchSpace::Record(right)) => {
            left.fields.len() == right.fields.len()
                && left.fields.iter().zip(right.fields.iter()).all(
                    |((left_name, left), (right_name, right))| {
                        left_name == right_name && space_subset_of(left, right)
                    },
                )
        }
        (MatchSpace::Tuple(left), MatchSpace::Tuple(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right.iter())
                    .all(|(left, right)| space_subset_of(left, right))
        }
        (MatchSpace::List(left), MatchSpace::List(right)) => list_subset_of(left, right),
        (MatchSpace::Opaque(left), MatchSpace::Opaque(right)) => left == right,
        _ => false,
    }
}

fn list_subset_of(left: &ListSpace, right: &ListSpace) -> bool {
    match (left, right) {
        (ListSpace::Nil, ListSpace::Nil) => true,
        (ListSpace::Nil, ListSpace::Any(_)) => true,
        (ListSpace::Nil, ListSpace::Cons { .. }) => false,
        (ListSpace::Cons { .. }, ListSpace::Nil) => false,
        (ListSpace::Any(_), ListSpace::Nil) => false,
        (ListSpace::Any(left_element), ListSpace::Any(right_element)) => {
            space_subset_of(left_element, right_element)
        }
        (ListSpace::Cons { head, tail }, ListSpace::Any(element)) => {
            space_subset_of(head, element)
                && space_subset_of(tail, &MatchSpace::List(ListSpace::Any(element.clone())))
        }
        (ListSpace::Any(_), ListSpace::Cons { .. }) => false,
        (
            ListSpace::Cons {
                head: left_head,
                tail: left_tail,
            },
            ListSpace::Cons {
                head: right_head,
                tail: right_tail,
            },
        ) => space_subset_of(left_head, right_head) && space_subset_of(left_tail, right_tail),
    }
}

fn difference_variant(left: &VariantSpace, right: &VariantSpace) -> MatchSpace {
    let products = difference_product_fields(&left.fields, &right.fields);
    normalize_space(MatchSpace::Union(
        products
            .into_iter()
            .map(|fields| {
                MatchSpace::Variant(VariantSpace {
                    owner: left.owner.clone(),
                    name: left.name.clone(),
                    fields,
                })
            })
            .collect(),
    ))
}

fn difference_tuple(left: &[MatchSpace], right: &[MatchSpace]) -> MatchSpace {
    let products = difference_product_fields(left, right);
    normalize_space(MatchSpace::Union(
        products.into_iter().map(MatchSpace::Tuple).collect(),
    ))
}

fn difference_record(left: &RecordSpace, right: &RecordSpace) -> MatchSpace {
    let left_fields = left
        .fields
        .iter()
        .map(|(_, field)| field.clone())
        .collect::<Vec<_>>();
    let right_fields = right
        .fields
        .iter()
        .map(|(_, field)| field.clone())
        .collect::<Vec<_>>();
    let products = difference_product_fields(&left_fields, &right_fields);
    normalize_space(MatchSpace::Union(
        products
            .into_iter()
            .map(|fields| {
                MatchSpace::Record(RecordSpace {
                    fields: left
                        .fields
                        .iter()
                        .map(|(name, _)| name.clone())
                        .zip(fields)
                        .collect(),
                })
            })
            .collect(),
    ))
}

fn difference_product_fields(base: &[MatchSpace], remove: &[MatchSpace]) -> Vec<Vec<MatchSpace>> {
    let mut results = Vec::new();
    let mut prefix = Vec::new();

    for index in 0..base.len() {
        let diff = space_difference(&base[index], &remove[index]);
        if !space_is_empty(&diff) {
            let mut fields = prefix.clone();
            fields.push(diff);
            fields.extend_from_slice(&base[index + 1..]);
            results.push(fields);
        }

        let equal = space_intersection(&base[index], &remove[index]);
        if space_is_empty(&equal) {
            break;
        }
        prefix.push(equal);
    }

    results
}

fn list_difference(base: &ListSpace, remove: &ListSpace) -> MatchSpace {
    match (base, remove) {
        (ListSpace::Nil, ListSpace::Nil) => MatchSpace::Empty,
        (ListSpace::Nil, ListSpace::Any(_)) => MatchSpace::Empty,
        (ListSpace::Nil, ListSpace::Cons { .. }) => MatchSpace::List(ListSpace::Nil),
        (ListSpace::Any(element), ListSpace::Nil) => cons_space(
            (**element).clone(),
            MatchSpace::List(ListSpace::Any(element.clone())),
        ),
        (ListSpace::Any(left_element), ListSpace::Any(right_element)) => difference_cons_like(
            left_element,
            &MatchSpace::List(ListSpace::Any(left_element.clone())),
            right_element,
            &MatchSpace::List(ListSpace::Any(right_element.clone())),
        ),
        (ListSpace::Any(element), ListSpace::Cons { head, tail }) => {
            normalize_space(MatchSpace::Union(vec![
                MatchSpace::List(ListSpace::Nil),
                difference_cons_like(
                    element,
                    &MatchSpace::List(ListSpace::Any(element.clone())),
                    head,
                    tail,
                ),
            ]))
        }
        (
            ListSpace::Cons {
                head: _left_head,
                tail: _left_tail,
            },
            ListSpace::Nil,
        ) => MatchSpace::List(base.clone()),
        (
            ListSpace::Cons {
                head: left_head,
                tail: left_tail,
            },
            ListSpace::Any(right_element),
        ) => difference_cons_like(
            left_head,
            left_tail,
            right_element,
            &MatchSpace::List(ListSpace::Any(right_element.clone())),
        ),
        (
            ListSpace::Cons {
                head: left_head,
                tail: left_tail,
            },
            ListSpace::Cons {
                head: right_head,
                tail: right_tail,
            },
        ) => difference_cons_like(left_head, left_tail, right_head, right_tail),
    }
}

fn difference_cons_like(
    base_head: &MatchSpace,
    base_tail: &MatchSpace,
    remove_head: &MatchSpace,
    remove_tail: &MatchSpace,
) -> MatchSpace {
    let products = difference_product_fields(
        &[base_head.clone(), base_tail.clone()],
        &[remove_head.clone(), remove_tail.clone()],
    );
    normalize_space(MatchSpace::Union(
        products
            .into_iter()
            .map(|fields| cons_space(fields[0].clone(), fields[1].clone()))
            .collect(),
    ))
}

fn cons_space(head: MatchSpace, tail: MatchSpace) -> MatchSpace {
    normalize_space(MatchSpace::List(ListSpace::Cons {
        head: Box::new(head),
        tail: Box::new(tail),
    }))
}

fn refine_space_at_path(
    base_space: &MatchSpace,
    path: &[ValuePathStep],
    constraint: &MatchSpace,
) -> Option<MatchSpace> {
    if path.is_empty() {
        return Some(space_intersection(base_space, constraint));
    }

    let step = &path[0];
    let rest = &path[1..];

    match base_space {
        MatchSpace::Union(items) => Some(normalize_space(MatchSpace::Union(
            items
                .iter()
                .filter_map(|item| refine_space_at_path(item, path, constraint))
                .collect(),
        ))),
        MatchSpace::Variant(variant) => {
            let ValuePathStep::VariantField(index) = step else {
                return None;
            };
            let field = variant.fields.get(*index)?;
            let refined_field = refine_space_at_path(field, rest, constraint)?;
            let mut fields = variant.fields.clone();
            fields[*index] = refined_field;
            Some(normalize_space(MatchSpace::Variant(VariantSpace {
                owner: variant.owner.clone(),
                name: variant.name.clone(),
                fields,
            })))
        }
        MatchSpace::Record(record) => {
            let ValuePathStep::Field(name) = step else {
                return None;
            };
            let index = record
                .fields
                .iter()
                .position(|(field_name, _)| field_name == name)?;
            let refined_field = refine_space_at_path(&record.fields[index].1, rest, constraint)?;
            let mut fields = record.fields.clone();
            fields[index].1 = refined_field;
            Some(normalize_space(MatchSpace::Record(RecordSpace { fields })))
        }
        MatchSpace::Tuple(items) => {
            let ValuePathStep::TupleIndex(index) = step else {
                return None;
            };
            let item = items.get(*index)?;
            let refined_item = refine_space_at_path(item, rest, constraint)?;
            let mut items = items.clone();
            items[*index] = refined_item;
            Some(normalize_space(MatchSpace::Tuple(items)))
        }
        MatchSpace::List(ListSpace::Cons { head, tail }) => match step {
            ValuePathStep::ListHead => Some(cons_space(
                refine_space_at_path(head, rest, constraint)?,
                (**tail).clone(),
            )),
            ValuePathStep::ListTail => Some(cons_space(
                (**head).clone(),
                refine_space_at_path(tail, rest, constraint)?,
            )),
            _ => None,
        },
        MatchSpace::List(ListSpace::Any(element)) => match step {
            ValuePathStep::ListHead => Some(cons_space(
                refine_space_at_path(element, rest, constraint)?,
                MatchSpace::List(ListSpace::Any(element.clone())),
            )),
            ValuePathStep::ListTail => Some(cons_space(
                (**element).clone(),
                refine_space_at_path(
                    &MatchSpace::List(ListSpace::Any(element.clone())),
                    rest,
                    constraint,
                )?,
            )),
            _ => None,
        },
        MatchSpace::List(ListSpace::Nil) => None,
        _ => None,
    }
}

fn space_to_case_summaries(space: &MatchSpace, limit: usize) -> Vec<String> {
    let mut summaries = Vec::new();
    collect_case_summaries(&normalize_space(space.clone()), limit, &mut summaries);
    if summaries.is_empty() {
        vec!["_".to_string()]
    } else {
        summaries
    }
}

fn collect_case_summaries(space: &MatchSpace, limit: usize, out: &mut Vec<String>) {
    if out.len() >= limit || space_is_empty(space) {
        return;
    }

    match space {
        MatchSpace::Empty => {}
        MatchSpace::Union(items) => {
            for item in items {
                collect_case_summaries(item, limit, out);
                if out.len() >= limit {
                    break;
                }
            }
        }
        MatchSpace::Primitive(primitive) => match primitive {
            PrimitiveSpace::Bool {
                allow_true,
                allow_false,
            } => {
                if *allow_true && out.len() < limit {
                    push_unique(out, "true".to_string());
                }
                if *allow_false && out.len() < limit {
                    push_unique(out, "false".to_string());
                }
            }
            PrimitiveSpace::Unit { present } => {
                if *present {
                    push_unique(out, "()".to_string());
                }
            }
            PrimitiveSpace::Int(ranges) => {
                if let Some(values) = int_range_singletons(ranges, limit.saturating_sub(out.len()))
                {
                    for value in values {
                        push_unique(out, value.to_string());
                        if out.len() >= limit {
                            break;
                        }
                    }
                } else {
                    push_unique(out, "_".to_string());
                }
            }
            PrimitiveSpace::EqFinite { values, .. } => {
                for value in values.iter().take(limit.saturating_sub(out.len())) {
                    push_unique(out, render_literal_atom(value));
                    if out.len() >= limit {
                        break;
                    }
                }
            }
            PrimitiveSpace::EqAny { .. } => push_unique(out, "_".to_string()),
        },
        MatchSpace::Variant(variant) => {
            let fields = variant
                .fields
                .iter()
                .map(case_summary_atom)
                .collect::<Vec<_>>();
            let summary = if fields.is_empty() {
                format!("{}()", variant.name)
            } else {
                format!("{}({})", variant.name, fields.join(","))
            };
            push_unique(out, summary);
        }
        MatchSpace::Record(record) => {
            let summary = format!(
                "{{{}}}",
                record
                    .fields
                    .iter()
                    .map(|(name, field)| format!("{}:{}", name, case_summary_atom(field)))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            push_unique(out, summary);
        }
        MatchSpace::Tuple(items) => {
            let product = tuple_summary_product(items, limit.saturating_sub(out.len()));
            if let Some(summaries) = product {
                for summary in summaries {
                    push_unique(out, summary);
                    if out.len() >= limit {
                        break;
                    }
                }
            } else {
                push_unique(out, "_".to_string());
            }
        }
        MatchSpace::List(ListSpace::Nil) => push_unique(out, "[]".to_string()),
        MatchSpace::List(ListSpace::Any(_)) => {
            push_unique(out, "[]".to_string());
            if out.len() < limit {
                push_unique(out, "[_,.rest]".to_string());
            }
        }
        MatchSpace::List(ListSpace::Cons { .. }) => {
            push_unique(out, list_case_summary(space));
        }
        MatchSpace::Opaque(_) => push_unique(out, "_".to_string()),
    }
}

fn push_unique(out: &mut Vec<String>, value: String) {
    if !out.contains(&value) {
        out.push(value);
    }
}

fn int_range_singletons(ranges: &IntRangeSet, limit: usize) -> Option<Vec<i64>> {
    let mut values = Vec::new();
    for interval in &ranges.intervals {
        let (Some(start), Some(end)) = (interval.start, interval.end) else {
            return None;
        };
        if start > end {
            continue;
        }
        let width = end.saturating_sub(start) as usize;
        if width > limit.saturating_sub(values.len()) {
            return None;
        }
        for value in start..=end {
            values.push(value);
            if values.len() > limit {
                return None;
            }
        }
    }
    Some(values)
}

fn tuple_summary_product(items: &[MatchSpace], limit: usize) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    for item in items {
        let item_summaries = space_to_case_summaries(item, limit);
        if item_summaries.len() == 1 && item_summaries[0] == "_" {
            return None;
        }
        parts.push(item_summaries);
    }

    let mut results = vec![String::new()];
    for item in parts {
        let mut next = Vec::new();
        for prefix in &results {
            for summary in &item {
                let combined = if prefix.is_empty() {
                    summary.clone()
                } else {
                    format!("{},{}", prefix, summary)
                };
                next.push(combined);
                if next.len() > limit {
                    return None;
                }
            }
        }
        results = next;
    }

    Some(
        results
            .into_iter()
            .map(|summary| format!("({})", summary))
            .collect(),
    )
}

fn list_case_summary(space: &MatchSpace) -> String {
    let mut elements = Vec::new();
    let mut current = space;
    loop {
        match current {
            MatchSpace::List(ListSpace::Cons { head, tail }) => {
                elements.push(case_summary_atom(head));
                current = tail;
            }
            MatchSpace::List(ListSpace::Nil) => return format!("[{}]", elements.join(",")),
            MatchSpace::List(ListSpace::Any(_)) => {
                if elements.is_empty() {
                    return "[_,.rest]".to_string();
                }
                return format!("[{},.rest]", elements.join(","));
            }
            _ => return "_".to_string(),
        }
    }
}

fn case_summary_atom(space: &MatchSpace) -> String {
    let summaries = space_to_case_summaries(space, 2);
    if summaries.len() == 1 {
        summaries[0].clone()
    } else {
        "_".to_string()
    }
}

fn render_literal_atom(atom: &LiteralAtom) -> String {
    match atom {
        LiteralAtom::Float(bits) => format!("{}", f64::from_bits(*bits)),
        LiteralAtom::String(value) => format!("{:?}", value),
        LiteralAtom::Char(value) => format!("'{}'", value),
    }
}

fn pattern_summary(pattern: &sigil_ast::Pattern) -> String {
    use sigil_ast::{Pattern, PatternLiteralType, PatternLiteralValue};

    match pattern {
        Pattern::Wildcard(_) => "_".to_string(),
        Pattern::Identifier(identifier) => identifier.name.clone(),
        Pattern::Literal(literal) => match (&literal.literal_type, &literal.value) {
            (PatternLiteralType::Bool, PatternLiteralValue::Bool(value)) => value.to_string(),
            (PatternLiteralType::Unit, _) => "()".to_string(),
            (PatternLiteralType::Int, PatternLiteralValue::Int(value)) => value.to_string(),
            (PatternLiteralType::Float, PatternLiteralValue::Float(value)) => format!("{}", value),
            (PatternLiteralType::String, PatternLiteralValue::String(value)) => {
                format!("{:?}", value)
            }
            (PatternLiteralType::Char, PatternLiteralValue::Char(value)) => format!("'{}'", value),
            _ => "_".to_string(),
        },
        Pattern::Tuple(tuple) => format!(
            "({})",
            tuple
                .patterns
                .iter()
                .map(pattern_summary)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Pattern::List(list) => {
            let mut parts = list
                .patterns
                .iter()
                .map(pattern_summary)
                .collect::<Vec<_>>();
            if list.rest.is_some() {
                parts.push(".rest".to_string());
            }
            format!("[{}]", parts.join(","))
        }
        Pattern::Constructor(constructor) => {
            let name = constructor_display_name(&constructor.module_path, &constructor.name);
            if constructor.patterns.is_empty() {
                format!("{}()", name)
            } else {
                format!(
                    "{}({})",
                    name,
                    constructor
                        .patterns
                        .iter()
                        .map(pattern_summary)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
        Pattern::Record(record) => format!(
            "{{{}}}",
            record
                .fields
                .iter()
                .map(|field| match &field.pattern {
                    Some(pattern) => format!("{}:{}", field.name, pattern_summary(pattern)),
                    None => field.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

pub(crate) fn expr_summary(expr: &Expr) -> String {
    match expr {
        Expr::Literal(literal) => match &literal.value {
            LiteralValue::Int(value) => value.to_string(),
            LiteralValue::Float(value) => format!("{}", value),
            LiteralValue::String(value) => format!("{:?}", value),
            LiteralValue::Char(value) => format!("'{}'", value),
            LiteralValue::Bool(value) => value.to_string(),
            LiteralValue::Unit => "()".to_string(),
        },
        Expr::Identifier(identifier) => identifier.name.clone(),
        Expr::Unary(unary) => format!("{}{}", unary.operator, expr_summary(&unary.operand)),
        Expr::Binary(binary) => format!(
            "{} {} {}",
            expr_summary(&binary.left),
            binary.operator,
            expr_summary(&binary.right)
        ),
        Expr::TypeAscription(type_asc) => expr_summary(&type_asc.expr),
        Expr::FieldAccess(field_access) => {
            format!(
                "{}.{}",
                expr_summary(&field_access.object),
                field_access.field
            )
        }
        Expr::Application(app) => format!("{}(...)", expr_summary(&app.func)),
        Expr::MemberAccess(access) => {
            format!("{}.{}", access.namespace.join("::"), access.member)
        }
        _ => "<expr>".to_string(),
    }
}
