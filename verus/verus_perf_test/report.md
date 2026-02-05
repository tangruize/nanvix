# Verus Performance Analysis Report

## Overview

This report investigates a significant performance difference between two nearly identical Verus files: `perf_issue.rs` (fast) and `perf_issue_slow.rs` (slow). The only difference is the presence of a seemingly "redundant" `used()` spec function that acts as an alias for `num_allocated()`.

## Performance Comparison

| Metric | FAST (perf_issue.rs) | SLOW (perf_issue_slow.rs) | Difference |
|--------|---------------------|---------------------------|------------|
| Total Time | 746 ms | 1,325 ms | **+77.6%** |
| SMT Time | 240 ms | 825 ms | **+243.8%** |
| rlimit | 309,813 | 337,350 | +8.9% |
| Quantifier Instantiations | 550 | 609 | +10.7% |

## Code Difference

The critical difference lies in the `SlabView::free()` function definition:

### FAST Version (perf_issue.rs)
```rust
impl SlabView {
    pub open spec fn num_allocated(&self) -> int {
        self.allocated_blocks.len() as int
    }

    // This "redundant" alias is critical for SMT performance
    pub open spec fn used(&self) -> int {
        self.num_allocated()
    }

    pub open spec fn free(&self) -> int {
        self.capacity() - self.used()  // Uses the alias
    }
}
```

### SLOW Version (perf_issue_slow.rs)
```rust
impl SlabView {
    pub open spec fn num_allocated(&self) -> int {
        self.allocated_blocks.len() as int
    }

    // NOTE: used() function REMOVED

    pub open spec fn free(&self) -> int {
        self.capacity() - self.num_allocated()  // Direct call
    }
}
```

## Root Cause Analysis

### SMT-Level Impact

When Verus translates spec functions to SMT-LIB format, each function becomes an uninterpreted function with associated axioms and trigger patterns.

**FAST Version SMT Definition:**
```smt2
;; used() has its own trigger pattern
(forall ((self! Poly)) (!
    (= (used.? self!) (num_allocated.? self!))
    :pattern ((used.? self!))
))

;; free() uses used()
(forall ((self! Poly)) (!
    (= (free.? self!) (Sub (capacity.? self!) (used.? self!)))
    :pattern ((free.? self!))
))
```

**SLOW Version SMT Definition:**
```smt2
;; free() directly uses num_allocated()
(forall ((self! Poly)) (!
    (= (free.? self!) (Sub (capacity.? self!) (num_allocated.? self!)))
    :pattern ((free.? self!))
))
```

### Why This Matters

1. **Trigger Pattern Isolation**: The `used()` function has its own trigger pattern `:pattern ((used.? self!))`, which creates an **abstraction barrier** that prevents Z3 from over-expanding.

2. **E-Matching Chain Explosion**: When `free()` directly references `num_allocated()`, and `num_allocated()` further expands to `Set::len(allocated_blocks)`, Z3's E-matching algorithm triggers more axiom instantiations.

3. **Quantifier Instantiation Propagation**: The indirect layer through `used()` limits how far quantifier instantiations propagate through the proof.

### Z3 Statistics Comparison

From Z3 profiling:

| Z3 Metric | FAST | SLOW |
|-----------|------|------|
| quant-instantiations | 6,512 | 7,612 |
| decisions | 2,009 | 3,492 |
| propagations | 22,350 | 32,421 |
| conflicts | 777 | 815 |
| rlimit-count | 949,434 | 1,434,333 |

The SLOW version shows:
- **17% more** quantifier instantiations
- **74% more** decisions
- **45% more** propagations
- **51% higher** rlimit consumption

### Verus Profiler Analysis

Using `verus --profile-all`, we observed that the top quantifiers by cost are related to:

1. The `forall|j| ... self@.is_allocated(j) <==> self.index.is_bit_set(...)` invariant
2. The `allocated_blocks_in_range` property
3. Frame conditions in allocate functions

In the SLOW version, these quantifiers are instantiated more frequently because the direct use of `num_allocated()` in `free()` creates additional matching opportunities for Set-related axioms.

## Conclusion

### The Problem

Removing what appears to be a "redundant" spec function alias (`used()`) causes a **77.6% increase in verification time** due to SMT solver performance degradation.

### The Mechanism

The `used()` function acts as a **function abstraction barrier** that:
- Isolates trigger patterns
- Limits quantifier instantiation propagation
- Reduces E-matching search space
- Prevents cascading axiom applications

### Best Practice Recommendation

In Verus, adding "seemingly redundant" spec function aliases can **significantly improve verification performance** when:

1. The underlying function involves complex types (like `Set::len`)
2. The function is used in multiple contexts (invariants, postconditions, lemmas)
3. The function participates in quantified formulas

**Pattern to follow:**
```rust
// Instead of using complex_operation() directly everywhere:
pub open spec fn complex_operation(&self) -> int {
    self.set_field.len() as int  // Involves Set axioms
}

// Add an alias for use in other spec functions:
pub open spec fn simple_alias(&self) -> int {
    self.complex_operation()
}

// Use the alias in derived specs:
pub open spec fn derived_property(&self) -> int {
    self.capacity() - self.simple_alias()  // Insulated from Set axioms
}
```

