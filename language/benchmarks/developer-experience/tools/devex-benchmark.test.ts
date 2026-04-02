import test from 'node:test';
import assert from 'node:assert/strict';
import os from 'node:os';
import path from 'node:path';
import { promises as fs } from 'node:fs';

import { MockExecutor } from './lib/executor.js';
import { loadTaskManifests } from './lib/manifests.js';
import { publishCompareRun } from './lib/publish.js';
import { compareRefRuns, runTasksForReference } from './lib/runner.js';
import { ensureDir, execShellCommand, writeJsonFile } from './lib/util.js';
import { createWorkingTreeSnapshot } from './lib/workspace.js';
import type { ExecutorResult, TaskManifest } from './lib/types.js';

const benchmarkRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), '..');
const tasksDir = path.join(benchmarkRoot, 'tasks');

test('current task manifests validate', async () => {
  const tasks = await loadTaskManifests(tasksDir);

  assert.equal(tasks.length, 6);
  assert.ok(tasks.some((task) => task.id === 'repair-ingest-received-timestamp'));
  assert.ok(tasks.some((task) => task.id === 'repair-feed-published-timestamp'));
});

test('runner records a passing task result with a mock executor', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'sigil-devex-runner-'));
  const fixturesDir = path.join(root, 'fixtures');
  const fixtureDir = path.join(fixturesDir, 'simple-pass');
  const runDir = path.join(root, '.local', 'runs', 'sample-run');
  await ensureDir(path.join(fixtureDir));
  await fs.writeFile(path.join(fixtureDir, 'note.txt'), 'broken\n', 'utf8');

  const task: TaskManifest = {
    id: 'simple-pass',
    title: 'Simple pass task',
    goal: 'Write a file that satisfies the oracle.',
    initialPrompt: 'Create fixed.txt in the workspace.',
    fixture: 'simple-pass',
    setupCommands: [],
    oracleCommands: [
      {
        command: 'test -f fixed.txt'
      }
    ],
    successCriteria: ['fixed.txt exists'],
    allowedEditPaths: ['fixed.txt'],
    forbiddenEditPaths: ['.local'],
    budgets: {
      maxTurns: 5,
      maxWallClockMs: 60_000
    },
    rootCauseTags: ['missing_output']
  };

  const executor = new MockExecutor(async (context): Promise<ExecutorResult> => {
    await fs.writeFile(path.join(context.workspacePath, 'fixed.txt'), 'ok\n', 'utf8');
    return {
      exitCode: 0,
      finalResponse: {
        summary: 'Created fixed.txt.',
        diagnosis: 'The fixture was missing fixed.txt.',
        diagnosisTags: ['missing_output'],
        filesChanged: ['fixed.txt']
      },
      usage: {
        inputTokens: 10,
        outputTokens: 5
      },
      toolCounts: {
        'event:item.completed': 1
      },
      artifact: {
        events: ['{"type":"item.completed","item":{"type":"agent_message"}}'],
        rawStdout: '',
        rawStderr: ''
      }
    };
  });

  const summary = await runTasksForReference(root, fixturesDir, executor, [task], runDir, {
    repoRoot: root,
    runsLocalDir: path.join(root, '.local', 'runs'),
    refLabel: 'candidate',
    ref: 'HEAD',
    sourceKind: 'ref',
    sigilBinOverride: '/usr/bin/true'
  });

  assert.equal(summary.passed, 1);
  assert.equal(summary.failed, 0);
  assert.equal(summary.taskResults[0].status, 'passed');
  assert.deepEqual(summary.taskResults[0].diagnosisTagsMatched, ['missing_output']);
});

