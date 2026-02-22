/**
 * Mint Type Checker - Unification Algorithm
 *
 * Implements Robinson's unification algorithm with occurs check
 * This is the core of Hindley-Milner type inference
 */

import * as AST from '../parser/ast.js';
import {
  InferenceType,
  Substitution,
  TVar,
  applySubst,
  composeSubstitutions
} from './types.js';
import { TypeError, typeMismatchError, occursCheckError } from './errors.js';

/**
 * Unify two types, returning a substitution that makes them equal
 *
 * Algorithm:
 * - unify(Int, Int) = ∅
 * - unify(α, Int) = [α ↦ Int]
 * - unify(α, β) = [α ↦ β] (assuming α ≠ β)
 * - unify(Int → Int, α → β) = [α ↦ Int, β ↦ Int]
 * - unify(Int, String) = ERROR
 *
 * @param t1 First type
 * @param t2 Second type
 * @param location Source location for error reporting
 * @returns Substitution that makes t1 and t2 equal
 * @throws TypeError if types cannot be unified
 */
export function unify(
  t1: InferenceType,
  t2: InferenceType,
  location?: AST.SourceLocation
): Substitution {
  // Dereference type variables (follow instance chains)
  t1 = prune(t1);
  t2 = prune(t2);

  // ========================================
  // BOTH ARE TYPE VARIABLES
  // ========================================

  if (t1.kind === 'var' && t2.kind === 'var') {
    if (t1.id === t2.id) {
      // Same variable - no substitution needed
      return new Map();
    }

    // Different variables - bind t1 to t2
    t1.instance = t2;
    return new Map([[t1.id, t2]]);
  }

  // ========================================
  // ONE IS A TYPE VARIABLE
  // ========================================

  if (t1.kind === 'var') {
    return unifyVariable(t1, t2, location);
  }

  if (t2.kind === 'var') {
    return unifyVariable(t2, t1, location);
  }

  // ========================================
  // BOTH ARE PRIMITIVES
  // ========================================

  if (t1.kind === 'primitive' && t2.kind === 'primitive') {
    if (t1.name === t2.name) {
      return new Map();
    }

    throw typeMismatchError(
      `Cannot unify primitive types`,
      t1,
      t2,
      location
    );
  }

  // ========================================
  // BOTH ARE FUNCTIONS
  // ========================================

  if (t1.kind === 'function' && t2.kind === 'function') {
    // Check arity
    if (t1.params.length !== t2.params.length) {
      throw new TypeError(
        `Function arity mismatch: ${t1.params.length} vs ${t2.params.length} parameters`,
        location,
        t1,
        t2
      );
    }

    let subst = new Map<number, InferenceType>();

    // Unify all parameters (left to right)
    for (let i = 0; i < t1.params.length; i++) {
      const param1 = applySubst(subst, t1.params[i]);
      const param2 = applySubst(subst, t2.params[i]);
      const s = unify(param1, param2, location);
      subst = composeSubstitutions(subst, s);
    }

    // Unify return types
    const ret1 = applySubst(subst, t1.returnType);
    const ret2 = applySubst(subst, t2.returnType);
    const sRet = unify(ret1, ret2, location);

    return composeSubstitutions(subst, sRet);
  }

  // ========================================
  // BOTH ARE LISTS
  // ========================================

  if (t1.kind === 'list' && t2.kind === 'list') {
    return unify(t1.elementType, t2.elementType, location);
  }

  // ========================================
  // BOTH ARE TUPLES
  // ========================================

  if (t1.kind === 'tuple' && t2.kind === 'tuple') {
    // Check length
    if (t1.types.length !== t2.types.length) {
      throw new TypeError(
        `Tuple length mismatch: ${t1.types.length} vs ${t2.types.length}`,
        location,
        t1,
        t2
      );
    }

    let subst = new Map<number, InferenceType>();

    // Unify all elements
    for (let i = 0; i < t1.types.length; i++) {
      const elem1 = applySubst(subst, t1.types[i]);
      const elem2 = applySubst(subst, t2.types[i]);
      const s = unify(elem1, elem2, location);
      subst = composeSubstitutions(subst, s);
    }

    return subst;
  }

  // ========================================
  // BOTH ARE RECORDS
  // ========================================

  if (t1.kind === 'record' && t2.kind === 'record') {
    // Check if they have the same fields
    if (t1.fields.size !== t2.fields.size) {
      throw typeMismatchError(
        'Record field count mismatch',
        t1,
        t2,
        location
      );
    }

    let subst = new Map<number, InferenceType>();

    // Unify all fields
    for (const [fieldName, fieldType1] of t1.fields) {
      const fieldType2 = t2.fields.get(fieldName);

      if (!fieldType2) {
        throw new TypeError(
          `Record field mismatch: field '${fieldName}' not found in second type`,
          location,
          t1,
          t2
        );
      }

      const ft1 = applySubst(subst, fieldType1);
      const ft2 = applySubst(subst, fieldType2);
      const s = unify(ft1, ft2, location);
      subst = composeSubstitutions(subst, s);
    }

    return subst;
  }

  // ========================================
  // BOTH ARE TYPE CONSTRUCTORS
  // ========================================

  if (t1.kind === 'constructor' && t2.kind === 'constructor') {
    // Check constructor names match
    if (t1.name !== t2.name) {
      throw typeMismatchError(
        `Type constructor mismatch: ${t1.name} vs ${t2.name}`,
        t1,
        t2,
        location
      );
    }

    // Check arity
    if (t1.typeArgs.length !== t2.typeArgs.length) {
      throw new TypeError(
        `Type constructor arity mismatch: ${t1.typeArgs.length} vs ${t2.typeArgs.length} type arguments`,
        location,
        t1,
        t2
      );
    }

    let subst = new Map<number, InferenceType>();

    // Unify all type arguments
    for (let i = 0; i < t1.typeArgs.length; i++) {
      const arg1 = applySubst(subst, t1.typeArgs[i]);
      const arg2 = applySubst(subst, t2.typeArgs[i]);
      const s = unify(arg1, arg2, location);
      subst = composeSubstitutions(subst, s);
    }

    return subst;
  }

  // ========================================
  // TYPE KIND MISMATCH
  // ========================================

  throw typeMismatchError(
    `Cannot unify types of different kinds: ${t1.kind} vs ${t2.kind}`,
    t1,
    t2,
    location
  );
}

