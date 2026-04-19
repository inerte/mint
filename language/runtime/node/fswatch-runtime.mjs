import { watch } from "node:fs";
import { access } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";

function normalizeRelativePath(rootPath, filename) {
  if (filename == null) {
    return ".";
  }
  const raw =
    typeof filename === "string"
      ? filename
      : filename instanceof Uint8Array
        ? Buffer.from(filename).toString("utf8")
        : String(filename);
  const absolutePath = resolve(rootPath, raw);
  const relativePath = relative(rootPath, absolutePath);
  if (!relativePath) {
    return ".";
  }
  return relativePath.split(sep).join("/");
}

async function pathExists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

async function pathExistsEventually(path, attempts = 4, delayMs = 10) {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    if (await pathExists(path)) {
      return true;
    }
    if (attempt + 1 < attempts) {
      await sleep(delayMs);
    }
  }
  return false;
}

export async function watchPath(rootPath, onEvent) {
  const absoluteRoot = resolve(String(rootPath ?? "."));
  const watcher = watch(
    absoluteRoot,
    { recursive: true },
    (eventType, filename) => {
      void (async () => {
        const relativePath = normalizeRelativePath(absoluteRoot, filename);
        const absolutePath =
          relativePath === "."
            ? absoluteRoot
            : resolve(absoluteRoot, relativePath);
        let event;
        if (String(eventType) === "change") {
          event = { __tag: "Changed", __fields: [relativePath] };
        } else {
          event = (await pathExistsEventually(absolutePath))
            ? { __tag: "Created", __fields: [relativePath] }
            : { __tag: "Removed", __fields: [relativePath] };
        }
        onEvent(event);
      })().catch(() => {});
    },
  );

  watcher.on("error", () => {
    try {
      watcher.close();
    } catch {}
  });

  return {
    close() {
      watcher.close();
    },
  };
}
