export type BenchmarkCommand = {
  command: string;
  timeoutMs?: number;
};

export type TaskBudgets = {
  maxTurns: number;
  maxWallClockMs: number;
};

export type TaskManifest = {
  id: string;
  title: string;
  goal: string;
  initialPrompt: string;
  fixture: string;
  setupCommands: BenchmarkCommand[];
  oracleCommands: BenchmarkCommand[];
  successCriteria: string[];
  allowedEditPaths: string[];
  forbiddenEditPaths: string[];
  budgets: TaskBudgets;
  rootCauseTags: string[];
};

export type AgentFinalResponse = {
  summary: string;
  diagnosis: string;
  diagnosisTags: string[];
  filesChanged: string[];
};

export type ExecutorUsage = {
  inputTokens?: number;
  cachedInputTokens?: number;
  outputTokens?: number;
};

export type ExecutionArtifact = {
  events: string[];
  rawStdout: string;
  rawStderr: string;
};

export type ExecutorResult = {
  exitCode: number;
  finalResponse: AgentFinalResponse | null;
  usage: ExecutorUsage | null;
  toolCounts: Record<string, number>;
  artifact: ExecutionArtifact;
  errorMessage?: string;
};

export type ExecutorRunContext = {
  task: TaskManifest;
  workspacePath: string;
  runLabel: string;
  prompt: string;
  env: Record<string, string>;
  timeoutMs: number;
};

export interface Executor {
  readonly kind: string;
  run(context: ExecutorRunContext): Promise<ExecutorResult>;
}

export type ShellCommandResult = {
  command: string;
  cwd: string;
  stdout: string;
  stderr: string;
  exitCode: number;
  durationMs: number;
};

export type PathPolicyResult = {
  allowed: boolean;
  forbiddenMatches: string[];
  outOfBoundsMatches: string[];
};

export type PatchStats = {
  additions: number;
  deletions: number;
  filesChanged: number;
};

export type TaskRunResult = {
  taskId: string;
  refLabel: string;
  ref: string;
  status: 'passed' | 'failed' | 'error';
  elapsedMs: number;
  oracleResults: ShellCommandResult[];
  setupResults: ShellCommandResult[];
  modifiedPaths: string[];
  patchStats: PatchStats;
  pathPolicy: PathPolicyResult;
  usage: ExecutorUsage | null;
  toolCounts: Record<string, number>;
  finalResponse: AgentFinalResponse | null;
  diagnosisTagsMatched: string[];
  transcriptPath: string;
  diffPath: string;
  workspaceNote?: string;
  errorMessage?: string;
};

export type ReferenceSourceKind = 'ref' | 'worktree' | 'binary';

export type RefPreparation = {
  refLabel: string;
  sourceKind: ReferenceSourceKind;
  requestedRef: string;
  resolvedRef: string;
  preparationPath: string | null;
  sigilBin: string;
};

export type RefRunSummary = {
  refLabel: string;
  sourceKind: ReferenceSourceKind;
  requestedRef: string;
  resolvedRef: string;
  taskResults: TaskRunResult[];
  passed: number;
  failed: number;
  errors: number;
  medianElapsedMs: number;
};

export type TaskComparison = {
  taskId: string;
  baseStatus: TaskRunResult['status'];
  candidateStatus: TaskRunResult['status'];
  direction: 'improved' | 'regressed' | 'neutral' | 'mixed';
};

export type CompareSummary = {
  status: 'improved' | 'neutral' | 'regressed' | 'mixed';
  taskIds: string[];
  base: RefRunSummary;
  candidate: RefRunSummary;
  taskComparisons: TaskComparison[];
  generatedAt: string;
};

export type PublishedSummary = {
  runId: string;
  label: string;
  status: CompareSummary['status'];
  generatedAt: string;
  baseRequestedRef?: string;
  baseRef?: string;
  candidateRequestedRef?: string;
  candidateRef?: string;
  passed?: {
    base: number;
    candidate: number;
  };
};