/**
 * Unify a type variable with a type
 *
 * Performs occurs check to prevent infinite types
 *
 * @param v Type variable
 * @param t Type to unify with
 * @param location Source location for error reporting
 */
function unifyVariable(
  v: TVar,
  t: InferenceType,
  location?: AST.SourceLocation
): Substitution {
  // Occurs check: prevent infinite types
  // Example: unifying α with [α] would create infinite type [[[...]]]
  if (occursIn(v.id, t)) {
    throw occursCheckError(v, t, location);
  }

  // Bind the variable to the type
  v.instance = t;
  return new Map([[v.id, t]]);
}

/**
 * Occurs check: does variable v occur in type t?
 *
 * This prevents creating infinite types like α = α → β
 *
 * @param varId Type variable ID to check
 * @param type Type to search in
 * @returns true if varId occurs in type
 */
function occursIn(varId: number, type: InferenceType): boolean {
  // Follow instances
  type = prune(type);

  switch (type.kind) {
    case 'primitive':
      return false;

    case 'var':
      return type.id === varId;

    case 'function':
      return (
        type.params.some(p => occursIn(varId, p)) ||
        occursIn(varId, type.returnType)
      );

    case 'list':
      return occursIn(varId, type.elementType);

    case 'tuple':
      return type.types.some(t => occursIn(varId, t));

    case 'record':
      for (const fieldType of type.fields.values()) {
        if (occursIn(varId, fieldType)) {
          return true;
        }
      }
      return false;

    case 'constructor':
      return type.typeArgs.some(arg => occursIn(varId, arg));
  }
}

/**
 * Prune: follow type variable instances to get the actual type
 *
 * This is critical for unification - we need to dereference variables
 * before comparing them
 *
 * Side effect: Updates instance pointers for path compression
 */
export function prune(type: InferenceType): InferenceType {
  if (type.kind === 'var' && type.instance) {
    // Path compression: update instance pointer directly to final type
    type.instance = prune(type.instance);
    return type.instance;
  }
  return type;
}

/**
 * Check if two types can be unified (without actually unifying them)
 *
 * Useful for testing and checking compatibility
 */
export function canUnify(t1: InferenceType, t2: InferenceType): boolean {
  try {
    unify(t1, t2);
    return true;
  } catch (e) {
    if (e instanceof TypeError) {
      return false;
    }
    throw e;
  }
}
