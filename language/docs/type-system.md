# Sigil Type System: Bidirectional Type Checking

## Overview

Sigil uses **bidirectional type checking** instead of traditional Hindley-Milner type inference.

### Why Bidirectional?

Sigil's philosophy is **"ONE way to write it"**. Type annotations must be:
- **Mandatory** on all function signatures
- **Explicit** (no optional syntax)
- **Canonical** (exactly one valid representation)

This makes Hindley-Milner's primary feature (inferring types with minimal annotations) unnecessary. Bidirectional type checking is:
- **Simpler** to implement (~40% less code)
- **Better** error messages ("expected Int, got String" instead of "failed to unify")
- **More extensible** (natural framework for polymorphism, effects, refinements)
- **Faster** to compile (no complex constraint solving in common cases)

## Two Modes

### Synthesis (⇒): Infer type from structure

Used for expressions where type can be determined from the expression itself:
- **Literals**: `5` ⇒ `Int`, `"hello"` ⇒ `String`, `true` ⇒ `Bool`
- **Variables**: `x` ⇒ look up in environment
- **Applications**: `f(x)` ⇒ synthesize `f`, check args, return result type
- **Pattern matching**: `match n{...}` ⇒ synthesize scrutinee, check arms have same type
- **Binary operations**: `x + y` ⇒ check operands, return result type

### Checking (⇐): Verify against expected type

Used for expressions where expected type is known from context:
- **Lambda bodies**: check against declared return type
- **Pattern match arms**: check against expected result type
- **Function arguments**: check against parameter types
- **Literals**: verify literal matches expected type

## Type Annotations

### Required Everywhere

All function signatures must have complete type annotations:

```sigil
⟦ Function declarations ⟧
λfactorial(n:Int)→Int=...

⟦ Lambda expressions ⟧
[1,2,3]↦λ(x:Int)→Int=x*2

⟦ Constants (when supported) ⟧
c pi:Float=3.14
```

### Parse Errors for Missing Annotations

The parser rejects code without type annotations:

```
Error: Expected ":" after parameter "n"
λfactorial(n)→Int=...
           ^
Type annotations are required (canonical form).

Error: Expected "→" after parameters for function "factorial"
λfactorial(n:Int)=...
               ^
Return type annotations are required (canonical form).
```

## Error Messages

Bidirectional type checking provides **excellent error messages**:

```
Error: Type mismatch in function 'main'
  Expected: Int
  Got: String
  Location: factorial.sigil:2:16

  2 | λmain()→Int="hello"
    |                ^

Literal type mismatch: expected Int, got String
```

Compare to traditional Hindley-Milner errors:
```
Failed to unify types Int and String
(no clear location or context)
```

## Type Inference Rules

The type checker uses two main functions:

```typescript
synthesize(expr: Expr, env: Env): Type
check(expr: Expr, expectedType: Type, env: Env): void
```

### Synthesis Rules

```
Γ ⊢ 5 ⇒ Int                           (Literal-Int)

Γ ⊢ "hello" ⇒ String                     (Literal-String)

x : T ∈ Γ
─────────────                        (Var)
Γ ⊢ x ⇒ T

Γ ⊢ f ⇒ (T₁,...,Tₙ) → R
Γ ⊢ e₁ ⇐ T₁  ...  Γ ⊢ eₙ ⇐ Tₙ
────────────────────────────         (App)
Γ ⊢ f(e₁,...,eₙ) ⇒ R

Γ ⊢ e ⇒ T
Γ, x₁:T₁,...,xₙ:Tₙ = match(p, T)
Γ, x₁:T₁,...,xₙ:Tₙ ⊢ body ⇒ R
──────────────────────────────       (Match-Arm)
Γ ⊢ match e{p→body|...} ⇒ R
```

### Checking Rules

```
Γ ⊢ e ⇒ T    T = T'
────────────────────                 (Check-Synth)
Γ ⊢ e ⇐ T'

λ(x₁:T₁,...,xₙ:Tₙ)→R annotation
Γ, x₁:T₁,...,xₙ:Tₙ ⊢ body ⇐ R
────────────────────────────         (Lambda)
Γ ⊢ λ(x₁:T₁,...,xₙ:Tₙ)→R=body ⇐ (T₁,...,Tₙ)→R
```

