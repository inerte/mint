/**
 * Mint Type Checker - Public API
 *
 * Main entry point for type checking Mint programs
 */

import * as AST from '../parser/ast.js';
import { TypeInferenceEngine } from './inference.js';
import { TypeError } from './errors.js';
import { TypeScheme } from './types.js';

// Re-export types
export { TypeError } from './errors.js';
export type { TypeScheme, InferenceType } from './types.js';

/**
 * Type check a Mint program
 *
 * Returns a map of function names to their inferred type schemes
 * Throws TypeError if type checking fails
 */
export function typeCheck(
  program: AST.Program,
  sourceCode?: string
): Map<string, TypeScheme> {
  try {
    const engine = new TypeInferenceEngine();
    return engine.inferProgram(program);
  } catch (error) {
    if (error instanceof TypeError && sourceCode) {
      // Format error with source context
      console.error(error.format(sourceCode));
    }
    throw error;
  }
}