test('working tree snapshots preserve uncommitted changes without copying ignored outputs', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'sigil-devex-snapshot-'));
  const localRoot = path.join(root, '.local', 'runs');
  await execShellCommand('git init -q', root, {});
  await execShellCommand('git config user.email "benchmarks@sigil.local"', root, {});
  await execShellCommand('git config user.name "Sigil Benchmarks"', root, {});
  await fs.writeFile(path.join(root, '.gitignore'), '.local/\n', 'utf8');
  await fs.writeFile(path.join(root, 'tracked.txt'), 'before\n', 'utf8');
  await fs.writeFile(path.join(root, 'keep.txt'), 'keep\n', 'utf8');
  await execShellCommand('git add .', root, {});
  await execShellCommand('git commit -qm "baseline"', root, {});

  await fs.writeFile(path.join(root, 'tracked.txt'), 'after\n', 'utf8');
  await fs.writeFile(path.join(root, 'added.txt'), 'new\n', 'utf8');
  await fs.rm(path.join(root, 'keep.txt'));
  await fs.mkdir(path.join(root, '.local'), { recursive: true });
  await fs.writeFile(path.join(root, '.local', 'ignored.txt'), 'ignored\n', 'utf8');
  const snapshot = await createWorkingTreeSnapshot(root, localRoot, 'candidate');

  assert.match(snapshot.resolvedRef, /\+worktree$/);
  assert.equal(await fs.readFile(path.join(snapshot.snapshotPath, 'tracked.txt'), 'utf8'), 'after\n');
  assert.equal(await fs.readFile(path.join(snapshot.snapshotPath, 'added.txt'), 'utf8'), 'new\n');
  await assert.rejects(fs.readFile(path.join(snapshot.snapshotPath, 'keep.txt'), 'utf8'));
  await assert.rejects(fs.readFile(path.join(snapshot.snapshotPath, '.local', 'ignored.txt'), 'utf8'));
});

test('compare summary is outcome-based and does not require feature metadata', async () => {
  const compare = compareRefRuns(
    {
      refLabel: 'base',
      sourceKind: 'ref',
      requestedRef: 'HEAD',
      resolvedRef: 'aaa111',
      taskResults: [
        {
          taskId: 'demo',
          refLabel: 'base',
          ref: 'aaa111',
          status: 'passed',
          elapsedMs: 100,
          oracleResults: [],
          setupResults: [],
          modifiedPaths: [],
          patchStats: { additions: 1, deletions: 1, filesChanged: 1 },
          pathPolicy: { allowed: true, forbiddenMatches: [], outOfBoundsMatches: [] },
          usage: null,
          toolCounts: {},
          finalResponse: null,
          diagnosisTagsMatched: [],
          transcriptPath: '/tmp/base.jsonl',
          diffPath: '/tmp/base.diff'
        }
      ],
      passed: 1,
      failed: 0,
      errors: 0,
      medianElapsedMs: 100
    },
    {
      refLabel: 'candidate',
      sourceKind: 'worktree',
      requestedRef: 'WORKTREE',
      resolvedRef: 'bbb222+worktree',
      taskResults: [
        {
          taskId: 'demo',
          refLabel: 'candidate',
          ref: 'bbb222+worktree',
          status: 'passed',
          elapsedMs: 80,
          oracleResults: [],
          setupResults: [],
          modifiedPaths: [],
          patchStats: { additions: 1, deletions: 1, filesChanged: 1 },
          pathPolicy: { allowed: true, forbiddenMatches: [], outOfBoundsMatches: [] },
          usage: null,
          toolCounts: {},
          finalResponse: null,
          diagnosisTagsMatched: [],
          transcriptPath: '/tmp/candidate.jsonl',
          diffPath: '/tmp/candidate.diff'
        }
      ],
      passed: 1,
      failed: 0,
      errors: 0,
      medianElapsedMs: 80
    }
  );

  assert.equal(compare.status, 'improved');
  assert.deepEqual(compare.taskIds, ['demo']);
  assert.equal(compare.taskComparisons[0].direction, 'improved');
});

test('publish writes history and latest summary files', async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), 'sigil-devex-publish-'));
  const resultsDir = path.join(root, 'results');
  const runDir = path.join(root, '.local', 'runs', 'publish-sample');
  await ensureDir(runDir);

  const base = {
    refLabel: 'base',
    sourceKind: 'ref',
    requestedRef: 'main',
    resolvedRef: 'aaa111',
    taskResults: [],
    passed: 1,
    failed: 0,
    errors: 0,
    medianElapsedMs: 100
  };
  const candidate = {
    refLabel: 'candidate',
    sourceKind: 'worktree',
    requestedRef: 'WORKTREE',
    resolvedRef: 'bbb222+worktree',
    taskResults: [],
    passed: 2,
    failed: 0,
    errors: 0,
    medianElapsedMs: 80
  };
  const compare = compareRefRuns(base, candidate);
  await writeJsonFile(path.join(runDir, 'compare.json'), compare);

  const published = await publishCompareRun(resultsDir, runDir, 'smoke-sample');

  assert.equal(published.label, 'smoke-sample');
  assert.match(await fs.readFile(path.join(resultsDir, 'history.jsonl'), 'utf8'), /smoke-sample/);
  assert.match(await fs.readFile(path.join(resultsDir, 'LATEST.md'), 'utf8'), /Latest Developer-Experience Benchmark/);
});
