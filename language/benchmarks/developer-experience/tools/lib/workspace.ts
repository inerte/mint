import { promises as fs } from 'node:fs';
import os from 'node:os';
import path from 'node:path';

import { copyDirectory, ensureDir, execShellCommand, summarizePatchStats } from './util.js';
import type { PatchStats, PathPolicyResult, TaskManifest } from './types.js';

async function symlinkIfPresent(sourcePath: string, targetPath: string, type?: Parameters<typeof fs.symlink>[2]): Promise<void> {
  const stat = await fs.lstat(sourcePath).catch(() => null);
  if (!stat) {
    return;
  }

  await fs.symlink(sourcePath, targetPath, type);
}

export async function prepareTaskWorkspace(
  task: TaskManifest,
  fixturesDir: string,
  languageRootPath?: string
): Promise<string> {
  const fixtureDir = path.join(fixturesDir, task.fixture);
  const workspaceRoot = await fs.mkdtemp(path.join(os.tmpdir(), `sigil-devex-${task.id}-`));
  const repoRoot = process.cwd();

  await copyDirectory(fixtureDir, workspaceRoot);
  await fs.rm(path.join(workspaceRoot, '.local'), { recursive: true, force: true });
  if (languageRootPath) {
    await fs.symlink(languageRootPath, path.join(workspaceRoot, 'language'), 'dir');
  }
  await symlinkIfPresent(path.join(repoRoot, 'package.json'), path.join(workspaceRoot, 'package.json'), 'file');
  await symlinkIfPresent(path.join(repoRoot, 'pnpm-workspace.yaml'), path.join(workspaceRoot, 'pnpm-workspace.yaml'), 'file');
  await symlinkIfPresent(path.join(repoRoot, 'pnpm-lock.yaml'), path.join(workspaceRoot, 'pnpm-lock.yaml'), 'file');
  await symlinkIfPresent(path.join(repoRoot, 'node_modules'), path.join(workspaceRoot, 'node_modules'), 'dir');
  await execShellCommand('git init -q', workspaceRoot, {});
  await ensureDir(path.join(workspaceRoot, '.git', 'info'));
  await fs.appendFile(path.join(workspaceRoot, '.git', 'info', 'exclude'), '\n.local/\n', 'utf8');
  await execShellCommand('git config user.email "benchmarks@sigil.local"', workspaceRoot, {});
  await execShellCommand('git config user.name "Sigil Benchmarks"', workspaceRoot, {});
  await execShellCommand('git add .', workspaceRoot, {});
  await execShellCommand('git commit -qm "fixture baseline"', workspaceRoot, {});

  return workspaceRoot;
}

export async function cleanupWorkspace(workspacePath: string): Promise<void> {
  await fs.rm(workspacePath, { recursive: true, force: true });
}

export async function collectModifiedPaths(workspacePath: string): Promise<string[]> {
  const status = await execShellCommand('git status --porcelain', workspacePath, {});

  return status.stdout
    .split('\n')
    .filter(Boolean)
    .map((line) => line.slice(3).trim())
    .filter((relativePath) => !relativePath.startsWith('.local/'))
    .sort();
}

export async function collectPatch(workspacePath: string): Promise<{ diff: string; stats: PatchStats }> {
  const patch = await execShellCommand('git diff --binary --no-ext-diff', workspacePath, {});
  const numstat = await execShellCommand('git diff --numstat --no-ext-diff', workspacePath, {});

  return {
    diff: patch.stdout,
    stats: summarizePatchStats(numstat.stdout)
  };
}

export function evaluatePathPolicy(task: TaskManifest, modifiedPaths: string[]): PathPolicyResult {
  const normalize = (value: string) => value.replace(/\\/g, '/').replace(/^\.?\//, '').replace(/\/+$/, '');
  const matches = (value: string, prefixes: string[]) => {
    const normalizedValue = normalize(value);
    return prefixes.some((prefix) => {
      const normalizedPrefix = normalize(prefix);
      return normalizedValue === normalizedPrefix || normalizedValue.startsWith(`${normalizedPrefix}/`);
    });
  };

  const forbiddenMatches = modifiedPaths.filter((modifiedPath) => matches(modifiedPath, task.forbiddenEditPaths));
  const outOfBoundsMatches = modifiedPaths.filter((modifiedPath) => !matches(modifiedPath, task.allowedEditPaths));

  return {
    allowed: forbiddenMatches.length === 0 && outOfBoundsMatches.length === 0,
    forbiddenMatches,
    outOfBoundsMatches
  };
}

export async function createWorktree(repoRoot: string, localRoot: string, refLabel: string, ref: string): Promise<{ worktreePath: string; resolvedRef: string }> {
  await ensureDir(localRoot);
  const worktreePath = await fs.mkdtemp(
    path.join(localRoot, `${refLabel}-${ref.replace(/[^A-Za-z0-9._-]/g, '_')}-`)
  );
  const add = await execShellCommand(`git worktree add --detach "${worktreePath}" "${ref}"`, repoRoot, {}, 600_000);
  if (add.exitCode !== 0) {
    await fs.rm(worktreePath, { recursive: true, force: true });
    throw new Error(`failed to create worktree for ref '${ref}': ${add.stderr || add.stdout}`);
  }

  const resolved = await execShellCommand('git rev-parse HEAD', worktreePath, {});
  return {
    worktreePath,
    resolvedRef: resolved.stdout.trim()
  };
}

function splitNullSeparatedPaths(value: string): string[] {
  return value
    .split('\0')
    .map((entry) => entry.trim())
    .filter(Boolean);
}

export async function createWorkingTreeSnapshot(
  repoRoot: string,
  localRoot: string,
  refLabel: string
): Promise<{ snapshotPath: string; resolvedRef: string }> {
  await ensureDir(localRoot);
  const snapshotPath = await fs.mkdtemp(path.join(localRoot, `${refLabel}-worktree-`));

  const head = await execShellCommand('git rev-parse HEAD', repoRoot, {});
  const tracked = await execShellCommand('git ls-files -z', repoRoot, {});
  const untracked = await execShellCommand('git ls-files --others --exclude-standard -z', repoRoot, {});
  const entries = Array.from(new Set([
    ...splitNullSeparatedPaths(tracked.stdout),
    ...splitNullSeparatedPaths(untracked.stdout)
  ])).sort();

  for (const entry of entries) {
    const sourcePath = path.join(repoRoot, entry);
    const targetPath = path.join(snapshotPath, entry);
    const stat = await fs.lstat(sourcePath).catch(() => null);

    if (!stat) {
      continue;
    }

    await ensureDir(path.dirname(targetPath));
    await fs.cp(sourcePath, targetPath, { recursive: stat.isDirectory(), force: true });
  }

  return {
    snapshotPath,
    resolvedRef: `${head.stdout.trim()}+worktree`
  };
}

export async function removeWorktree(repoRoot: string, worktreePath: string): Promise<void> {
  await execShellCommand(`git worktree remove --force "${worktreePath}"`, repoRoot, {}, 600_000);
  await fs.rm(worktreePath, { recursive: true, force: true });
}