This pattern creates intentional abstraction boundaries that help the SMT solver reason more efficiently.

## Appendix: Reproduction Commands

```bash
# Run fast version with timing
verus --crate-type lib --time-expanded perf_issue.rs

# Run slow version with timing
verus --crate-type lib --time-expanded perf_issue_slow.rs

# Run with full profiling
verus --crate-type lib --profile-all --verify-root perf_issue.rs
verus --crate-type lib --profile-all --verify-root perf_issue_slow.rs

# Generate SMT logs for analysis
verus --crate-type lib --log-all --log-dir ./logs perf_issue.rs
```

---

## Discussion: Is This a Bug in Verus?

### The Gray Area

This issue exists in a gray area between "expected behavior" and "bug/deficiency":

| Perspective | Explanation |
|-------------|-------------|
| **Not a Bug** | Z3's E-matching algorithm is inherently sensitive to formula structure. This is a fundamental limitation of all SMT-based verification tools, not specific to Verus. |
| **Is a Bug/Deficiency** | Verus could theoretically detect these patterns and automatically insert abstraction barriers, or provide better trigger control mechanisms. The fact that users need to understand SMT internals to write performant specs is a usability issue. |
| **Design Trade-off** | Verus chose a "transparent mapping" to SMT, giving users more control but also exposing underlying complexity. This is a deliberate design choice with both benefits and drawbacks. |

### Is This a Best Practice?

**Not really.** This is more of a **workaround** than a best practice. In SMT-based verification, this technique is known as "trigger engineering" or "quantifier instantiation control," but it:

- Requires users to understand SMT solver internals
- Is unintuitive and hard to predict when needed
- Adds code redundancy and maintenance burden
- Can make code harder to read and understand

A true best practice would be one that doesn't require users to have deep knowledge of the underlying solver mechanics.

### How Do Other Verification Tools Handle This?

| Tool | Approach |
|------|----------|
| **Dafny** | Provides `{:trigger}` attributes for manual trigger control. Also has `{:opaque}` to hide function definitions from the solver until explicitly revealed. Similar issues exist but with explicit user controls. |
| **F\*** | Uses SMT patterns combined with fuel-based unfolding. Provides `unfold` and `fold` tactics. Has similar trigger sensitivity issues. |
| **Why3** | Allows explicit trigger annotations in specifications. Provides transformation strategies to control proof search. |
| **Coq/Lean** | Uses tactics-based proving, completely different approach. No E-matching issues because proofs are constructed step-by-step, not searched. Trade-off is more manual proof effort. |
| **Liquid Haskell** | Uses refinement types with SMT. Has similar issues but the type system constrains the problem space. |

### What Could Verus Do Better?

1. **Automatic Abstraction Insertion**: The Verus compiler could detect when complex operations (like `Set::len`, recursive functions, etc.) are used in multiple places and automatically generate intermediate abstraction layers.

2. **Better Diagnostics**: When verification performance degrades, Verus could analyze the SMT logs and suggest potential causes, such as "Consider adding an alias for `num_allocated()` to reduce quantifier instantiation."

3. **Trigger Hints API**: Provide a more user-friendly API to control quantifier instantiation without requiring users to understand SMT internals. For example:
   ```rust
   #[verifier::abstract_barrier]  // Hypothetical attribute
   pub open spec fn used(&self) -> int {
       self.num_allocated()
   }
   ```

4. **Fuel/Opacity Controls**: More fine-grained controls over when and how deeply function definitions are unfolded, similar to Dafny's `{:opaque}` or F\*'s fuel system.

5. **Profile-Guided Optimization**: Use profiling data to automatically identify and suggest optimizations for slow-verifying functions.

### Should This Be Transparent to Users?

**Ideally, yes.** Users should be able to write natural, clean specifications without worrying about SMT encoding details. The current situation where:

- A "redundant" alias provides 77% speedup
- Users need to understand E-matching and trigger patterns
- Performance is unpredictable and sensitive to small changes

...represents a significant usability barrier for Verus adoption.

### Recommendation

This issue is worth reporting to the Verus team as it demonstrates a real user experience problem. A GitHub issue at [verus-lang/verus](https://github.com/verus-lang/verus) could include:

1. This minimal reproducible example
2. The performance measurements
3. The SMT-level analysis showing why the alias helps
4. A discussion of potential compiler-level solutions

The Verus team may already be aware of this class of issues, but concrete examples with analysis are valuable for prioritizing improvements.

### Final Thoughts

The tension between "transparent SMT encoding" and "user-friendly verification" is a fundamental challenge in SMT-based tools. Verus is still a relatively young project, and improvements in this area could significantly enhance its usability. Until then, users working with complex specifications involving sets, sequences, and quantifiers should be aware that:

1. Function abstraction can dramatically impact performance
2. The `--profile-all` flag is essential for diagnosing slow verification
3. Sometimes "redundant" code is actually performance-critical infrastructure
