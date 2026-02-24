/**
 * Mint Semantic Map Writer
 *
 * Writes semantic maps to .sigil.map files
 */

import { writeFileSync } from 'fs';
import { SemanticMap } from './types.js';

/**
 * Write semantic map to file
 */
export function writeSemanticMap(map: SemanticMap, outputFile: string): void {
  const json = JSON.stringify(map, null, 2);
  writeFileSync(outputFile, json + '\n', 'utf-8');
}