## Implementation

### Status: ✅ Completed (2026-02-22)

The bidirectional type checker is fully implemented and integrated into the compiler pipeline.

**Location:** `compiler/crates/sigil-typechecker/src/`
- `lib.rs` - Main entry point
- `types.rs` - Type representations
- `errors.rs` - Error formatting
- `bidirectional.rs` - Core type checking algorithm

### Current Phase: Explicit Generics Without HM

Sigil now supports **explicit parametric polymorphism** at declaration boundaries.

Implemented today:
- Primitive types: `Int` (Int), `String` (String), `Bool` (Bool), `Unit` (Unit)
- Function types: `λ(T₁,...,Tₙ)→R`
- List types: `[T]`
- Tuple types: `(T₁,T₂,...,Tₙ)`
- Record types: `{field₁:T₁, field₂:T₂, ...}`
- Map types: `{K↦V}`
- Generic type declarations: `Option[T]`, `Result[T,E]`, user-defined generic ADTs
- Generic top-level function declarations: `λmap[T,U](fn:λ(T)→U,list:[T])→[U]`
- Canonical form requires record fields to be alphabetically ordered everywhere records appear
- Local bindings remain monomorphic unless they refer to an explicitly generic top-level declaration

**Type equality** uses canonical structural comparison:
```typescript
function typesEqual(t1: Type, t2: Type): boolean {
  // Int = Int, String = String, etc.
  // (A→B) = (C→D) if A=C and B=D
  // [T] = [U] if T = U
  // etc.
}
```

Before equality-sensitive checks, Sigil normalizes aliases and named product types to
their canonical structural form. This is not inference. It is canonical semantic
comparison for already-explicit types.

Examples:

```sigil
t MkdirOptions={recursive:Bool}
t Todo={done:Bool,id:Int,text:String}

⟦ Named product type and structural record are the same after normalization ⟧
c opts=({recursive:true}:MkdirOptions)

⟦ [Todo] and [{done:Bool,id:Int,text:String}] compare by canonical form ⟧
λaddTodo(id:Int,text:String,todos:[Todo])→[Todo]=[Todo{done:false,id:id,text:text}]⧺todos
```

Sigil keeps **sum types nominal**. A sum type does not normalize into a structural
record payload just because one of its variants carries a record.

Sigil does **not** use Hindley-Milner let-polymorphism:
- `l id=...` does not become implicitly polymorphic
- lambdas are not generic declarations
- there is no call-site type argument syntax like `f[Int](x)`

Generic instantiation is driven by ordinary bidirectional typing:
- argument types
- expected return types
- type ascriptions
- pattern-match scrutinee types

`Option[T]`, `Result[T,E]`, `Some`, `None`, `Ok`, and `Err` are part of the implicit `core⋅prelude`.
`Map` is a core collection concept with:
- type syntax `{K↦V}`
- literal syntax `{key↦value,...}` and `{↦}`

Records are exact fixed-shape products using `:`.
Maps remain dynamic keyed collections using `↦`.

That exactness is not just documentation. Sigil enforces:
- no row polymorphism
- no width subtyping for records
- no open or partial record forms
- no “maybe this field exists” semantics outside explicit `Option[T]`

If a field might be absent, the canonical answer is:

```sigil
t MaybeMessage={createdAt:Option[stdlib⋅time.Instant],text:String}
```

not an open record, optional-field syntax, or a half-populated record value.

## Trusted Internal Data

Sigil wants internal code to operate on trusted values, not on raw boundary blobs.

Practical rule:

```text
raw JSON / raw text / raw protocol value
→ parse
→ decode / validate
→ exact internal record or named wrapper
```

For example, this is the intended shape:

```sigil
t Message={createdAt:stdlib⋅time.Instant,text:String}
```

Once business logic has a `Message`, `message.createdAt` is simply there.
Sigil is trying to make “defensively check the field again just in case” both
unnecessary and mechanically suspicious.

When a validated value should remain distinct from a raw primitive, use a named
wrapper rather than an alias:

```sigil
t Email=Email(String)
t UserId=UserId(Int)
```

This keeps “validated internal value” separate from “raw string/int from a boundary”.

### Future Phase: Advanced Features

