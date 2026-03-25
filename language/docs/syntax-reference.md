# Sigil Syntax Reference

This document describes the current Sigil surface accepted by the compiler in
this repository.

Sigil is canonical by design. This is not a style guide with alternatives. It
documents the one surface form the parser, internal canonical printer,
validator, and typechecker accept.

If source parses but does not exactly match the compiler's canonical printed
form for that AST, `compile`, `run`, and `test` reject it. There is no public
formatter.

## Source Files

Sigil distinguishes file purpose with file extensions:

- `.lib.sigil` for libraries
- `.sigil` for executables and tests

Canonical filename rules:

- basename must be `lowerCamelCase`
- no underscores
- no hyphens
- no spaces
- filename must end with `.sigil` or `.lib.sigil`

Valid examples:

- `userService.lib.sigil`
- `fibonacci.sigil`
- `ffiNodeConsole.lib.sigil`

Invalid examples:

- `UserService.lib.sigil`
- `user_service.lib.sigil`
- `user-service.lib.sigil`

## Comments

Sigil uses one comment syntax:

```text
⟦ This is a comment ⟧
```

`#`, `//`, and `/* ... */` are not Sigil comments.

## Top-Level Declarations

Module scope is declaration-only.

Valid top-level forms:

- `t`
- `e`
- `i`
- `c`
- `λ`
- `test`

Invalid at top level:

- `l`

Canonical declaration ordering is:

```text
t => e => c => λ => test
```

There is no `export` keyword in current Sigil. Visibility is file-based:

- top-level declarations in `.lib.sigil` files are referenceable from other modules
- `.sigil` files are executable-oriented
- top-level functions, consts, and types in `.sigil` files must be reachable
  from `main` or tests
- `.lib.sigil` files may still expose declarations that are unused locally

## Function Declarations

Function declarations require:

- a name
- typed parameters
- a return type

Regular expression body:

```sigil module
λadd(x:Int,y:Int)=>Int=x+y
```

Match body:

```sigil module
λfactorial(n:Int)=>Int match n{
  0=>1|
  1=>1|
  value=>value*factorial(value-1)
}
```

For function declarations:

- `=` is required before a non-`match` body
- `=` is forbidden before a `match` body
- the canonical printer keeps the full signature on one physical line
- a direct `match` body begins on that same line

Effects, when present, appear between `=>` and the return type:

```sigil program
e axios:{get:λ(String)=>!Http String}

e console:{log:λ(String)=>!Log Unit}

λfetchUser(id:Int)=>!Http String=axios.get("https://example.com/"+§string.intToString(id))

λmain()=>!Log Unit=console.log("hello")
```

The built-in primitive effects are:

- `Clock`
- `Fs`
- `Http`
- `Log`
- `Process`
- `Tcp`
- `Timer`

Projects may define reusable multi-effect aliases only in `src/effects.lib.sigil`:

```sigil module projects/docsDriftAudit/src/effects.lib.sigil
effect CliIo=!Fs!Log!Process
```

Those aliases are project-global and may be used directly in signatures.

## Lambda Expressions

Lambda expressions are fully typed and use the same body rule as top-level
functions:

```sigil expr
λ(x:Int)=>Int=x*2
```

```sigil expr
λ(value:Int)=>Int match value{
  0=>1|
  n=>n+1
}
```

Lambda expressions require:

- parentheses around parameters
- typed parameters
- a return type

Generic lambdas are not part of Sigil's surface.

## Type Declarations

### Product Types

```sigil module
t User={active:Bool,id:Int,name:String}
```

Record fields are canonical alphabetical order everywhere records appear.

### Sum Types

```sigil module
t Color=Red()|Green()|Blue()

t ConcurrentOutcome[T,E]=Aborted()|Failure(E)|Success(T)

t Option[T]=Some(T)|None()

t Result[T,E]=Ok(T)|Err(E)
```

Imported constructors use qualified module syntax in expressions and patterns:

```sigil module projects/algorithms/src/orderingExample.lib.sigil
λorderingResult()=>•graphTypes.TopologicalSortResult=•graphTypes.Ordering([1,2,3])

λorderingValues()=>[Int] match orderingResult(){
  •graphTypes.Ordering(order)=>order|
  •graphTypes.CycleDetected()=>[]
}
```

## Constants

Constants require a value ascription:

```sigil module
c answer=(42:Int)

c greeting=("hello":String)
```

Current parser behavior requires the typed form above. Untyped constants and the
older `c name:Type=value` surface are not current Sigil.

## Rooted Module References

Sigil uses rooted module references directly at the use site. There are no
top-level import declarations:

```sigil module projects/todo-app/src/importsExample.lib.sigil
λtodoCount(todos:[•todoDomain.Todo])=>Int=•todoDomain.completedCount(todos)
```

Use rooted members through the namespace:

```sigil expr
•todoDomain.completedCount(todos)
```

```sigil expr
§list.last(items)
```

Canonical roots include:

- `¶...`
- `¤...`
- `•...`
- `§...`
- `※...`
- `†...`

There are no selective imports, import aliases, or separate import
declarations.

## Externs

Extern declarations use `e`:

```sigil module
e console:{log:λ(String)=>!Log Unit}

e axios:{get:λ(String)=>!Http String}
```

Unused extern declarations are non-canonical.

## Local Bindings

Local bindings use `l` inside expressions:

```sigil module
λdoubleAndAdd(x:Int,y:Int)=>Int={
  l doubled=(x*2:Int);
  doubled+doubled+y
}
```

