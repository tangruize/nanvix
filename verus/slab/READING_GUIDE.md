# Slab Allocator Proof Reading Guide

This guide helps you navigate and understand the 3200+ line `slab_core.rs` verification file.

## File Structure Overview

The file is organized into logical sections. Here's a roadmap:

```
Lines 1-30:      Imports and dependencies
Lines 31-80:     Slab struct definition (executable)
Lines 81-350:    SlabView spec struct and helper specs
Lines 351-700:   Proof lemmas for slab properties
Lines 701-1100:  Slab invariant and View implementation
Lines 1101-1700: from_raw_parts constructor with proofs
Lines 1701-2200: allocate() function with proofs
Lines 2201-2700: deallocate() function with proofs
Lines 2701-3000: new() constructor with proofs
Lines 3001-3263: Test functions (proof-based tests)
```

## Key Concepts to Understand First

### 1. The Abstraction Layers

```
┌─────────────────────────────────────────────────────────┐
│  SlabView (Spec Level - Ghost State)                    │
│  - allocated_blocks: Set<int>  (abstract set of blocks) │
│  - num_data_blocks, block_size, data_addr (pure specs)  │
└─────────────────────────────────────────────────────────┘
                          ↑ view()
┌─────────────────────────────────────────────────────────┐
│  Slab (Exec Level - Real State)                         │
│  - index: Bitmap (tracks allocation via bits)           │
│  - data_addr, num_index_blocks, num_data_blocks, etc.   │
└─────────────────────────────────────────────────────────┘
                          ↓ uses
┌─────────────────────────────────────────────────────────┐
│  Bitmap / RawArray (Trusted Dependencies)               │
│  - Axiomatized with external_body                       │
│  - Trusted specs define their behavior                  │
└─────────────────────────────────────────────────────────┘
```

### 2. The Invariant (`inv()`)

The invariant is the heart of the verification. Find it around line 700:

```rust
pub open spec fn inv(&self) -> bool {
    // Basic validity
    self.block_size > 0 &&
    self.num_data_blocks > 0 &&
    // Power-of-two and alignment (Issue 3 fix)
    is_power_of_two(self.block_size as int) &&
    self.data_addr % self.block_size == 0 &&
    // Buffer bounds tracking (Issue 6 fix)
    self.base_addr <= self.data_addr &&
    self.data_addr + self.num_data_blocks * self.block_size <= self.base_addr + self.total_len &&
    // Bitmap has correct size
    self.index@.len() >= self.num_index_blocks + self.num_data_blocks &&
    // ... more constraints
}
```

**Reading tip**: The invariant defines what makes a Slab "valid". Every function:
- **Requires** `self.inv()` (precondition)
- **Ensures** `self.inv()` is preserved (postcondition)

### 3. The View Function

Around line 750, the `view()` function connects concrete state to abstract state:

```rust
impl View for Slab {
    type V = SlabView;
    
    closed spec fn view(&self) -> SlabView {
        SlabView {
            allocated_blocks: Set::new(|i: int|
                0 <= i < self.num_data_blocks as int &&
                self.index.is_bit_set(self.num_index_blocks as int + i)
            ),
            num_data_blocks: self.num_data_blocks as int,
            // ...
        }
    }
}
```

**Key insight**: `allocated_blocks` is defined by which bits are set in the bitmap!

## How to Read Each Major Function

### Pattern: Pre/Post + Body + Proof

Each verified function follows this pattern:

```rust
pub fn function_name(&mut self, args...)
    requires
        // What must be true BEFORE calling
        old(self).inv(),
        // ... more preconditions
    ensures
        // What is guaranteed AFTER calling
        self.inv(),
        // ... more postconditions
{
    // EXECUTABLE CODE
    let result = do_something();
    
    // PROOF ANNOTATIONS (ghost code)
    proof {
        // Help Verus connect dots
        lemma_some_property();
        assert(some_intermediate_fact);
    }
    
    result
}
```

### Reading `allocate()` (Lines ~1701-2200)

1. **Start with the contract** (requires/ensures):
   ```rust
   requires old(self).inv()
   ensures 
       self.inv(),  // invariant preserved
       result is Ok ==> old(self)@.can_allocate(),  // success means was allocatable
       result is Ok ==> self@.is_allocated(...)     // block is now allocated
   ```

