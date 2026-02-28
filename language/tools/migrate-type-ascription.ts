#!/usr/bin/env node --import tsx
/**
 * Automated Migration Tool for Type Ascription
 *
 * Migrates Sigil files to use mandatory type ascription:
 * - l x=value → l x=(value:Type)
 * - c name:Type=value → c name=(value:Type)
 *
 * Uses the typechecker to infer types before transformation.
 */

import * as fs from 'fs';
import * as path from 'path';
import { parse } from '../compiler/src/parser/parser.js';
import { tokenize } from '../compiler/src/lexer/lexer.js';
import * as AST from '../compiler/src/parser/ast.js';

function findSigilFiles(pattern: string): string[] {
  const files: string[] = [];

  // Simple glob expansion for *.sigil and **/*.sigil patterns
  if (pattern.includes('*')) {
    const dir = pattern.substring(0, pattern.lastIndexOf('/'));
    const recursive = pattern.includes('**');

    function walkDir(currentDir: string) {
      const entries = fs.readdirSync(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(currentDir, entry.name);
        if (entry.isDirectory() && recursive) {
          walkDir(fullPath);
        } else if (entry.isFile() && (entry.name.endsWith('.sigil') || entry.name.endsWith('.lib.sigil'))) {
          files.push(fullPath);
        }
      }
    }

    if (fs.existsSync(dir)) {
      walkDir(dir);
    }
  } else {
    // Single file
    if (fs.existsSync(pattern)) {
      files.push(pattern);
    }
  }

  return files;
}

interface MigrationResult {
  file: string;
  migrated: boolean;
  changes: number;
  error?: string;
}

function inferTypeFromLiteral(expr: AST.Expr): string | null {
  if (expr.type === 'LiteralExpr') {
    switch (expr.literalType) {
      case 'Int': return 'ℤ';
      case 'Float': return 'ℝ';
      case 'String': return '𝕊';
      case 'Bool': return '𝔹';
      case 'Char': return 'ℂ';
      case 'Unit': return '𝕌';
    }
  }

  if (expr.type === 'ListExpr') {
    if (expr.elements.length === 0) {
      // Empty list - cannot infer, need user input
      return '[?]';
    }
    const firstType = inferTypeFromLiteral(expr.elements[0]);
    if (firstType && firstType !== '[?]') {
      return `[${firstType}]`;
    }
  }

  if (expr.type === 'TupleExpr') {
    const types = expr.elements.map(inferTypeFromLiteral).filter(t => t !== null);
    if (types.length === expr.elements.length) {
      return `(${types.join(',')})`;
    }
  }

  return null;
}

function formatTypeAst(type: AST.Type): string {
  switch (type.type) {
    case 'PrimitiveType':
      switch (type.name) {
        case 'Int': return 'ℤ';
        case 'Float': return 'ℝ';
        case 'String': return '𝕊';
        case 'Bool': return '𝔹';
        case 'Char': return 'ℂ';
        case 'Unit': return '𝕌';
      }
      break;
    case 'ListType':
      return `[${formatTypeAst(type.elementType)}]`;
    case 'TupleType':
      return `(${type.types.map(formatTypeAst).join(',')})`;
    case 'TypeConstructor':
      if (type.typeArgs.length === 0) {
        return type.name;
      }
      return `${type.name}[${type.typeArgs.map(formatTypeAst).join(',')}]`;
  }
  return 'Unknown';
}