**Phase 3+** (Future): Extend as needed
- **Higher-rank polymorphism**: Functions taking polymorphic functions
- **Refinement types**: Types with constraints (e.g., `{n:Int | n > 0}`)
- **Effect tracking**: `λread()→!IO String`
- **Dependent types**: If needed for verification

All these remain easier to add on top of bidirectional typing than on top of Hindley-Milner.

## Comparison: Bidirectional vs Hindley-Milner

| Feature | Hindley-Milner | Bidirectional |
|---------|----------------|---------------|
| **Type annotations** | Optional | Mandatory in canonical positions |
| **Best for** | Type inference | Type checking |
| **Error messages** | "Failed to unify X and Y" | "Expected X, got Y at line:col" |
| **Implementation** | Complex (generalization + implicit polymorphic locals) | Simpler and more explicit |
| **Code size** | ~1,468 lines (inference + unification + patterns) | ~829 lines |
| **Extensibility** | Hard to extend | Natural framework |
| **Performance** | Good for inference | Excellent for checking |
| **Fit for Sigil** | Too much hidden behavior | Matches Sigil's machine-first explicitness |

## Pattern Matching Type Checking

Pattern matching is type-checked using bidirectional rules:

```sigil
λlength(list:[Int])→Int match list{
  []→0|
  [_,.rest]→1+length(rest)
}
```

Type checking:
1. **Synthesize** scrutinee type: `list : [Int]`
2. **Check** each pattern against scrutinee type:
   - `[]` : `[Int]` ✓ (empty list pattern)
   - `[_,.rest]` : `[Int]` ✓ (binds `rest : [Int]`)
3. **Synthesize** each arm body:
   - `0` ⇒ `Int` ✓
   - `1+length(rest)` ⇒ `Int` ✓
4. **Verify** all arms have same type: `Int = Int` ✓
5. **Return** result type: `Int`

## List Operations

Built-in list operations are type-checked specially:

```sigil
[1,2,3]↦λ(x:Int)→Int=x*2        ⟦ [Int] ↦ (Int→Int) ⇒ [Int] ⟧
[1,2,3]⊳λ(x:Int)→Bool=x>1        ⟦ [Int] ⊳ (Int→Bool) ⇒ [Int] ⟧
[1,2,3]⊕λ(acc:Int,x:Int)→Int=acc+x⊕0  ⟦ [Int] ⊕ (Int→Int→Int) ⊕ Int ⇒ Int ⟧
```

Type rules:
```
Γ ⊢ list ⇒ [T]
Γ ⊢ fn ⇐ λ(T)→U
─────────────────
Γ ⊢ list↦fn ⇒ [U]

Γ ⊢ list ⇒ [T]
Γ ⊢ pred ⇐ λ(T)→Bool
────────────────────
Γ ⊢ list⊳pred ⇒ [T]

Γ ⊢ list ⇒ [T]
Γ ⊢ fn ⇐ λ(R,T)→R
Γ ⊢ init ⇐ R
──────────────────────
Γ ⊢ list⊕fn⊕init ⇒ R
```

## Sum Types (Algebraic Data Types)

Sigil supports sum types (also called tagged unions or algebraic data types) for type-safe value representation.

### Syntax

```sigil
⟦ Simple enum (no type parameters) ⟧
t Color=Red|Green|Blue

⟦ Generic Option type ⟧
t Option[T]=Some(T)|None

⟦ Generic Result type ⟧
t Result[T,E]=Ok(T)|Err(E)

⟦ Multiple fields ⟧
t Tree[T]=Leaf(T)|Branch(Tree[T],Tree[T])
```

### Type Declaration

Sum types are declared with `t TypeName=Variant1|Variant2|...`:
- Type name must be uppercase
- Variant names must be uppercase
- Variants can have zero or more fields
- Type parameters use `[T,U,...]` syntax

### Constructor Calls

Constructors are functions that create sum type values:

```sigil
⟦ Nullary constructors (no fields) - require () ⟧
λgetRed()→Color=Red()
λgetGreen()→Color=Green()

⟦ Constructors with fields ⟧
λsomeValue()→Option=Some(42)
λnoValue()→Option=None()

⟦ Multiple fields ⟧
λokResult()→Result=Ok(100)
λerrResult()→Result=Err("file not found")
```

