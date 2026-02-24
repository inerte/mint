# Mint Syntax Reference

This is a **canonical syntax reference** for Mint.

It exists for:
- reviewing generated Sigil code
- building tools (compiler, LSP, editors)
- grounding AI prompts against the current language surface

It is not a style guide for multiple alternatives, because Mint intentionally has one canonical form.

## Scope

This document covers the current syntax surface in this repo:
- declarations (`export`, `λ`, `t`, `c`, `i`, `e`, `test`)
- expressions and pattern matching
- built-in list operators (`↦`, `⊳`, `⊕`, `⧺`)
- effects, mocks, and test syntax
- comments

For formatting/canonical whitespace rules, see:
- `docs/CANONICAL_FORMS.md`
- `docs/CANONICAL_ENFORCEMENT.md`

## Source Files

- Mint source files use `.sigil`
- Files should end with a final newline
- Tests live in project `./tests`
- App/library code lives in project `./src`

## Comments

Mint uses one comment syntax only:

```mint
⟦ This is a comment ⟧

λfactorial(n:ℤ)→ℤ≡n{
  0→1|  ⟦ inline comment ⟧
  n→n*factorial(n-1)
}
```

- `#`, `//`, and `/* ... */` are not Mint comments

## Declarations

## Function declarations

```mint
λadd(x:ℤ,y:ℤ)→ℤ=x+y
```

Rules:
- function name is required
- parameter types are required
- return type is required
- `=` is required for regular expression bodies
- `=` is omitted when body starts with match (`≡...`)

Match-body form:

```mint
λfactorial(n:ℤ)→ℤ≡n{
  0→1|
  1→1|
  n→n*factorial(n-1)
}
```

## Effectful function declarations

Effects are declared between `→` and the return type:

```mint
λfetchUser(id:ℤ)→!Network 𝕊=axios.get("https://api.example.com/users/"+id)
λmain()→!IO 𝕌=console.log("hello")
```

## Mockable function declarations (tests)

```mint
mockable λfetchUser(id:ℤ)→!Network 𝕊="real"
```

- `mockable` is only valid on functions
- mockable functions must be effectful
- mock targets are used by `with_mock(...) { ... }` in tests

## Exported declarations (explicit)

Only explicitly exported top-level declarations are visible to other Mint modules.

Canonical export forms:

```mint
export λdouble(x:ℤ)→ℤ=x*2
export t Todo={id:ℤ,text:𝕊,done:𝔹}
export c VERSION:𝕊="0.1"
```

Notes:
- `export` applies to top-level `λ`, `t`, and `c`
- `export test`, `export i ...`, and `export e ...` are invalid

## Type declarations (`t`)

## Product type (record)

```mint
t User={id:ℤ,name:𝕊,active:𝔹}
```

## Sum type (ADT)

```mint
t Color=Red|Green|Blue
t Option[T]=Some(T)|None
t Result[T,E]=Ok(T)|Err(E)
```

Constructor usage:

```mint
Red()
Some(42)
Err("not found")
```

## Constants (`c`)

```mint
c ANSWER:ℤ=42
c GREETING:𝕊="hello"
```

## Imports and externs

## Mint imports (`i`)

Mint-to-Mint imports are namespace imports only.

```mint
i src/todo-domain
i stdlib/list_utils
```

Use imported members with fully qualified namespace access:

```mint
src/todo-domain.completedCount(todos)
stdlib/list_utils.len([1,2,3])
```

Canonical Mint import roots:
- `src/...`
- `stdlib/...`

Not supported:
- `i ./...`
- `i ../...`
- selective imports
- aliasing

## External module interop (`e`)

```mint
e console
e fs/promises
e react-dom/client
```

Use with namespace member access:

```mint
console.log("hello")
fs/promises.writeFile("x.txt","data")
react-dom/client.createRoot(root)
```

## Tests

Tests are first-class declarations and must live under `./tests`.

## Basic test

```mint
test "adds numbers" {
  1+1=2
}
```

## Effectful test

```mint
e console

test "logs" →!IO {
  console.log("x")=()
}
```

## Mocked test

```mint
mockable λfetchUser(id:ℤ)→!Network 𝕊="real"

test "mocked fetch" →!Network {
  with_mock(fetchUser,λ(id:ℤ)→!Network 𝕊="mocked"){
    fetchUser(1)="mocked"
  }
}
```

## Expressions

## Literals and primitives

Primitive types:
- `ℤ` integer
- `ℝ` float
- `𝔹` boolean
- `𝕊` string
- `𝕌` unit

Boolean values:
- `⊤`
- `⊥`

Examples:

```mint
42
3.14
"hello"
⊤
⊥
()
```

## Variables and calls

```mint
add(1,2)
factorial(n-1)
```

## Pattern matching (`≡`)

```mint
≡value{
  pattern1→result1|
  pattern2→result2|
  _→defaultResult
}
```

Examples:

```mint
λsign(n:ℤ)→𝕊≡n{
  0→"zero"|
  n→"non-zero"
}

λdescribeBoth(a:𝔹,b:𝔹)→𝕊≡(a,b){
  (⊤,⊤)→"both"|
  (⊤,⊥)→"left"|
  (⊥,⊤)→"right"|
  (⊥,⊥)→"none"
}
```

## Lists

List literals:

```mint
[]
[1,2,3]
["a","b","c"]
```

List patterns:

```mint
≡xs{
  []→0|
  [x,.rest]→1
}
```

Concatenation:

```mint
"ab"++"cd"      ⟦ string concat only ⟧
[1,2]⧺[3,4]     ⟦ list concat only ⟧
```

## Records and field access

```mint
User{id:1,name:"A",active:⊤}
todo.done
todo.text
```

## Indexing

```mint
xs[0]
```

## Operators

## Arithmetic

```mint
a+b
a-b
a*b
a/b
a%b
```

## Comparison

```mint
a=b
a≠b
a<b
a>b
a≤b
a≥b
```

## Logical

```mint
a∧b
a∨b
¬a
```

## Built-in list operators (language constructs)

Map:

```mint
[1,2,3]↦λ(x:ℤ)→ℤ=x*2
```

Filter:

```mint
[1,2,3,4]⊳λ(x:ℤ)→𝔹=x%2=0
```

Fold:

```mint
[1,2,3]⊕λ(acc:ℤ,x:ℤ)→ℤ=acc+x⊕0
```

## Lambdas

Lambda parameters and return type annotations are required.

```mint
λ(x:ℤ)→ℤ=x*2
λ(todo:Todo)→𝔹=¬todo.done
```

Effectful lambda:

```mint
λ(msg:𝕊)→!IO 𝕌=console.log(msg)
```

## Canonical Formatting Reminders

- No trailing whitespace
- Max one blank line
- Final newline required
- No tabs
- `λf()→T=...` for regular bodies
- `λf()→T≡...` for match bodies (no `=`)

See `docs/CANONICAL_FORMS.md` for the full enforced rules.
