/**
 * Mint Type Checker - Pattern Type Checking
 *
 * Handles type inference for pattern matching constructs
 */

import * as AST from '../parser/ast.js';
import {
  InferenceType,
  TPrimitive,
  Substitution,
  applySubst,
  composeSubstitutions
} from './types.js';
import { TypeInferenceEngine } from './inference.js';
import { TypeError } from './errors.js';

/**
 * Pattern inference result
 * Returns: [substitution, patternType, bindings]
 */
export type PatternResult = [
  Substitution,
  InferenceType,
  Map<string, InferenceType>
];

/**
 * Infer the type of a pattern
 *
 * Returns:
 * - Substitution: type variable bindings from unification
 * - InferenceType: the type that this pattern matches
 * - Map<string, InferenceType>: variables bound by this pattern
 */
export function inferPattern(
  engine: TypeInferenceEngine,
  pattern: AST.Pattern
): PatternResult {
  switch (pattern.type) {
    case 'LiteralPattern':
      return inferLiteralPattern(pattern);

    case 'IdentifierPattern':
      return inferIdentifierPattern(engine, pattern);

    case 'WildcardPattern':
      return inferWildcardPattern(engine);

    case 'ListPattern':
      return inferListPattern(engine, pattern);

    case 'TuplePattern':
      return inferTuplePattern(engine, pattern);

    case 'ConstructorPattern':
      return inferConstructorPattern(engine, pattern);

    case 'RecordPattern':
      return inferRecordPattern(engine, pattern);

    default:
      throw new Error(`Unknown pattern type: ${(pattern as any).type}`);
  }
}

/**
 * Infer type of a literal pattern
 *
 * Example: 0 → ℤ, true → 𝔹, "hello" → 𝕊
 */
function inferLiteralPattern(pattern: AST.LiteralPattern): PatternResult {
  let type: TPrimitive;

  switch (pattern.literalType) {
    case 'Int':
      type = { kind: 'primitive', name: 'Int' };
      break;

    case 'Float':
      type = { kind: 'primitive', name: 'Float' };
      break;

    case 'Bool':
      type = { kind: 'primitive', name: 'Bool' };
      break;

    case 'String':
      type = { kind: 'primitive', name: 'String' };
      break;

    case 'Char':
      type = { kind: 'primitive', name: 'Char' };
      break;

    case 'Unit':
      type = { kind: 'primitive', name: 'Unit' };
      break;

    default:
      throw new Error(`Unknown literal type: ${(pattern as any).literalType}`);
  }

  // Literal patterns don't bind any variables
  return [new Map(), type, new Map()];
}

/**
 * Infer type of an identifier pattern
 *
 * Example: x → α (fresh type variable), binds x:α
 */
function inferIdentifierPattern(
  engine: TypeInferenceEngine,
  pattern: AST.IdentifierPattern
): PatternResult {
  // Create fresh type variable for this identifier
  const type = engine.freshVar(pattern.name);

  // Bind the identifier to this type
  const bindings = new Map<string, InferenceType>();
  bindings.set(pattern.name, type);

  return [new Map(), type, bindings];
}

/**
 * Infer type of a wildcard pattern
 *
 * Example: _ → α (fresh type variable), binds nothing
 */
function inferWildcardPattern(engine: TypeInferenceEngine): PatternResult {
  // Wildcard matches any type (fresh type variable)
  const type = engine.freshVar();

  // Wildcard doesn't bind any variables
  return [new Map(), type, new Map()];
}

/**
 * Infer type of a list pattern
 *
 * Examples:
 * - [] → [α] (empty list)
 * - [x] → [α], binds x:α
 * - [x,.xs] → [α], binds x:α, xs:[α]
 * - [x,y,.rest] → [α], binds x:α, y:α, rest:[α]
 */
function inferListPattern(
  engine: TypeInferenceEngine,
  pattern: AST.ListPattern
): PatternResult {
  // Empty list pattern
  if (pattern.patterns.length === 0 && !pattern.rest) {
    const elemType = engine.freshVar();
    return [new Map(), { kind: 'list', elementType: elemType }, new Map()];
  }

  let subst: Substitution = new Map();
  const bindings = new Map<string, InferenceType>();

  // Create fresh type variable for list element type
  const elemType = engine.freshVar();

  // Infer types for each element pattern
  for (const elemPattern of pattern.patterns) {
    const [elemSubst, _elemPatternType, elemBindings] = inferPattern(engine, elemPattern);

    // Compose substitutions
    subst = composeSubstitutions(subst, elemSubst);

    // Element pattern type must unify with element type
    // This is simplified - proper implementation would use unify from unification.ts
    // For now, we'll just add to bindings
    for (const [name, type] of elemBindings) {
      bindings.set(name, applySubst(subst, type));
    }
  }

  // Handle rest parameter (e.g., .xs in [x,.xs])
  if (pattern.rest) {
    // Rest has type [T] where T is the element type
    const restType: InferenceType = {
      kind: 'list',
      elementType: applySubst(subst, elemType)
    };

    bindings.set(pattern.rest, restType);
  }

  // Pattern type is [T]
  const patternType: InferenceType = {
    kind: 'list',
    elementType: applySubst(subst, elemType)
  };

  return [subst, patternType, bindings];
}

/**
 * Infer type of a tuple pattern
 *
 * Example: (x, y, z) → (α, β, γ), binds x:α, y:β, z:γ
 */
