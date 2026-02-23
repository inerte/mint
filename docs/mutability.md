# Mint Mutability System

## Overview

Mint uses **immutable by default** with explicit `mut` annotations for mutability.

This prevents common logic errors at compile time while keeping syntax minimal—just one keyword instead of Rust's complex `&`, `&mut`, and lifetime system.

## Rules

### Rule 1: Immutable by Default

All values are immutable unless marked `mut`:

```mint
λsum(list:[ℤ])→ℤ=list⊕(λ(a:ℤ,x:ℤ)→ℤ=a+x)⊕0
⟦ list cannot be modified ⟧
```

### Rule 2: Explicit Mutability

Use `mut` keyword for mutable parameters:

```mint
λsort(list:mut [ℤ])→𝕌=quicksort_impl(list)
⟦ list will be modified in place ⟧
```

### Rule 3: No Aliasing of Mutables

Cannot create multiple references to mutable values:

```mint
⟦ ERROR: Cannot alias mutable ⟧
λbad(x:mut [ℤ])→𝕌≡{
  let y=x    ⟦ ERROR: Can't create alias ⟧
}

⟦ OK: Direct use ⟧
λgood(x:mut [ℤ])→𝕌=modify(x)
```

### Rule 4: Mutation Tracking

Functions that mutate use `!` suffix by convention:

```mint
λsort!(list:mut [ℤ])→𝕌=...     ⟦ Mutates in place ⟧
λsorted(list:[ℤ])→[ℤ]=...      ⟦ Returns new sorted list ⟧
```

## Examples

### Valid Code

```mint
# Immutable list operations
λdouble(list:[ℤ])→[ℤ]=list↦λ(x:ℤ)→ℤ=x*2

# Explicit mutation
λreverse!(list:mut [ℤ])→𝕌=reverse_impl!(list)

# Multiple immutable uses (OK)
λprocess(data:[ℤ])→ℤ≡{
  let sum=data⊕λ(a,x)→a+x⊕0
  let len=data⊕λ(a,_)→a+1⊕0
  sum/len
}
```

### Errors Prevented

```mint
# Error: Mutating immutable
λbad1(list:[ℤ])→𝕌=list↦!λ(x)→x*2
# Error: Cannot use mutating operation on immutable parameter

# Error: Aliasing mutable
λbad2(x:mut [ℤ])→𝕌≡{
  let y=x    # Error: Cannot create alias of mutable value 'x'
}

# Error: Passing immutable to mutable parameter
λbad3()→𝕌≡{
  let data=[1,2,3]
  sort!(data)    # Error: Cannot pass immutable 'data' to mutable parameter
}
```

## Why Mutability Checking?

### Problems It Prevents

**1. Accidental Mutation:**
```mint
# Without mutability checking:
λprocess(data:[ℤ])→[ℤ]≡{
  data↦!λ(x)→x*2;    # Oops! Modified input
  data
}

# With mutability checking:
# Compile error: Cannot mutate immutable parameter 'data'
```

**2. Aliasing Bugs:**
```mint
# Without mutability checking:
λbug(x:mut [ℤ])→𝕌≡{
  let y=x
  modify!(x)    # Modifies through x
  process(y)    # y changed too!
}

# With mutability checking:
# Compile error: Cannot create alias of mutable value 'x'
```

**3. Unclear Intent:**
```mint
# Without mutability checking:
λmysterious(data:[ℤ])→[ℤ]=???
# Does this modify data or return new list?

# With mutability checking:
λsorted(data:[ℤ])→[ℤ]=...        # Returns new list
λsort!(data:mut [ℤ])→𝕌=...       # Modifies in place
# Intent is crystal clear!
```

## Comparison to Other Languages

| Language | Approach | Complexity | Memory Safety |
|----------|----------|------------|---------------|
| **Rust** | Borrow checker with `&`, `&mut`, lifetimes | High | Yes (prevents use-after-free) |
| **TypeScript** | No mutability tracking | None | No |
| **Mint** | `mut` keyword with aliasing prevention | Low | No (relies on JS GC) |

