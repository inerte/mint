# Mint FFI (Foreign Function Interface)

## Overview

Mint can call JavaScript functions and import npm packages using `e` (extern) declarations.

## Syntax

```mint
e module/path
```

That's it. Exactly ONE way to do FFI (canonical form).

## Examples

### Console Output

```mint
e console

λmain()→𝕌=console.log("Hello from Mint!")
```

### Node.js Built-ins

```mint
e fs/promises

λwriteFile(path:𝕊,content:𝕊)→𝕌=fs/promises.writeFile(path,content)

λmain()→𝕌=writeFile("output.txt","Hello, Mint!")
```

### NPM Packages

First install the package:
```bash
npm install axios
```

Then use it:
```mint
e axios

λfetchUser(id:ℤ)→𝕌=axios.get("https://api.example.com/users/" ++ id)

λmain()→𝕌=fetchUser(123)
```

## How It Works

### 1. Declaration

```mint
e module/path
```

Declares that you'll use a JavaScript module.

### 2. Usage

```mint
module/path.member(args)
```

Access members using full namespace path + dot + member name.

### 3. Validation

The compiler validates externals at **link-time**:
- Loads the module (requires `npm install` first)
- Checks if accessed members exist
- Fails BEFORE writing `.js` if member not found

This catches typos WITHOUT needing type annotations!

### 4. Code Generation

```mint
e fs/promises
λmain()→𝕌=fs/promises.readFile("file.txt","utf-8")
```

Compiles to:

```javascript
import * as fs_promises from 'fs/promises';

export function main() {
  return fs_promises.readFile("file.txt", "utf-8");
}
```

## Namespace Rules

- Full path becomes namespace: `e fs/promises` → use as `fs/promises.readFile`
- No conflicts possible: `moduleA/utils` and `moduleB/utils` are different namespaces
- Slash visible in Mint source (machines don't care about syntax aesthetics)
- Converted to underscores in JavaScript: `fs_promises.readFile`

## Validation Examples

### ✅ Works - Correct member

```mint
e console
λmain()→𝕌=console.log("works!")
```

### ❌ Fails - Typo in member

```mint
e console
λmain()→𝕌=console.logg("typo!")
```

```
Error: Member 'logg' does not exist on module 'console'
Available members: log, error, warn, info, debug, ...
Check for typos or see module documentation.
```

### ❌ Fails - Module not installed

```mint
e axios
λmain()→𝕌=axios.get("url")
```

```
Error: Cannot load external module 'axios':
  Cannot find module 'axios'
Make sure it's installed: npm install axios
```

## Type System Integration

Currently uses `any` type for FFI calls (trust mode).

Member validation is **structural** (does it exist?) not type-based.

Future: Optional type declarations for better safety.

## Promises and Async

FFI calls return whatever JavaScript returns, including Promises.

Currently no `await` support (prints `Promise { <pending> }`).

Future feature: `async` functions and `await` expressions.

## Canonical Form

FFI has exactly **ONE syntactic form**:

✅ ONLY: `e module/path`
❌ NO: `extern module/path` (no full keyword)
❌ NO: `e module/path as alias` (no aliasing)
❌ NO: `e module/path{member1,member2}` (no member lists)
❌ NO: Type annotations on extern declarations

This ensures deterministic, unambiguous code generation for LLMs.

## Limitations

### No Direct Object Construction

```mint
❌ Cannot: new Date()
❌ Cannot: new RegExp(pattern)
```

Must use factory functions or FFI wrappers.

### No Method Chaining (Yet)

```mint
❌ Cannot: axios.get(url).then(fn)
```

Each FFI call is a single member access.

Future: Expression-level member access.

### No Class Interop (Yet)

```mint
❌ Cannot: class instances
❌ Cannot: this binding
```

Use functional APIs or wrapper functions.

## Best Practices

### 1. Wrap FFI in Mint Functions

```mint
e console

λlog(msg:𝕊)→𝕌=console.log(msg)
λerror(msg:𝕊)→𝕌=console.error(msg)

λmain()→𝕌={
  log("Info message")
  error("Error message")
}
```

### 2. Use Semantic Names

```mint
e fs/promises

λreadFile(path:𝕊)→𝕌=fs/promises.readFile(path,"utf-8")
λwriteFile(path:𝕊,content:𝕊)→𝕌=fs/promises.writeFile(path,content)
```

### 3. Validate at Boundaries

Use contracts (future feature) to validate FFI inputs/outputs.

## Future Extensions

- Async/await for Promise handling
- Type annotations for FFI declarations
- Method chaining syntax
- Class/object interop
- Callback conversions (JS → Mint functions)

---

**FFI unlocks the entire JavaScript ecosystem for Mint programs!** 🚀
