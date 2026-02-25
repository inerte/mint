import type * as AST from '../parser/ast.js';
import type { SourceLocation as TokenSourceLocation, Token } from '../lexer/token.js';
import type { Diagnostic, Fixit, SourceSpan, Suggestion } from './types.js';

export function astLocationToSpan(file: string, location?: AST.SourceLocation): SourceSpan | undefined {
  if (!location) return undefined;
  return {
    file,
    start: { line: location.start.line, column: location.start.column, offset: location.start.offset },
    end: { line: location.end.line, column: location.end.column, offset: location.end.offset },
  };
}

export function tokenLocToSpan(file: string, start: TokenSourceLocation, end?: TokenSourceLocation): SourceSpan {
  return {
    file,
    start: { line: start.line, column: start.column, offset: start.offset },
    end: end ? { line: end.line, column: end.column, offset: end.offset } : undefined,
  };
}

export function tokenToSpan(file: string, token: Token): SourceSpan {
  return tokenLocToSpan(file, token.start, token.end);
}

export function replaceTokenFixit(file: string, token: Token, text: string): Fixit {
  return { kind: 'replace', range: tokenToSpan(file, token), text };
}

export function diagnostic(
  code: string,
  phase: Diagnostic['phase'],
  message: string,
  extras: Omit<Diagnostic, 'code' | 'phase' | 'message'> = {}
): Diagnostic {
  return { code, phase, message, ...extras };
}

export function suggestReplaceSymbol(
  message: string,
  replacement: string,
  target?: 'namespace_separator' | 'local_binding_keyword'
): Suggestion {
  return { kind: 'replace_symbol', message, replacement, target };
}

export function suggestExportMember(message: string, member?: string, targetFile?: string): Suggestion {
  return { kind: 'export_member', message, member, targetFile };
}

export function suggestUseOperator(message: string, operator: string, replaces?: string): Suggestion {
  return { kind: 'use_operator', message, operator, replaces };
}

export function suggestReorderDeclaration(
  message: string,
  category?: string,
  name?: string,
  before?: string
): Suggestion {
  return { kind: 'reorder_declaration', message, category, name, before };
}

export function suggestGeneric(message: string, action?: string): Suggestion {
  return { kind: 'generic', message, action };
}