### Why Not Full Borrow Checking?

**Rust needs borrow checking because:**
- Manual memory management
- Prevents use-after-free, double-free, data races
- Systems programming requirements

**Mint doesn't need it because:**
- Compiles to JavaScript (garbage collected)
- No manual memory management
- Goal is logic correctness, not memory safety

**Key Insight:**
Rust's borrow checker solves **memory safety**.
Mint's mutability checker solves **logic correctness**.

Different problems require different solutions.

## Design Philosophy

### Simplicity Over Complexity

**Instead of Rust's approach:**
```rust
fn process(data: &Vec<i32>) -> usize { ... }      // Immutable borrow
fn modify(data: &mut Vec<i32>) { ... }            // Mutable borrow
let x = &data;                                     // Borrow
let y = &mut data;                                 // Mutable borrow
```

**Mint's simpler approach:**
```mint
λprocess(data:[ℤ])→ℤ=...           # Immutable by default
λmodify(data:mut [ℤ])→𝕌=...        # Explicit mut
```

**Just ONE new keyword:** `mut`

### Canonical Forms

Mint enforces canonical forms—one way to do each thing.

**No tail-call optimization:**
```mint
# This style is BLOCKED:
λfactorial(n:ℤ,acc:ℤ)→ℤ≡n{
  0→acc|
  n→factorial(n-1,n*acc)
}

# Only primitive recursion allowed:
λfactorial(n:ℤ)→ℤ≡n{
  0→1|
  1→1|
  n→n*factorial(n-1)
}
```

Mutability fits this philosophy: either mutable or immutable, no third option.

## Error Messages

Mint provides clear, actionable error messages:

```
Mutability Error: Cannot create alias of mutable value 'x'

  12 | λbad(x:mut [ℤ])→𝕌≡{
  13 |   let y=x
       ^^^^^^^
```

```
Mutability Error: Cannot mutate immutable parameter 'list'

  5 | λprocess(list:[ℤ])→𝕌=list↦!λ(x)→x*2
                         ^^^^^^^^^^^^^^^^
```

## Future Enhancements

### Possible Extensions (Not Yet Implemented):

**1. Mutable let bindings:**
```mint
let mut counter=0
counter=counter+1  # Allow reassignment
```

**2. Interior mutability (Cell/RefCell):**
```mint
let cell=Cell(5)
cell.set(10)  # Controlled mutation
```

**3. Effect tracking:**
```mint
λread()→!IO 𝕊=...                # IO effect
λsort!(list:mut [ℤ])→!Mut 𝕌=...  # Mutation effect
```

These features may be added later, but the current system is focused and practical.

## Best Practices

### When to Use Mutable Parameters

**Use `mut` when:**
- Algorithm requires in-place modification for performance
- Operating on large data structures where copying is expensive
- Building APIs that match JavaScript conventions (e.g., Array.sort)

**Don't use `mut` when:**
- Default immutable approach is sufficient
- Function can return a new value instead
- Not sure—default to immutable

### Naming Conventions

**Mutating functions use `!` suffix:**
```mint
λsort!(list:mut [ℤ])→𝕌=...       # In-place sort
λsorted(list:[ℤ])→[ℤ]=...        # Returns sorted copy
```

**This makes intent obvious at call sites:**
```mint
sort!(data)      # I know data will be modified
let x=sorted(data)  # I know data is unchanged
```

## Summary

Mint's mutability system:
- ✅ Prevents mutation bugs at compile time
- ✅ Prevents aliasing bugs
- ✅ Makes intent clear (`mut` = will be modified)
- ✅ Minimal syntax (just one keyword)
- ✅ Practical for JavaScript target
- ✅ Fits canonical form philosophy

It's the sweet spot between TypeScript (no checking) and Rust (complex borrow checking).