function inferTuplePattern(
  engine: TypeInferenceEngine,
  pattern: AST.TuplePattern
): PatternResult {
  let subst: Substitution = new Map();
  const bindings = new Map<string, InferenceType>();
  const types: InferenceType[] = [];

  // Infer type for each element pattern
  for (const elemPattern of pattern.patterns) {
    const [elemSubst, elemType, elemBindings] = inferPattern(engine, elemPattern);

    // Compose substitutions
    subst = composeSubstitutions(subst, elemSubst);

    // Apply substitution to element type
    types.push(applySubst(subst, elemType));

    // Merge bindings
    for (const [name, type] of elemBindings) {
      bindings.set(name, applySubst(subst, type));
    }
  }

  // Pattern type is tuple of all element types
  const patternType: InferenceType = {
    kind: 'tuple',
    types
  };

  return [subst, patternType, bindings];
}

/**
 * Infer type of a constructor pattern
 *
 * Example: Some(x) → Option[α], binds x:α
 *          None → Option[α]
 */
function inferConstructorPattern(
  engine: TypeInferenceEngine,
  pattern: AST.ConstructorPattern
): PatternResult {
  let subst: Substitution = new Map();
  const bindings = new Map<string, InferenceType>();
  const argTypes: InferenceType[] = [];

  // Infer types for constructor arguments
  for (const argPattern of pattern.patterns) {
    const [argSubst, argType, argBindings] = inferPattern(engine, argPattern);

    // Compose substitutions
    subst = composeSubstitutions(subst, argSubst);

    // Apply substitution to argument type
    argTypes.push(applySubst(subst, argType));

    // Merge bindings
    for (const [name, type] of argBindings) {
      bindings.set(name, applySubst(subst, type));
    }
  }

  // For now, create a generic constructor type
  // Proper implementation would look up constructor in environment
  const typeArgs = argTypes.length > 0 ? argTypes : [engine.freshVar()];
  const patternType: InferenceType = {
    kind: 'constructor',
    name: pattern.name,
    typeArgs
  };

  return [subst, patternType, bindings];
}

/**
 * Infer type of a record pattern
 *
 * Example: {x, y: z} → {x: α, y: β}, binds x:α, z:β
 */
function inferRecordPattern(
  engine: TypeInferenceEngine,
  pattern: AST.RecordPattern
): PatternResult {
  let subst: Substitution = new Map();
  const bindings = new Map<string, InferenceType>();
  const fields = new Map<string, InferenceType>();

  // Infer types for each field pattern
  for (const field of pattern.fields) {
    // If field.pattern is null, it means shorthand: {x} means {x: x}
    const fieldPattern = field.pattern || {
      type: 'IdentifierPattern' as const,
      name: field.name,
      location: field.location
    };

    const [fieldSubst, fieldType, fieldBindings] = inferPattern(
      engine,
      fieldPattern
    );

    // Compose substitutions
    subst = composeSubstitutions(subst, fieldSubst);

    // Apply substitution to field type
    fields.set(field.name, applySubst(subst, fieldType));

    // Merge bindings
    for (const [name, type] of fieldBindings) {
      bindings.set(name, applySubst(subst, type));
    }
  }

  // Pattern type is record with all field types
  const patternType: InferenceType = {
    kind: 'record',
    fields
  };

  return [subst, patternType, bindings];
}

/**
 * Check if patterns are exhaustive (basic implementation)
 *
 * Full exhaustiveness checking is complex and uses pattern matrix algorithms.
 * This is a simplified version that catches obvious missing cases.
 */
export function checkExhaustiveness(
  scrutineeType: InferenceType,
  patterns: AST.Pattern[],
  location: AST.SourceLocation
): void {
  // Check if there's a catch-all pattern (wildcard or identifier)
  const hasCatchAll = patterns.some(
    p => p.type === 'WildcardPattern' || p.type === 'IdentifierPattern'
  );

  if (hasCatchAll) {
    // Catch-all patterns make the match exhaustive
    return;
  }

  // Check for obvious missing cases based on scrutinee type
  const missing: string[] = [];

  if (scrutineeType.kind === 'list') {
    // For list types, check for [] and non-empty patterns
    const hasEmpty = patterns.some(
      p => p.type === 'ListPattern' && p.patterns.length === 0 && !p.rest
    );

    const hasNonEmpty = patterns.some(
      p =>
        p.type === 'ListPattern' &&
        (p.patterns.length > 0 || p.rest !== undefined)
    );

    if (!hasEmpty) {
      missing.push('[]');
    }

    if (!hasNonEmpty) {
      missing.push('[_,...]');
    }
  }

  if (scrutineeType.kind === 'primitive' && scrutineeType.name === 'Bool') {
    // For boolean types, check for true and false
    const literalValues = patterns
      .filter(p => p.type === 'LiteralPattern')
      .map(p => (p as AST.LiteralPattern).value);

    if (!literalValues.includes(true)) {
      missing.push('⊤');
    }

    if (!literalValues.includes(false)) {
      missing.push('⊥');
    }
  }

  // If we found missing patterns, throw error
  if (missing.length > 0) {
    throw new TypeError(
      `Non-exhaustive pattern match.\n\nMissing cases:\n${missing.map(p => `  - ${p}`).join('\n')}`,
      location
    );
  }

  // For other types (tuples, constructors, etc.), we'd need more sophisticated checking
  // For now, accept them if they have a catch-all or we can't determine exhaustiveness
}
