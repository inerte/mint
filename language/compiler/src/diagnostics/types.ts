export type SigilPhase =
  | 'cli'
  | 'io'
  | 'surface'
  | 'lexer'
  | 'parser'
  | 'canonical'
  | 'typecheck'
  | 'mutability'
  | 'extern'
  | 'codegen'
  | 'runtime';

export type SourcePoint = {
  line: number;
  column: number;
  offset?: number;
};

export type SourceSpan = {
  file: string;
  start: SourcePoint;
  end?: SourcePoint;
};

export type Fixit = {
  kind: 'replace' | 'insert' | 'delete';
  range: SourceSpan;
  text?: string;
};

export type Suggestion =
  | {
      kind: 'replace_symbol';
      message: string;
      replacement: string;
      target?: 'namespace_separator' | 'local_binding_keyword';
    }
  | {
      kind: 'export_member';
      message: string;
      targetFile?: string;
      member?: string;
    }
  | {
      kind: 'use_operator';
      message: string;
      operator: string;
      replaces?: string;
    }
  | {
      kind: 'reorder_declaration';
      message: string;
      category?: string;
      name?: string;
      before?: string;
    }
  | {
      kind: 'generic';
      message: string;
      action?: string;
    };

export type Diagnostic = {
  code: string;
  phase: SigilPhase;
  message: string;
  location?: SourceSpan;
  found?: unknown;
  expected?: unknown;
  details?: Record<string, unknown>;
  fixits?: Fixit[];
  suggestions?: Suggestion[];
};

export type CommandEnvelope<TData = unknown> = {
  formatVersion: 1;
  command: string;
  ok: boolean;
  phase?: SigilPhase;
  data?: TData;
  error?: Diagnostic;
};