Local names must not shadow names from the same or any enclosing lexical scope.

Named local bindings used zero times are non-canonical.

Pure local bindings used exactly once are non-canonical and must be inlined.

When a binding exists only to sequence effects, use the wildcard pattern:

```sigil expr
l _=(§io.println("x"):Unit);
next
```

## Pattern Matching

Sigil uses `match` for value-based branching:

```sigil module
λclassify(value:Int)=>String match value{
  0=>"zero"|
  1=>"one"|
  _=>"many"
}
```

Canonical `match` shape comes from the internal printer:

- multi-arm `match` prints multiline
- each arm begins as `pattern=>`
- nested branching may continue on following indented lines
- there is no alternate printed layout for the same `match` AST

Patterns include:

- literals
- identifiers
- `_`
- constructors
- list patterns
- record patterns

Examples:

```sigil module
λfromOption(option:Option[Int])=>Int match option{
  Some(value)=>value|
  None()=>0
}

λheadOrZero(list:[Int])=>Int match list{
  []=>0|
  [head,.rest]=>head
}
```

## Lists, Maps, and Records

List type:

```sigil module
t IntList=[Int]
```

List literal:

```sigil expr
[1,2,3]
```

Map type:

```sigil module
t StringIntMap={String↦Int}
```

Map literals use `↦`:

```sigil exprs
{"a"↦1,"b"↦2}
({↦}:{String↦Int})
```

Record types and literals use `:`:

```sigil module
t User={id:Int,name:String}

λsampleUser()=>User={id:1,name:"Ana"}
```

## Built-In List Operators

Sigil includes canonical list operators:

- `map` projection
- `filter` filtering
- `reduce ... from ...` ordered reduction
- `⧺` concatenation

Examples:

```sigil module
λconcatenated()=>[Int]=[1,2]⧺[3,4]

λdoubled()=>[Int]=[1,2,3] map (λ(x:Int)=>Int=x*2)

λfiltered()=>[Int]=[1,2,3] filter (λ(x:Int)=>Bool=x>1)

λsummed()=>Int=[1,2,3] reduce (λ(acc:Int,x:Int)=>Int=acc+x) from 0
```

`map` and `filter` require pure callbacks.

## Concurrent Regions

Sigil uses one explicit concurrency surface:

```sigil program
λmain()=>!Timer [ConcurrentOutcome[Int,String]]=concurrent urlAudit@5:{jitterMs:Some({max:25,min:1}),stopOn:shouldStop,windowMs:Some(1000)}{
  spawn one()
  spawnEach [1,2,3] process
}

λone()=>!Timer Result[Int,String]={
  l _=(§time.sleepMs(0):Unit);
  Ok(1)
}

λprocess(value:Int)=>!Timer Result[Int,String]={
  l _=(§time.sleepMs(0):Unit);
  Ok(value)
}

λshouldStop(err:String)=>Bool=false
```

Rules:

- regions are named: `concurrent name@width{...}`
- width is required after `@`
- optional policy attaches as `:{...}`
- policy fields are canonical alphabetical order:
  - `jitterMs`
  - `stopOn`
  - `windowMs`
- region bodies are spawn-only:
  - `spawn expr`
  - `spawnEach list fn`
- `spawn` requires an effectful computation returning `Result[T,E]`
- `spawnEach` requires a list and an effectful function returning `Result[T,E]`
- regions return `[ConcurrentOutcome[T,E]]`

Omitted policy defaults to no jitter, no early stop, and no windowing.

`windowMs` and `jitterMs` belong to the region policy, not to `map` or `filter`.

Sigil also treats these operators as the canonical surface for common list
plumbing:

- do not hand-write recursive `all` clones; use `§list.all`
- do not hand-write recursive `any` clones; use `§list.any`
- do not count with `#(xs filter pred)`; use `§list.countIf`
- do not hand-write recursive `map` clones when `map` fits
- do not hand-write recursive `filter` clones when `filter` fits
- do not hand-write recursive `find` clones; use `§list.find`
- do not hand-write recursive `flatMap` clones; use `§list.flatMap`
- do not hand-write recursive `fold` clones when `reduce ... from ...` fits
- do not hand-write recursive `reverse` clones; use `§list.reverse`
- do not build recursive list results with `self(rest)⧺rhs`

## Tests

Tests are top-level declarations and must live under `tests/`:

```sigil program language/test-fixtures/tests/addsNumbers.sigil
λmain()=>Unit=()

test "adds numbers" {
  1+1=2
}
```

Effectful tests use explicit effect annotations:

```sigil program language/test-fixtures/tests/writesLog.sigil
λmain()=>Unit=()

test "writes log" =>!Log  {
  §io.println("x")=()
}
```

Tests may also derive the active world locally:

```sigil program language/test-fixtures/tests/testWorld.sigil
λmain()=>Unit=()

test "captured log contains line" =>!Log world {
  c log=(†log.capture():†log.LogEntry)
} {
  §io.println("captured")=() and
  ※check::log.contains("captured")
}
```

Rules:

- `world { ... }` appears between the optional test effects and the body
- world clauses are declaration-only and use `c` bindings
- world bindings must be pure entry values from `†...`
- `※observe` and `※check` are test-only roots for reading the active test world

## Canonical References

For canonical formatting and validator-enforced rules, see:

- `language/docs/CANONICAL_FORMS.md`
- `language/docs/CANONICAL_ENFORCEMENT.md`
