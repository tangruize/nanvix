# Verus Macro-to-Annotation Style Translator

## Overview

`translate_verus_to_annotation.py` converts Verus `verus!{}` macro-style code to
the annotation-based style (`#[verus_spec]`, `#[verus_verify]`, `proof!{}`,
`proof_decl!{}`, `proof_with!{}`). This brings Nanvix's verification code closer
to the style used by the [SVSM project](https://github.com/coconut-svsm/svsm),
where verified code blends more naturally with standard Rust.

### Why annotation style?

| Concern | `verus!{}` macro style | Annotation style |
|---|---|---|
| Code readability | Entire file wrapped in `verus!{}` | Standard Rust with attributes |
| IDE support | Limited (custom grammar) | Standard rust-analyzer works |
| API impact | `Tracked<T>` params in public signatures | Hidden via `with` clause |
| Build dependency | Hard dependency on verus | Can stub out for non-verus builds |
| Incremental adoption | All-or-nothing per file | Function-by-function |

### Reference documents

- [Verus Discussion #1716: Attribute-based macros](https://github.com/verus-lang/verus/discussions/1716)
- [Verus Discussion #1843: Limitations and future work](https://github.com/verus-lang/verus/discussions/1843)
- [Verus syntax_attr.rs test suite](https://github.com/verus-lang/verus/blob/main/source/rust_verify_test/tests/syntax_attr.rs)
- SVSM reference: `~/svsm/kernel/src/mm/alloc.rs`

---

## Quick Start

### Prerequisites

```bash
pip install tree-sitter tree-sitter-verus   # or: cd ~/nanvix/tree-sitter-verus && pip install -e .
```

### Basic usage

```bash
# Preview translation (stdout)
python3 ~/nanvix/scripts/translate_verus_to_annotation.py src/libs/bitmap/src/lib.rs

# Write to a different file
python3 ~/nanvix/scripts/translate_verus_to_annotation.py src/libs/bitmap/src/lib.rs -o /tmp/bitmap_new.rs

# Edit in place
python3 ~/nanvix/scripts/translate_verus_to_annotation.py src/libs/bitmap/src/lib.rs --in-place
```

### After translation

1. Run `./z build -- all BUILD_OPT=no MACHINE=microvm RELEASE=no` to check standard compilation.
2. Run `make verify` to check Verus verification.
3. Update any callers if function signatures changed (e.g., `Tracked` params removed).

---

## What Gets Translated

### 1. Struct definitions

**Before:**
```rust
verus! {
#[verifier::external_derive]
#[derive(Debug)]
#[verifier::ext_equal]
pub struct Bitmap { ... }
}
```

**After:**
```rust
#[derive(Debug)]
#[cfg_attr(verus_keep_ghost, verifier::ext_equal)]
#[verus_verify]
pub struct Bitmap { ... }
```

### 2. Exec functions (no Tracked params/returns)

**Before (inside `verus!{}`):**
```rust
verus! {
pub fn new(number_of_bits: usize) -> (result: Result<Self, Error>)
    ensures
        result matches Ok(bitmap) ==> { ... },
{
    proof { Self::lemma_new_bitmap_inv(&result); }
    Ok(result)
}
}
```

**After:**
```rust
#[verus_verify]
impl Bitmap {
    #[verus_spec(result =>
        ensures
            result matches Ok(bitmap) ==> { ... }
    )]
    pub fn new(number_of_bits: usize) -> Result<Self, Error> {
        proof! { Self::lemma_new_bitmap_inv(&result); }
        Ok(result)
    }
}
```

Key changes:
- `impl` block gets `#[verus_verify]`
- `(result: Type)` named return → plain `Type` with `result =>` in `#[verus_spec]`
- `proof { }` → `proof! { }`
- `let ghost x = ...;` → `proof_decl! { let ghost x = ...; }`

### 3. Loop annotations (invariant, decreases)

**Before (inside `verus!{}`):**
```rust
for i in 0..n
    invariant
        some_invariant(i),
    decreases n - i,
{
    // body
}
```

**After:**
```rust
#[cfg_attr(verus_keep_ghost_body, verus_spec(
    invariant some_invariant(i),
    decreases n - i
))]
for i in 0..n
{
    // body
}
```

> **Note:** The `cfg_attr(verus_keep_ghost_body, ...)` wrapping ensures the attribute
> is only active during verus verification builds. Standard Rust builds skip it entirely.

### 4. Spec/proof functions and impl View

These stay in a residual `verus!{}` block because they use Verus-specific syntax
(spec fn, proof fn, closed/open modifiers) that has no annotation-style equivalent.

```rust
verus! {

impl View for Bitmap {
    type V = BitmapView;
    closed spec fn view(&self) -> BitmapView { ... }
}

impl Bitmap {
    spec fn bit_at(bytes: Seq<u8>, bit_index: int) -> bool { ... }
    pub open spec fn inv(&self) -> bool { ... }
}

} // verus!
```

---

## What Does NOT Get Translated (Current Limitations)

The script keeps certain functions inside `verus!{}` when the annotation style
is not fully supported. Below are the **specific limitations**, categorized as
either **verus toolchain limitations** (require upstream fix) or **code migration
requirements** (can be fixed in nanvix).

---

### ⚠️ TOOLCHAIN LIMITATION 1: `invariant_except_break` in loop annotations

**Classification: verus `builtin_macros` proc macro bug**

**Symptom:** `error: unexpected token` at `invariant` keyword when using
`#[verus_spec(invariant_except_break ..., invariant ...)]` on a loop.

**Root cause:** The `verus_spec` proc macro's loop annotation parser
(`verus_builtin_macros`) does not correctly handle the transition from
`invariant_except_break` items to `invariant` items. The parser reads
`invariant_except_break expr1, expr2,` and then encounters `invariant`
which it tries to parse as another expression rather than recognizing it
as a new clause keyword.

**Affected code (bitmap `alloc_range`):**
```rust
// This WORKS in verus!{} but FAILS in #[verus_spec] on a loop:
while offset < size
    invariant_except_break
        start == start_before_inner,
        free,
    invariant
        self.alloc_range_probe_loop_invariant(...)
    ensures
        self.alloc_range_probe_loop_ensures(...)
    decreases size - offset,
{ ... }
```

**Current workaround:** Functions with `invariant_except_break` loops stay in `verus!{}`.

**Evidence:**
- SVSM never uses `invariant_except_break` in annotation style.
- The verus test `syntax_attr.rs` tests `invariant_except_break` on `loop` blocks
  but only with simple expressions (e.g. `i <= 9`), not with complex multi-clause
  combinations on `while` loops.
- The string `"invariant_except_breakinvariantouter attributes only allowed on function's ensures"`
  in `libverus_builtin_macros.so` suggests incomplete support.

**Suggested fix (in verus):** In `verus_builtin_macros/src/attr_rewrite.rs` (or
the equivalent loop-spec parser), update the token stream parser to recognize
`invariant_except_break`, `invariant`, `ensures`, and `decreases` as clause-starting
keywords when parsing `#[verus_spec(...)]` on loop expressions, regardless of order
and with multi-line bodies. The test to add:

```rust
#[verus_spec]
fn test_while_complex_clauses() {
    let mut start: usize = 0;
    let mut free: bool = true;
    #[verus_spec(
        invariant_except_break
            start == 0,
            free,
        invariant
            start <= 10,
        ensures
            !free || start == 10,
        decreases 10 - start
    )]
    while start < 10 {
        if some_condition() { free = false; break; }
        start = start + 1;
    }
}
```

**Verus version tested:** `0.2026.02.06` (commit `4a2b93e`), `verus_builtin_macros 0.0.0-2026-02-08-0120`

---

### ⚠️ TOOLCHAIN LIMITATION 2: Tracked output (`with -> Tracked<T>`) with early returns

**Classification: verus `builtin_macros` proc macro limitation**

**Symptom:** `error[E0308]: mismatched types` — function body has
`return Err(e)` but return type was rewritten to `(Result<T, E>, Tracked<U>)`.

**Root cause:** When `#[verus_spec(with ... -> output: Tracked<T>)]` is used,
the proc macro rewrites the function's return type from `R` to `(R, Tracked<T>)`.
The `proof_with!(|= Tracked(x))` before the final expression tells the macro
to wrap that expression into a tuple. However, **early `return` statements are
NOT rewritten** — they still return bare `R` instead of `(R, Tracked<T>)`.

**Affected code (slab `allocate`, `from_raw_parts`):**
```rust
// The #[verus_spec] generates verus_verified_allocate with return type
// (Result<(*mut u8, Tracked<PointsToRaw>), Error>)
// But the early return is NOT wrapped:
pub fn allocate(&mut self) -> Result<(*mut u8, Tracked<PointsToRaw>), Error> {
    match self.index.alloc() {
        Err(e) => return Err(e),  // ← NOT wrapped in tuple by proc macro
        Ok(b) => { ... }
    };
    proof_with!(|= Tracked(block_perm));
    Ok(block_addr)  // ← this IS wrapped
}
```

**Evidence:**
- SVSM **never uses** `with -> output: Tracked<T>` in any file.
- The verus test `syntax_attr.rs` only tests tracked output with
  single-exit-point functions (no early `return`).
- The error message `"with ghost inputs/outputs cannot be applied to a non-call expression"`
  in the macro suggests `proof_with!(|= ...)` is designed for expressions, not for
  transforming `return` statements.

**Current workaround:** Functions with `Tracked<T>` in their return type stay
in `verus!{}`. The `Tracked` stays in the return type.

**Suggested fix (in verus):** The `verus_spec` proc macro in
`verus_builtin_macros` should scan the function body for all `return` statements
and wrap them: `return expr` → `return (expr, Tracked::assume_new())` when the
function has `with -> output`. This ensures all exit paths return the expected
tuple type. Alternatively, a new `proof_return!(|= Tracked(x)) return Err(e);`
syntax could be introduced.

**Verus version tested:** `0.2026.02.06` (commit `4a2b93e`)

---

### 🔧 CODE LIMITATION 3: `verus!{}` callers cannot call annotation-style functions with `with` Tracked params

**Classification: CODE migration requirement (NOT a toolchain limitation)**

**Symptom:** `error[E0061]: this method takes N arguments but M arguments were supplied`
when verus-verified test code calls a function whose Tracked params were moved to `with`.

**Root cause:** This is NOT a toolchain limitation. The verus toolchain fully supports
calling annotation-style functions with Tracked `with` params. The mechanism is:

1. `#[verus_spec(with Tracked(x): Tracked<T>)]` on the callee generates two function
   variants: `verus_verified_{name}(args, Tracked(x))` and `{name}(args)` (stub).
2. The caller must use `proof_with!(Tracked(val));` before the call, and the caller
   function must have `#[verus_spec]` so the proc macro processes the `proof_with!`.
3. `proof_with!` is a **placeholder macro** that expands to nothing on its own. Only
   `#[verus_spec]` on the enclosing function gives it meaning.

**Why current tests fail:** The test file (`lib.test.rs`) is included inside a `verus!{}`
block. Inside `verus!{}`, there is no `#[verus_spec]` processing — `proof_with!` expands
to nothing, and the call uses the stub signature (too few args).

**This is fully fixable in our code:**

1. Move test functions out of `verus!{}` into annotation style:
   ```rust
   // Before (inside verus!{}):
   fn test_deallocate_verified(...) {
       slab.deallocate(ptr, Tracked(block_perm), Tracked(&mut slab_perms))
   }

   // After (annotation style):
   #[verus_spec]
   fn test_deallocate_verified(...) {
       proof_with!(Tracked(block_perm), Tracked(&mut slab_perms));
       slab.deallocate(ptr);
   }
   ```

2. Update `kheap.rs` callers — just remove `Tracked::assume_new()` args:
   ```rust
   // Before:
   self.slab.deallocate(ptr, Tracked::assume_new(), Tracked::assume_new())
   // After:
   self.slab.deallocate(ptr)
   ```

**Verified experimentally:** When we used `proof_with!` inside `verus!{}`, the verus
compiler DID recognize the `with` clause (79 verified, 5 precondition failures from
the stub's `requires false`). The compilation and routing worked — only the verification
conditions failed because `proof_with!` wasn't processed by any enclosing `#[verus_spec]`.

**Effort estimate:** Migrate `lib.test.rs` from `verus!{}` style to annotation style.
The test functions become `#[verus_spec]` functions, and all Tracked-param calls use
`proof_with!`. This is a mechanical transformation similar to what the translator
script does for exec functions.

---

### 🔧 CODE LIMITATION 4: `#![feature(proc_macro_hygiene)]` required for loop annotations

**Classification: Rust nightly feature requirement (NOT a verus limitation)**

**Symptom:** `error[E0658]: custom attributes cannot be applied to expressions`

**Root cause:** Rust requires `#![feature(proc_macro_hygiene)]` to allow
proc macro attributes on expressions (like `for`/`while`/`loop`).

**Current handling:** The script auto-inserts
`#![cfg_attr(verus_keep_ghost, feature(proc_macro_hygiene))]` at the top of
files that use loop annotations. This is gated on `verus_keep_ghost` so it
only activates during verus builds. Same pattern as SVSM.

---

## Summary: Toolchain vs Code Limitations

| # | Type | Issue | What blocks | Fix responsibility |
|---|---|---|---|---|
| 1 | ⚠️ **Toolchain** | `invariant_except_break` parsing bug | bitmap `alloc_range` | verus upstream |
| 2 | ⚠️ **Toolchain** | `with -> output` + early returns | slab `allocate`, `from_raw_parts` | verus upstream |
| 3 | 🔧 **Code** | Test files in `verus!{}` can't call annotation fns | slab `deallocate` | nanvix migration |
| 4 | 🔧 **Code** | `proc_macro_hygiene` feature needed | Loop annotations | Auto-handled by script |

---

## Translation Rules Reference

| verus!{} syntax | Annotation style | Notes |
|---|---|---|
| `verus! { fn f() ... }` | `#[verus_spec] fn f() ...` | Function-level |
| `fn f() -> (ret: T)` | `fn f() -> T` + `ret =>` in spec | Named return |
| `requires R` | `requires R` in `#[verus_spec]` | Unchanged content |
| `ensures E` | `ensures E` in `#[verus_spec]` | Unchanged content |
| `decreases D` | `decreases D` in `#[verus_spec]` | Unchanged content |
| `Tracked(x): Tracked<T>` param | `with Tracked(x): Tracked<T>` | Input only* |
| `proof { stmts }` | `proof! { stmts }` | In function body |
| `let ghost x = e;` | `proof_decl! { let ghost x = e; }` | Function-scoped |
| `let tracked x = e;` | `proof_decl! { let tracked x = e; }` | Function-scoped |
| `for i in r invariant I { }` | `#[cfg_attr(verus_keep_ghost_body, verus_spec(invariant I))] for i in r { }` | Simple invariants only |
| `while c invariant I { }` | `#[cfg_attr(verus_keep_ghost_body, verus_spec(invariant I))] while c { }` | Simple invariants only |
| `#[verifier::external_derive]` struct | `#[verus_verify]` struct | On struct |
| `spec fn`, `proof fn` | Stays in `verus!{}` | No annotation equivalent |
| `impl View for T` | Stays in `verus!{}` | No annotation equivalent |

*Tracked output (`with -> Tracked<T>`) is not used due to Limitation 2.

---

## Architecture of the Script

```
translate_verus_to_annotation.py
├── VerusTranslator              Main class
│   ├── translate()              Entry: find verus!{} blocks, process, reassemble
│   ├── _process_verus_block()   Classify items, separate exec vs ghost
│   ├── _classify()              Determine item type (struct/exec_fn/spec_fn/impl/...)
│   ├── _transform_struct()      Add #[verus_verify], handle verifier attrs
│   ├── _transform_exec_impl()   Transform impl block with exec methods
│   ├── _transform_exec_fn()     Core: build #[verus_spec] + transform body
│   │   ├── _extract_params()    Separate normal vs Tracked params
│   │   ├── _extract_return_info()  Parse return type for Tracked elements
│   │   ├── _extract_specs()     Get requires/ensures/decreases
│   │   ├── _build_verus_spec_attr()  Format the #[verus_spec(...)] attribute
│   │   ├── _build_fn_sig()      Clean function signature
│   │   └── _transform_body()    proof→proof!, ghost lets→proof_decl!, loops
│   │       ├── _collect_body_transforms()  Walk AST for transformations
│   │       └── _transform_loop()           Loop annotation rewriting
│   └── Safety checks
│       ├── _has_unsupported_loop_clauses()    invariant_except_break detection
│       └── _has_tracked_params_or_return()    Tracked in signature detection
└── Helpers: nt(), fc(), fcs(), indent_at(), make_parser()
```

### Decision flow for each function

```
Is it a spec/proof fn?
  → YES: Keep in verus!{}

Is it an exec fn with Tracked params or return?
  → YES: Keep in verus!{}

Does it have loops with invariant_except_break or loop-level ensures?
  → YES: Keep in verus!{}

Otherwise:
  → Translate to annotation style
```

---

## Testing

```bash
# Translate and verify bitmap
python3 ~/nanvix/scripts/translate_verus_to_annotation.py src/libs/bitmap/src/lib.rs --in-place
./z build -- all BUILD_OPT=no MACHINE=microvm RELEASE=no  # standard build
make verify                                                 # verus verification

# Translate and verify slab
python3 ~/nanvix/scripts/translate_verus_to_annotation.py src/libs/slab/src/lib.rs --in-place
./z build -- all BUILD_OPT=no MACHINE=microvm RELEASE=no
make verify

# Idempotency check (running twice produces same output)
python3 ~/nanvix/scripts/translate_verus_to_annotation.py src/libs/bitmap/src/lib.rs | \
  diff - src/libs/bitmap/src/lib.rs
```

### Current verification results

| Crate | Verified | Errors | Notes |
|---|---|---|---|
| bitmap | 69 | 0 | 7 exec functions translated, 1 kept in verus (alloc_range) |
| slab | 84 | 0 | struct translated, all exec functions kept in verus (Tracked) |

---

## Roadmap: Enabling Full Tracked Support

To translate slab's exec functions (with Tracked params) to annotation style:

### Step 1: Migrate test files to annotation style
Convert `lib.test.rs` from `verus!{}` to use `#[verus_spec]` + `proof_with!()`.
This removes the caller-side incompatibility (Limitation 3).

### Step 2: Move Tracked-input-only functions out
Functions like `deallocate` (Tracked inputs, no Tracked return) can then use
`#[verus_spec(with Tracked(...): Tracked<T>)]`. Callers in tests use
`proof_with!(Tracked(...))` before the call.

### Step 3: Fix `invariant_except_break` in verus proc macro
File an issue or PR on [verus-lang/verus](https://github.com/verus-lang/verus)
to support `invariant_except_break` in `#[verus_spec]` on loop expressions.
Then `alloc_range` can be translated.

### Step 4: Fix tracked output with early returns
Either fix the verus proc macro to handle early returns with `with -> output`,
or refactor functions like `allocate`/`from_raw_parts` to use a single exit point
pattern (accumulate result, return at end).