2. **Skim the body** - it's mostly:
   - Call `self.index.alloc()` to find a free bit
   - Convert bit index to address
   - Return the address

3. **Study the proof blocks** when you need to understand WHY something holds:
   ```rust
   proof {
       // This proof connects bitmap allocation to SlabView
       assert(self@.allocated_blocks.contains(block_idx));
   }
   ```

### Reading `deallocate()` (Lines ~2201-2700)

Similar structure:
1. **Contract**: Takes an address, requires it's currently allocated
2. **Body**: Clear the bit in bitmap
3. **Ensures**: Block is no longer in `allocated_blocks` set

## Reading the Lemmas (Lines ~351-700)

Lemmas are reusable proof pieces. Key ones:

### `lemma_allocated_blocks_finite`
Proves that `allocated_blocks` is a finite set (needed for `Set::len()`).

### `lemma_address_block_inverse`
Proves that `addr_to_block_idx(block_idx_to_addr(i)) == i` - address conversion roundtrips.

### `lemma_allocation_unique`
Proves that a newly allocated address wasn't previously allocated.

### `lemma_allocate_disjoint`
Proves that different blocks have non-overlapping memory ranges.

## Reading the Properties (SlabView methods)

Find these around lines 81-350:

| Property | Meaning |
|----------|---------|
| `is_valid_idx(i)` | Index i is in range [0, num_data_blocks) |
| `is_allocated(i)` | Block i is in the allocated set |
| `is_valid_addr(a)` | Address a maps to a valid block and is aligned |
| `can_allocate()` | At least one block is free |
| `blocks_disjoint(i,j)` | Block i and j don't overlap in memory |
| `is_within_buffer(a)` | Address a is within the original buffer bounds |

## Reading Tests (Lines ~3001-3263)

Tests are `proof fn` that verify properties hold universally:

```rust
proof fn test_allocate_returns_valid_address()
{
    // This proves for ALL valid slab configurations:
    // If allocate succeeds, returned address is valid
    assert forall |slab: Slab, addr: usize|
        slab.inv() && is_ok_addr(slab, addr) ==>
        slab@.is_valid_addr(addr as int)
}
```

**Key insight**: Unlike runtime tests that check specific cases, these prove properties hold for ALL possible inputs!

## Common Proof Patterns

### Pattern 1: Invoke a Lemma
```rust
proof {
    self.lemma_allocated_blocks_finite();
    // Now Verus knows allocated_blocks.len() is valid
}
```

### Pattern 2: Assert Intermediate Facts
```rust
proof {
    assert(old(self)@.free() > 0);  // Trigger proof search
    assert(self@.allocated_blocks.len() == old(self)@.allocated_blocks.len() + 1);
}
```

### Pattern 3: Witness a Set Element
```rust
proof {
    assert(self@.allocated_blocks.contains(block_idx));  // Witness
}
```

## Trusted Boundaries

These are axiomatized (we assume they're correct):

| Module | What's Trusted |
|--------|----------------|
| `bitmap.rs` | Bitmap operations (alloc, set, clear, test) |
| `raw_array.rs` | Raw memory array operations |
| `error.rs` | Error type definitions |

The slab proofs assume these specs are correct. The bitmap module itself has separate verification in `/verus/bitmap/`.

## Quick Reference: Where to Find Things

| What | Lines (approx) |
|------|----------------|
| Slab struct | 31-65 |
| SlabView struct | 67-80 |
| SlabView methods | 81-200 |
| Helper specs (is_power_of_two, etc.) | 200-350 |
| Proof lemmas | 351-700 |
| Slab::inv() | 700-750 |
| View impl | 750-800 |
| from_raw_parts | 1101-1700 |
| allocate | 1701-2200 |
| deallocate | 2201-2700 |
| new | 2701-3000 |
| Tests | 3001-3263 |

## Summary: The Verification Story

1. **Abstract State**: `SlabView` represents "what blocks are allocated" as a set
2. **Invariant**: Defines valid slab configurations  
3. **View**: Connects bitmap bits to the abstract set
4. **Operations**: `allocate`/`deallocate` modify bitmap and update abstract set
5. **Proofs**: Show invariant is preserved and operations behave correctly
6. **Properties**: High-level guarantees (disjointness, uniqueness, etc.)

The beauty: if verification passes, we have mathematical proof that the allocator is correct for ALL possible inputs, not just tested cases!
