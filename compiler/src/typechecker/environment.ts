/**
 * Mint Type Checker - Type Environment (Bidirectional)
 *
 * Manages variable bindings during type checking.
 * Simplified from HM version - no type schemes, direct InferenceType bindings.
 */

import { InferenceType } from './types.js';

/**
 * Type environment (Γ in type theory notation)
 *
 * Maps variable names to their types
 * Supports nested scopes via parent chaining
 */
export class TypeEnvironment {
  private bindings: Map<string, InferenceType>;
  private parent?: TypeEnvironment;

  constructor(parent?: TypeEnvironment) {
    this.bindings = new Map();
    this.parent = parent;
  }

  /**
   * Look up a variable's type
   *
   * Searches this environment and all parent environments
   */
  lookup(name: string): InferenceType | undefined {
    const local = this.bindings.get(name);
    if (local) {
      return local;
    }

    // Search parent scope
    return this.parent?.lookup(name);
  }

  /**
   * Bind a variable to a type
   *
   * Only affects the current scope
   */
  bind(name: string, type: InferenceType): void {
    this.bindings.set(name, type);
  }

  /**
   * Create a child environment with additional bindings
   *
   * Example: when entering a lambda or match arm with pattern bindings
   */
  extend(newBindings?: Map<string, InferenceType>): TypeEnvironment {
    const child = new TypeEnvironment(this);
    if (newBindings) {
      for (const [name, type] of newBindings) {
        child.bind(name, type);
      }
    }
    return child;
  }

  /**
   * Get all bindings in this scope (for debugging/testing)
   */
  getBindings(): Map<string, InferenceType> {
    return new Map(this.bindings);
  }

  /**
   * Create the initial environment with built-in operators
   */
  static createInitialEnvironment(): TypeEnvironment {
    const env = new TypeEnvironment();

    // Built-in operators are handled directly in synthesizeBinary/synthesizeUnary
    // This environment is primarily for user-defined functions and constants

    return env;
  }
}