**Important:** Even nullary constructors (like `Red`, `None`) require `()` to be called.

Imported constructors use the same fully qualified namespace style as imported functions:

```sigil
i src⋅graphTypes

λsorted(order:[Int])→src⋅graphTypes.TopologicalSortResult=
  src⋅graphTypes.Ordering(order)
```

### Pattern Matching

Sum types are deconstructed using pattern matching:

```sigil
⟦ Match on simple enum ⟧
λcolorToInt(color:Color)→Int match color{
  Red→1|
  Green→2|
  Blue→3
}

⟦ Extract values from constructors ⟧
λprocessOption(opt:Option)→Int match opt{
  Some(x)→x|
  None→0
}

⟦ Nested patterns ⟧
λprocessResult(res:Result)→String match res{
  Ok(value)→"Success: "+value|
  Err(msg)→"Error: "+msg
}

⟦ Imported constructor patterns use fully qualified names ⟧
λproject(result:src⋅graphTypes.TopologicalSortResult)→[Int] match result{
  src⋅graphTypes.Ordering(order)→order|
  src⋅graphTypes.CycleDetected()→[]
}
```

### Type Checking Rules

Constructor pattern matching is type-checked with environment lookup:

```
Γ ⊢ scrutinee ⇒ Constructor(TypeName, [])
Constructor ∈ Γ
Γ ⊢ Constructor ⇒ (T₁,...,Tₙ) → Constructor(TypeName, [])
Γ, x₁:T₁,...,xₙ:Tₙ ⊢ body ⇒ R
────────────────────────────────────────────────
Γ ⊢ Constructor(x₁,...,xₙ)→body : R  (Constructor-Pattern)
```

The type checker:
1. Looks up constructor in environment
2. Verifies constructor returns the scrutinee type
3. Binds pattern variables to constructor parameter types
4. Type checks the arm body with extended environment

### Code Generation

Sum types compile to TypeScript/JavaScript objects with `__tag` and `__fields`:

```javascript
// t Color=Red|Green|Blue compiles to:
export function Red() {
  return { __tag: "Red", __fields: [] };
}
export function Green() {
  return { __tag: "Green", __fields: [] };
}
export function Blue() {
  return { __tag: "Blue", __fields: [] };
}

// Pattern matching compiles to:
// match color{Red→1|...} becomes:
switch(color.__tag) {
  case "Red": return 1;
  // ...
}
```

### Standard Library Types

The standard library provides two essential sum types:

**Option[T]** - Represents optional values:
```sigil
t Option[T]=Some(T)|None

⟦ Usage ⟧
λdivide(a:Int,b:Int)→Option match b{
  0→None()|
  b→Some(a/b)
}
```

**Result[T,E]** - Represents success or failure:
```sigil
t Result[T,E]=Ok(T)|Err(E)

⟦ Usage ⟧
λparseInt(s:String)→Result match validInput(s){
  true→Ok(parseInt(s))|
  false→Err("invalid input")
}
```

### Current Limitations

**Phase 1** (Implemented):
- Sum type declarations with `t Name=V1|V2`
- Constructor function generation
- Pattern matching with type checking
- Generic type declarations (`Option[T]`, `Result[T,E]`)

**Limitations:**
- Generic type inference incomplete (type parameters use `any`)
- No generic utility functions yet (e.g., `map[T,U](opt,fn)` not supported)
- Nullary constructors require explicit `()` calls

**Future improvements:**
- Full generic type inference
- Type parameter constraints
- Generic utility functions in stdlib
- Exhaustiveness checking for pattern matches

### Examples

See `examples/sumTypesDemo.sigil` for comprehensive examples including:
- Simple enums (Color)
- Generic Option and Result types
- Pattern matching techniques
- Practical use cases

## Concatenation Operators

Sigil uses distinct operators for distinct concatenation semantics:

- `++` for string concatenation (`String × String → String`)
- `⧺` for list concatenation (`[T] × [T] → [T]`)

```sigil
λgreet(name:String)→String="Hello, "++name
λmerge(xs:[Int],ys:[Int])→[Int]=xs⧺ys
```

This preserves canonical surface forms by avoiding one overloaded concat operator for different data kinds.