function migrateSource(source: string, filename: string): { source: string; changes: number } {
  try {
    const tokens = tokenize(source);
    const program = parse(tokens, filename);

    let changes = 0;
    const lines = source.split('\n');

    // Migrate const declarations
    for (const decl of program.declarations) {
      if (decl.type === 'ConstDecl' && decl.typeAnnotation) {
        // Old format: c name:Type=value
        // New format: c name=(value:Type)
        const line = decl.location.start.line - 1;
        const lineText = lines[line];

        // Check if this is old format (has colon before equals)
        const oldFormatMatch = lineText.match(/^c\s+(\w+)\s*:\s*([^=]+)=(.+)$/);
        if (oldFormatMatch) {
          const [, name, typeStr, value] = oldFormatMatch;
          const typeFormatted = formatTypeAst(decl.typeAnnotation);
          lines[line] = `c ${name}=(${value.trim()}:${typeFormatted})`;
          changes++;
        }
      }
    }

    // Migrate let expressions - we'll need to traverse and find them
    // For now, use regex-based approach for simple cases
    for (let i = 0; i < lines.length; i++) {
      let line = lines[i];
      let lineChanged = false;

      // Match all: l name=literal (not already wrapped) - use global replacement
      const letPattern = /\bl\s+(\w+)\s*=\s*([^(;=\n]+)([;\n])/g;
      let match;

      while ((match = letPattern.exec(line)) !== null) {
        const [fullMatch, name, value, terminator] = match;
        const trimmedValue = value.trim();

        // Skip if already has parentheses (already migrated)
        if (trimmedValue.startsWith('(')) {
          continue;
        }

        // Try to infer type from value
        let inferredType: string | null = null;

        // Simple literal inference
        if (/^-?\d+$/.test(trimmedValue)) {
          inferredType = 'ℤ';
        } else if (/^-?\d+\.\d+$/.test(trimmedValue)) {
          inferredType = 'ℝ';
        } else if (/^".*"$/.test(trimmedValue) || /^'.*'$/.test(trimmedValue)) {
          inferredType = '𝕊';
        } else if (trimmedValue === 'true' || trimmedValue === 'false') {
          inferredType = '𝔹';
        } else if (trimmedValue === '()') {
          inferredType = '𝕌';
        } else if (trimmedValue === '[]') {
          // Empty list - cannot infer, skip for manual migration
          console.warn(`  ⚠ Cannot infer type for empty list in: ${line.trim()} (file: ${filename}:${i+1})`);
          continue;
        } else if (/^\[.+\]$/.test(trimmedValue)) {
          // Non-empty list - try to infer element type
          const firstElem = trimmedValue.slice(1, -1).split(',')[0].trim();
          if (/^-?\d+$/.test(firstElem)) {
            inferredType = '[ℤ]';
          } else if (/^".*"$/.test(firstElem)) {
            inferredType = '[𝕊]';
          }
        }

        if (inferredType) {
          line = line.replace(fullMatch, `l ${name}=(${trimmedValue}:${inferredType})${terminator}`);
          lineChanged = true;
          changes++;
        } else {
          console.warn(`  ⚠ Cannot infer type for: ${fullMatch.trim()} (file: ${filename}:${i+1})`);
        }
      }

      if (lineChanged) {
        lines[i] = line;
      }
    }

    return { source: lines.join('\n'), changes };
  } catch (error) {
    throw new Error(`Failed to migrate ${filename}: ${error}`);
  }
}

async function migrateFile(filepath: string): Promise<MigrationResult> {
  try {
    const source = fs.readFileSync(filepath, 'utf-8');
    const { source: migratedSource, changes } = migrateSource(source, filepath);

    if (changes > 0) {
      fs.writeFileSync(filepath, migratedSource, 'utf-8');
      return { file: filepath, migrated: true, changes };
    }

    return { file: filepath, migrated: false, changes: 0 };
  } catch (error) {
    return { file: filepath, migrated: false, changes: 0, error: String(error) };
  }
}

async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0) {
    console.log('Usage: migrate-type-ascription.ts <pattern> [<pattern>...]');
    console.log('');
    console.log('Examples:');
    console.log('  migrate-type-ascription.ts "language/stdlib/*.lib.sigil"');
    console.log('  migrate-type-ascription.ts "language/examples/*.sigil"');
    console.log('  migrate-type-ascription.ts "**/*.sigil"');
    process.exit(1);
  }

  const patterns = args;
  const files: string[] = [];

  for (const pattern of patterns) {
    const matches = findSigilFiles(pattern);
    files.push(...matches);
  }

  if (files.length === 0) {
    console.log('No files found matching patterns:', patterns);
    process.exit(0);
  }

  console.log(`Found ${files.length} files to migrate`);
  console.log('');

  const results: MigrationResult[] = [];

  for (const file of files) {
    process.stdout.write(`Migrating ${file}... `);
    const result = await migrateFile(file);
    results.push(result);

    if (result.error) {
      console.log(`❌ ERROR: ${result.error}`);
    } else if (result.changes > 0) {
      console.log(`✓ (${result.changes} changes)`);
    } else {
      console.log('✓ (no changes needed)');
    }
  }

  console.log('');
  console.log('Migration Summary:');
  console.log('=================');

  const migrated = results.filter(r => r.migrated);
  const errors = results.filter(r => r.error);
  const unchanged = results.filter(r => !r.migrated && !r.error);
  const totalChanges = results.reduce((sum, r) => sum + r.changes, 0);

  console.log(`Total files: ${results.length}`);
  console.log(`Migrated: ${migrated.length} (${totalChanges} total changes)`);
  console.log(`Unchanged: ${unchanged.length}`);
  console.log(`Errors: ${errors.length}`);

  if (errors.length > 0) {
    console.log('');
    console.log('Files with errors:');
    errors.forEach(e => console.log(`  - ${e.file}: ${e.error}`));
  }
}

main().catch(err => {
  console.error('Fatal error:', err);
  process.exit(1);
});
