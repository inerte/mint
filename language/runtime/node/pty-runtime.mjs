import { chmodSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import * as nodePtyModule from "node-pty";

const require = createRequire(import.meta.url);
let spawnHelperPrepared = false;

function spawnFunction() {
  if (typeof nodePtyModule.spawn === "function") {
    return nodePtyModule.spawn;
  }
  if (typeof nodePtyModule.default?.spawn === "function") {
    return nodePtyModule.default.spawn;
  }
  throw new Error("node-pty did not expose a spawn function");
}

function ensureSpawnHelperExecutable() {
  if (spawnHelperPrepared || process.platform === "win32") {
    return;
  }
  spawnHelperPrepared = true;
  try {
    const packageRoot = dirname(require.resolve("node-pty/package.json"));
    const helperPath = join(
      packageRoot,
      "prebuilds",
      `${process.platform}-${process.arch}`,
      "spawn-helper",
    );
    if (existsSync(helperPath)) {
      chmodSync(helperPath, 0o755);
    }
  } catch {}
}

function commandCwd(command) {
  return command?.cwd?.__tag === "Some"
    ? String(command.cwd.__fields?.[0] ?? "")
    : process.cwd();
}

function commandEnv(command) {
  const entries = Array.isArray(command?.env?.__sigil_map)
    ? command.env.__sigil_map
    : [];
  const env = {};
  for (const [key, value] of Object.entries(process.env)) {
    if (typeof value === "string") {
      env[key] = value;
    }
  }
  for (const [key, value] of entries) {
    env[String(key)] = String(value);
  }
  return env;
}

function commandCols(command) {
  return Math.max(1, Math.trunc(Number(command?.cols ?? 80)));
}

function commandRows(command) {
  return Math.max(1, Math.trunc(Number(command?.rows ?? 24)));
}

export async function spawnPty(command) {
  const argv = Array.isArray(command?.argv)
    ? command.argv.map((item) => String(item))
    : [];
  if (argv.length === 0) {
    throw new Error("§pty.spawn requires at least one argv item");
  }
  ensureSpawnHelperExecutable();
  const spawn = spawnFunction();
  return spawn(argv[0], argv.slice(1), {
    cols: commandCols(command),
    cwd: commandCwd(command),
    env: commandEnv(command),
    name: "xterm-color",
    rows: commandRows(command),
  });
}