## Empty List Contextual Typing

The empty list literal `[]` requires type context to determine its element type.
Non-empty list literals preserve nesting exactly as written; they do not implicitly concatenate inner lists.

**Works in these contexts:**
- **Function return type**: `λf()→[Int]=[]` provides `[Int]` context
- **Pattern matching arms**: First arm establishes type for subsequent arms
- **Record literals**: Expected record type provides context for field values
- **Explicit checking contexts**: Where expected type flows downward

**Example - Pattern Matching:**
```sigil
⟦ Basic: empty list infers from function return type ⟧
λemptyInts()→[Int]=[]

⟦ Pattern matching: first arm pattern infers from scrutinee, body from return type ⟧
λreverse(xs:[Int])→[Int] match xs{
  []→[]|                 ⟦ OK: expected type is [Int] from function signature ⟧
  [x,.rest]→reverse(rest)⧺[x]
}

⟦ Pattern matching: subsequent arms checked against first arm's type ⟧
λfirstNonEmpty(a:[Int],b:[Int])→[Int] match a{
  [x,.xs] → a|      ⟦ First arm synthesizes to [Int] ⟧
  [] → b            ⟦ Second arm checked against [Int] from first arm ⟧
}

⟦ Multiple empty arms work when return type provides context ⟧
t Foo=A|B|C

λtest(x:Foo)→[Int] match x{
  A → [1,2,3]|      ⟦ First arm synthesizes to [Int] ⟧
  B → []|           ⟦ Checked against [Int] ⟧
  C → []            ⟦ Checked against [Int] ⟧
}

⟦ Nested list construction preserves shape ⟧
λwrap(xs:[Int])→[[Int]]=[xs]
```

**Example - Record Literals:**
```sigil
⟦ Record type provides context for empty list fields ⟧
t ParseState={
  code_lines:[String],
  list_items:[String],
  para_lines:[String]
}

λempty_state()→ParseState={
  code_lines:[],    ⟦ OK: infers [String] from ParseState.code_lines ⟧
  list_items:[],    ⟦ OK: infers [String] from ParseState.list_items ⟧
  para_lines:[]     ⟦ OK: infers [String] from ParseState.para_lines ⟧
}

⟦ Mixed empty and non-empty fields ⟧
λpartial_state()→ParseState={
  code_lines:["fn main() {}"],
  list_items:[],
  para_lines:["intro text"]
}
```

**Does NOT work when:**
- Standalone expression with no context: `c x=[]` (no type known)
- All pattern arms are empty and no function return type
- Nested expressions in synthesis mode without surrounding context

## Examples

### Valid Programs

```sigil
⟦ Factorial with pattern matching ⟧
λfactorial(n:Int)→Int match n{
  0→1|
  1→1|
  n→n*factorial(n-1)
}

⟦ GCD (multi-parameter recursion allowed) ⟧
λgcd(a:Int,b:Int)→Int match b{
  0→a|
  b→gcd(b,a%b)
}

⟦ List operations ⟧
λdoubleEvens(list:[Int])→[Int]=
  list↦λ(x:Int)→Int=x*2⊳λ(x:Int)→Bool=x%2=0
```

### Type Errors

```sigil
⟦ Error: Type mismatch ⟧
λbad()→Int="hello"
⟦ Error: Literal type mismatch: expected Int, got String ⟧

⟦ Error: Argument type mismatch ⟧
λid(x:Int)→Int=x
λmain()→String=id("hello")
⟦ Error: Argument 0 type mismatch: expected Int, got String ⟧

⟦ Error: Pattern match type mismatch ⟧
λneg(b:Bool)→Bool match b{5→false|_→true}
⟦ Error: Pattern type mismatch: expected Bool, got Int ⟧
```

## Summary

Bidirectional type checking is the right choice for Sigil because:

1. **Mandatory annotations** are a core principle → use a system designed for them
2. **Simpler implementation** → less code, fewer bugs, easier to maintain
3. **Better errors** → help developers understand and fix issues quickly
4. **More extensible** → natural framework for future features
5. **Perfect fit** → aligns with Sigil's canonical form philosophy

Like the canonical form refinement (blocking accumulators while allowing structural parameters), this is a case of **using the right tool for the job**.
