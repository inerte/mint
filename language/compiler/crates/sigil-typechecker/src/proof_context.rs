//! Proof context types for the bidirectional typechecker.
//!
//! Holds `ProofContext`, `SymbolicValue`, `ConstraintProofResult`, and related
//! pure arithmetic helpers shared across synthesis, checking, and symbolic lowering.

use crate::errors::TypeError;
use crate::types::InferenceType;
use sigil_ast::Expr;
use sigil_solver::{ComparisonOp, Formula, LinearExpr, SolverOutcome, SymbolPath};
use std::collections::{BTreeMap, HashMap, HashSet};

// ============================================================================
// Symbolic value representation
// ============================================================================

#[derive(Debug, Clone)]
pub(crate) enum SymbolicValue {
    Int(LinearExpr),
    Bool(Formula),
    Collection(SymbolicCollection),
    Record(SymbolicRecord),
    /// The protocol state of a handle — produced by `handle.state` field access.
    State { path: SymbolPath, protocol: String },
    /// An UpperCamelCase state name literal on the RHS of a state equality check.
    StateLabel(String),
}

#[derive(Debug, Clone)]
pub(crate) enum SymbolicCollection {
    Path(SymbolPath),
    KnownLength(LinearExpr),
}

impl SymbolicCollection {
    pub(crate) fn length_expr(&self) -> LinearExpr {
        match self {
            Self::Path(path) => LinearExpr::from_path(path.length()),
            Self::KnownLength(length) => length.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SymbolicRecord {
    Literal(BTreeMap<String, Expr>),
    Path {
        base: SymbolPath,
        fields: HashMap<String, InferenceType>,
    },
}

// ============================================================================
// Assumption collector (used during symbolic lowering)
// ============================================================================

#[derive(Default)]
pub(crate) struct AssumptionCollector {
    pub(crate) assumptions: Vec<Formula>,
    pub(crate) seen_bindings: HashSet<String>,
}

// ============================================================================
// Proof context
// ============================================================================

#[derive(Debug, Clone, Default)]
pub(crate) struct ProofContext {
    pub(crate) assumptions: Vec<Formula>,
    pub(crate) symbolic_bindings: HashMap<String, SymbolicValue>,
}

impl ProofContext {
    pub(crate) fn with_assumptions<I>(&self, assumptions: I) -> Self
    where
        I: IntoIterator<Item = Formula>,
    {
        let mut next = self.clone();
        next.assumptions.extend(assumptions);
        next
    }

    pub(crate) fn with_assumption(&self, assumption: Formula) -> Self {
        self.with_assumptions([assumption])
    }

    pub(crate) fn with_symbolic_bindings<I>(&self, bindings: I) -> Self
    where
        I: IntoIterator<Item = (String, SymbolicValue)>,
    {
        let mut next = self.clone();
        next.symbolic_bindings.extend(bindings);
        next
    }

    pub(crate) fn lookup_symbolic_binding(&self, name: &str) -> Option<SymbolicValue> {
        self.symbolic_bindings.get(name).cloned()
    }
}

pub(crate) const MATCH_SCRUTINEE_BINDING: &str = "$match_scrutinee";

// ============================================================================
// Proof result and outcome helpers
// ============================================================================

pub(crate) enum ConstraintProofResult {
    Proved,
    Failed(sigil_solver::ProofCheck),
}

impl ConstraintProofResult {
    pub(crate) fn proved(&self) -> bool {
        matches!(self, Self::Proved)
    }

    pub(crate) fn proved_trivially() -> Self {
        Self::Proved
    }

    pub(crate) fn failed_check(&self) -> Option<&sigil_solver::ProofCheck> {
        match self {
            Self::Proved => None,
            Self::Failed(check) => Some(check),
        }
    }
}

pub(crate) fn proof_outcome_reason(outcome: &SolverOutcome) -> String {
    match outcome {
        SolverOutcome::Proved => "proved".to_string(),
        SolverOutcome::Refuted { model } => {
            if model.is_empty() {
                "solver found a counterexample".to_string()
            } else {
                format!(
                    "solver found a counterexample: {}",
                    serde_json::to_string(model).unwrap_or_else(|_| "{}".to_string())
                )
            }
        }
        SolverOutcome::Unknown { reason } => format!("solver returned unknown: {}", reason),
    }
}

// ============================================================================
// Refinement error helper
// ============================================================================

pub(crate) fn refinement_type_support_error(name: &str, reason: &str) -> TypeError {
    TypeError::new(
        format!(
            "Type constraint for '{}' uses unsupported refinement syntax: {}. Supported refinement constraints use Bool/Int literals, value, #value, field access, +, -, comparisons, and/or/not.",
            name, reason
        ),
        None,
    )
}

// ============================================================================
// Pure arithmetic helpers (used by symbolic lowering and coverage)
// ============================================================================

pub(crate) fn solve_exact_single_var(coeff: i64, rhs: i64) -> Option<i64> {
    if coeff == 0 || rhs % coeff != 0 {
        return None;
    }
    Some(rhs / coeff)
}

pub(crate) fn solve_single_var_interval(
    coeff: i64,
    op: ComparisonOp,
    rhs: i64,
) -> Option<(Option<i64>, Option<i64>)> {
    if coeff == 0 {
        return None;
    }

    let (normalized_op, normalized_rhs) = if coeff > 0 {
        (op, rhs)
    } else {
        (
            match op {
                ComparisonOp::Lt => ComparisonOp::Gt,
                ComparisonOp::Le => ComparisonOp::Ge,
                ComparisonOp::Gt => ComparisonOp::Lt,
                ComparisonOp::Ge => ComparisonOp::Le,
                other => other,
            },
            -rhs,
        )
    };
    let divisor = coeff.abs();

    let interval = match normalized_op {
        ComparisonOp::Lt => (None, Some(div_floor(normalized_rhs - 1, divisor))),
        ComparisonOp::Le => (None, Some(div_floor(normalized_rhs, divisor))),
        ComparisonOp::Gt => (Some(div_ceil(normalized_rhs + 1, divisor)), None),
        ComparisonOp::Ge => (Some(div_ceil(normalized_rhs, divisor)), None),
        ComparisonOp::Eq | ComparisonOp::Ne => return None,
    };

    Some(interval)
}

pub(crate) fn div_floor(numerator: i64, denominator: i64) -> i64 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder != 0 && ((remainder > 0) != (denominator > 0)) {
        quotient - 1
    } else {
        quotient
    }
}

pub(crate) fn div_ceil(numerator: i64, denominator: i64) -> i64 {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder != 0 && ((remainder > 0) == (denominator > 0)) {
        quotient + 1
    } else {
        quotient
    }
}
