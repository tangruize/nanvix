# Performance Issue

## 1. Removing `num_allocated` or `used` in slab.rs

| Version | kheap rlimit | Real Time |
|---------|--------------|----------|
| Baseline | 19,968,891 | 4.1s |
| Slower | 147,004,787 (**7.4x**) | 16.3s (**4x**) |

### Slower version

Commands to reproduce:

```sh
git checkout e42d67be
cd verus
time verus --crate-type lib lib.rs --time-expanded | tee result.log
```

<details>
<summary>Result</summary>

```txt
Some checks are taking longer than 2s (diagnostics for these may be reported out of order)
• kheap.rs:1195:5: 1195:84 has finished in 13s
  ⌝ function body check for lib::kheap::Kheap::allocate                                           verification results:: 458 verified, 0 errors
(use --output-json for machine-readable output)
verus-build-info
Verus
  Version: 0.2025.12.05.d3fefb4
  Profile: release
  Platform: linux_x86_64
  Toolchain: 1.91.0-x86_64-unknown-linux-gnu

total-time:           16011 ms    (estimated total cpu time 20080 ms)
    rust-time:                1378 ms
        init-and-types:            774 ms
        trait-conflicts:            34 ms
        compile-time:              569 ms
    verification-time:       14432 ms
        vir-time:                  760 ms
            hir-time:                  194 ms
            import-time:               140 ms
            rust-to-vir:               160 ms
        verify-crate-time:       13671 ms
    unaccounted-time:          200 ms

verify-crate-time-breakdown
    total verify-time:          17660 ms   (24 threads)
      1. kheap                                         13518 ms
      2. slab                                           1118 ms
      3. bitmap                                         1052 ms
        total air-time:              1272 ms   (24 threads)
      1. slab                                            339 ms
      2. bitmap                                          269 ms
      3. kheap                                           118 ms
        total smt-time:             15141 ms   (24 threads)
            total smt-init:               445 ms   (24 threads)
                1. slab                                            138 ms
                2. kheap                                            67 ms
                3. bitmap                                           61 ms
            total smt-run:              14696 ms, 157737146 rlimit (24 threads)
                1. kheap                                         13193 ms, 147004787 rlimit
                2. bitmap                                          531 ms,  3914971 rlimit
                3. slab                                            462 ms,  2452581 rlimit
verus --crate-type lib lib.rs --time-expanded  19.48s user 1.17s system 126% cpu 16.307 total
tee result.log  0.00s user 0.00s system 0% cpu 16.307 total
```

</details>

Show diff in this commit:

```sh
git show
```

### Baseline

Baseline (one commit before):

```sh
git checkout e42d67be~1
cd verus
time verus --crate-type lib lib.rs --time-expanded | tee result_baseline.log
```

<details>
<summary>Result</summary>

```
verification results:: 458 verified, 0 errors
(use --output-json for machine-readable output)
verus-build-info
Verus
  Version: 0.2025.12.05.d3fefb4
  Profile: release
  Platform: linux_x86_64
  Toolchain: 1.91.0-x86_64-unknown-linux-gnu

total-time:            3781 ms    (estimated total cpu time 7885 ms)
    rust-time:                1376 ms
        init-and-types:            774 ms
        trait-conflicts:            35 ms
        compile-time:              567 ms
    verification-time:        2200 ms
        vir-time:                  792 ms
            hir-time:                  211 ms
            import-time:               139 ms
            rust-to-vir:               169 ms
        verify-crate-time:        1408 ms
    unaccounted-time:          203 ms

verify-crate-time-breakdown
    total verify-time:           5432 ms   (24 threads)
      1. slab                                           1213 ms
      2. kheap                                          1186 ms
      3. bitmap                                         1049 ms
        total air-time:              1229 ms   (24 threads)
      1. slab                                            334 ms
      2. bitmap                                          266 ms
      3. kheap                                            87 ms
        total smt-time:              2944 ms   (24 threads)
            total smt-init:               445 ms   (24 threads)
                1. slab                                            138 ms
                2. kheap                                            66 ms
                3. bitmap                                           60 ms
            total smt-run:               2499 ms, 30775496 rlimit (24 threads)
                1. kheap                                           896 ms, 19968891 rlimit
                2. slab                                            559 ms,  2526827 rlimit
                3. bitmap                                          531 ms,  3914971 rlimit
verus --crate-type lib lib.rs --time-expanded  7.24s user 1.20s system 207% cpu 4.070 total
tee result_baseline.log  0.00s user 0.00s system 0% cpu 4.070 total
```

</details>

## 2. Using struct update in slab.rs

| Version | kheap rlimit | Real Time |
|---------|--------------|----------|
| Baseline | 20,846,250 | 4.1s |
| Slower | 57,496,565 (**2.8x**) | 5.0s (**1.2x**) |

```
git checkout a1df2a55
cd verus
```

### Baseline

```
time verus --crate-type lib lib.rs --time-expanded | tee result_baseline.log
```

<details>
<summary>Result</summary>

```txt
verification results:: 458 verified, 0 errors
(use --output-json for machine-readable output)
verus-build-info
Verus
  Version: 0.2025.12.05.d3fefb4
  Profile: release
  Platform: linux_x86_64
  Toolchain: 1.91.0-x86_64-unknown-linux-gnu

total-time:            3805 ms    (estimated total cpu time 7929 ms)
    rust-time:                1354 ms
        init-and-types:            762 ms
        trait-conflicts:            34 ms
        compile-time:              557 ms
    verification-time:        2257 ms
        vir-time:                  751 ms
            hir-time:                  191 ms
            import-time:               137 ms
            rust-to-vir:               157 ms
        verify-crate-time:        1505 ms
    unaccounted-time:          194 ms

verify-crate-time-breakdown
    total verify-time:           5551 ms   (24 threads)
      1. slab                                           1318 ms
      2. kheap                                          1240 ms
      3. bitmap                                         1034 ms
        total air-time:              1217 ms   (24 threads)
      1. slab                                            325 ms
      2. bitmap                                          264 ms
      3. kheap                                            87 ms
        total smt-time:              3090 ms   (24 threads)
            total smt-init:               444 ms   (24 threads)
                1. slab                                            139 ms
                2. kheap                                            67 ms
                3. bitmap                                           61 ms
            total smt-run:               2646 ms, 31253410 rlimit (24 threads)
                1. kheap                                           951 ms, 20846250 rlimit
                2. slab                                            676 ms,  2386331 rlimit
                3. bitmap                                          518 ms,  3829564 rlimit
verus --crate-type lib lib.rs --time-expanded  7.31s user 1.12s system 207% cpu 4.053 total
tee result_baseline.log  0.00s user 0.00s system 0% cpu 4.052 total
```

</details>

### Slower version

```
git apply <(cat <<'EOF'
diff --git a/verus/slab.rs b/verus/slab.rs
index 57ed43c6..e714e6c4 100644
--- a/verus/slab.rs
+++ b/verus/slab.rs
@@ -1807,10 +1807,8 @@ impl Slab {
                 &&& 0 <= block_idx < self@.num_data_blocks
                 &&& !old(self)@.is_allocated(block_idx)
                 &&& self@.is_allocated(block_idx)
-                // Frame: static fields unchanged.
-                &&& self@.num_data_blocks == old(self)@.num_data_blocks
-                &&& self@.block_size == old(self)@.block_size
-                &&& self@.data_addr == old(self)@.data_addr
+                // Frame: static fields unchanged (using struct update syntax).
+                &&& self@ == (SlabView { allocated_blocks: self@.allocated_blocks, ..old(self)@ })
                 // Frame: other blocks unchanged.
                 &&& forall|i: int| 0 <= i < self@.num_data_blocks && i != block_idx ==>
                     self@.is_allocated(i) == old(self)@.is_allocated(i)
@@ -2007,10 +2005,8 @@ impl Slab {
             result is Ok ==> {
                 let block_idx = old(self)@.addr_to_block_idx(addr as int);
                 &&& !self@.is_allocated(block_idx)
-                // Frame: static fields unchanged.
-                &&& self@.num_data_blocks == old(self)@.num_data_blocks
-                &&& self@.block_size == old(self)@.block_size
-                &&& self@.data_addr == old(self)@.data_addr
+                // Frame: static fields unchanged (using struct update syntax).
+                &&& self@ == (SlabView { allocated_blocks: self@.allocated_blocks, ..old(self)@ })
                 // Frame: other blocks unchanged.
                 &&& forall|i: int| 0 <= i < self@.num_data_blocks && i != block_idx ==>
                     self@.is_allocated(i) == old(self)@.is_allocated(i)
EOF
)

time verus --crate-type lib lib.rs --time-expanded | tee result.log
git restore slab.rs
```

<details>
<summary>Result</summary>

```txt
verification results:: 458 verified, 0 errors
(use --output-json for machine-readable output)
verus-build-info
Verus
  Version: 0.2025.12.05.d3fefb4
  Profile: release
  Platform: linux_x86_64
  Toolchain: 1.91.0-x86_64-unknown-linux-gnu

total-time:            4763 ms    (estimated total cpu time 9059 ms)
    rust-time:                1352 ms
        init-and-types:            758 ms
        trait-conflicts:            34 ms
        compile-time:              559 ms
    verification-time:        3215 ms
        vir-time:                  745 ms
            hir-time:                  190 ms
            import-time:               140 ms
            rust-to-vir:               155 ms
        verify-crate-time:        2469 ms
    unaccounted-time:          196 ms

verify-crate-time-breakdown
    total verify-time:           6687 ms   (24 threads)
      1. kheap                                          2317 ms
      2. slab                                           1335 ms
      3. bitmap                                         1057 ms
        total air-time:              1248 ms   (24 threads)
      1. slab                                            339 ms
      2. bitmap                                          266 ms
      3. kheap                                            95 ms
        total smt-time:              4165 ms   (24 threads)
            total smt-init:               445 ms   (24 threads)
                1. slab                                            137 ms
                2. kheap                                            67 ms
                3. bitmap                                           61 ms
            total smt-run:               3720 ms, 67910178 rlimit (24 threads)
                1. kheap                                          2010 ms, 57496565 rlimit
                2. slab                                            675 ms,  2392784 rlimit
                3. bitmap                                          532 ms,  3829564 rlimit
verus --crate-type lib lib.rs --time-expanded  8.47s user 1.11s system 190% cpu 5.021 total
tee result.log  0.00s user 0.00s system 0% cpu 5.021 total
```

</details>

**Note** An even slower version is using `insert()` to update `allocated_blocks` in the struct directly, which leads to 6x more rlimit.

### 3. Removing unused spec fn in kheap.rs

| Version | kheap rlimit | Real Time |
|---------|--------------|----------|
| Baseline | 20,846,250 | 4.1s |
| Remove all 4 | 37,119,890 (**1.8x**) | 4.5s (**1.1x**) |
| Remove only 2 | 313,205,148 (**15x**) | ❌ Timeout |

### Baseline

```
git checkout c2a3fdb0
cd verus
time verus --crate-type lib lib.rs --time-expanded | tee result_baseline.log
```

<details>
<summary>Result</summary>

```txt
verification results:: 458 verified, 0 errors
(use --output-json for machine-readable output)
verus-build-info
Verus
  Version: 0.2025.12.05.d3fefb4
  Profile: release
  Platform: linux_x86_64
  Toolchain: 1.91.0-x86_64-unknown-linux-gnu

total-time:            3827 ms    (estimated total cpu time 7944 ms)
    rust-time:                1363 ms
        init-and-types:            766 ms
        trait-conflicts:            34 ms
        compile-time:              562 ms
    verification-time:        2268 ms
        vir-time:                  763 ms
            hir-time:                  192 ms
            import-time:               146 ms
            rust-to-vir:               159 ms
        verify-crate-time:        1504 ms
    unaccounted-time:          195 ms

verify-crate-time-breakdown
    total verify-time:           5542 ms   (24 threads)
      1. slab                                           1310 ms
      2. kheap                                          1242 ms
      3. bitmap                                         1040 ms
        total air-time:              1215 ms   (24 threads)
      1. slab                                            326 ms
      2. bitmap                                          268 ms
      3. kheap                                            86 ms
        total smt-time:              3095 ms   (24 threads)
            total smt-init:               441 ms   (24 threads)
                1. slab                                            136 ms
                2. kheap                                            66 ms
                3. bitmap                                           61 ms
            total smt-run:               2654 ms, 31308234 rlimit (24 threads)
                1. kheap                                           953 ms, 20846250 rlimit
                2. slab                                            676 ms,  2386331 rlimit
                3. bitmap                                          522 ms,  3829564 rlimit
verus --crate-type lib lib.rs --time-expanded  7.40s user 1.08s system 207% cpu 4.084 total
tee result_baseline.log  0.00s user 0.00s system 0% cpu 4.083 total
```

</details>

### Slower version

**Note**: the following content are generated by AI. I have double-checked.

Remove four "unused" or "simple alias" spec functions:
- `can_allocate_in_slab` (simple delegate to `get_slab(size).can_allocate()`)
- `addr_in_slab` (simple delegate to `get_slab(size).is_valid_addr(addr)`)
- `spec_base_addr` (simple delegate to `self.base_addr@`)
- `spec_total_size` (simple delegate to `self.total_size@`)

```
git apply <(cat <<'EOF'
diff --git a/verus/kheap.rs b/verus/kheap.rs
index 45e24ec7..c4f36a26 100644
--- a/verus/kheap.rs
+++ b/verus/kheap.rs
@@ -186,17 +186,6 @@ impl KheapView {
         self.total_allocated() == 0
     }
 
-    /// Returns true if a specific slab can allocate.
-    ///
-    /// # Note
-    ///
-    /// This function appears to be a simple delegate to `get_slab(size).can_allocate()`.
-    /// However, it is essential for SMT term sharing optimization. Removing it causes
-    /// verification rlimit to increase by ~250% due to quantifier instantiation explosion.
-    pub open spec fn can_allocate_in_slab(&self, size: SlabSize) -> bool {
-        self.get_slab(size).can_allocate()
-    }
-
     /// Returns true if all slabs are within the heap extent.
     ///
     /// # Description
@@ -273,11 +262,6 @@ impl KheapView {
     // Address Validity
     //==============================================================================================
 
-    /// Returns true if an address is within a specific slab's region.
-    pub open spec fn addr_in_slab(&self, addr: int, size: SlabSize) -> bool {
-        self.get_slab(size).is_valid_addr(addr)
-    }
-
     /// Returns true if an address is valid in any slab.
     pub open spec fn is_valid_heap_addr(&self, addr: int) -> bool {
         ||| self.slab_8.is_valid_addr(addr)
@@ -425,21 +409,6 @@ impl View for Kheap {
 }
 
 impl Kheap {
-    /// Returns the base address of the heap (spec).
-    ///
-    /// # Note
-    ///
-    /// This function is essential for SMT term sharing optimization.
-    /// Removing it causes verification rlimit to increase significantly.
-    pub closed spec fn spec_base_addr(&self) -> int {
-        self.base_addr@
-    }
-
-    /// Returns the total size of the heap (spec).
-    pub closed spec fn spec_total_size(&self) -> int {
-        self.total_size@
-    }
-
     /// Spec helper for disjointness checking between two slabs.
     pub open spec fn spec_slabs_disjoint(s1: &SlabView, s2: &SlabView) -> bool {
         let s1_start: int = s1.data_addr;
EOF
)

time verus --crate-type lib lib.rs --time-expanded | tee result.log
git restore kheap.rs
```

<details>
<summary>Result</summary>

```txt
verification results:: 458 verified, 0 errors
(use --output-json for machine-readable output)
verus-build-info
Verus
  Version: 0.2025.12.05.d3fefb4
  Profile: release
  Platform: linux_x86_64
  Toolchain: 1.91.0-x86_64-unknown-linux-gnu

total-time:            4195 ms    (estimated total cpu time 8512 ms)
    rust-time:                1363 ms
        init-and-types:            766 ms
        trait-conflicts:            34 ms
        compile-time:              563 ms
    verification-time:        2639 ms
        vir-time:                  754 ms
            hir-time:                  192 ms
            import-time:               139 ms
            rust-to-vir:               159 ms
        verify-crate-time:        1885 ms
    unaccounted-time:          192 ms

verify-crate-time-breakdown
    total verify-time:           6123 ms   (24 threads)
      1. kheap                                          1732 ms
      2. slab                                           1315 ms
      3. bitmap                                         1040 ms
        total air-time:              1234 ms   (24 threads)
      1. slab                                            330 ms
      2. bitmap                                          265 ms
      3. kheap                                            89 ms
        total smt-time:              3629 ms   (24 threads)
            total smt-init:               454 ms   (24 threads)
                1. slab                                            136 ms
                2. kheap                                            68 ms
                3. bitmap                                           60 ms
            total smt-run:               3175 ms, 47581874 rlimit (24 threads)
                1. kheap                                          1440 ms, 37119890 rlimit
                2. slab                                            674 ms,  2386331 rlimit
                3. bitmap                                          520 ms,  3829564 rlimit
verus --crate-type lib lib.rs --time-expanded  7.89s user 1.14s system 207% cpu 4.452 total
tee result.log  0.00s user 0.00s system 0% cpu 4.452 total
```

</details>

**Summary:** Removing these four "simple alias" spec functions causes kheap rlimit to increase from 20,846,250 to 37,119,890 (**+78%**). These functions appear to enable SMT term sharing optimization.

### Partial removal (even worse!)

Interestingly, removing **only** `spec_base_addr` and `spec_total_size` (2 out of 4 functions) causes **much worse** performance degradation than removing all four!

```
git apply <(cat <<'EOF'
diff --git a/verus/kheap.rs b/verus/kheap.rs
index 45e24ec7..54b60289 100644
--- a/verus/kheap.rs
+++ b/verus/kheap.rs
@@ -442,20 +442,6 @@ impl View for Kheap {
 
 impl Kheap {
     /// Returns the base address of the heap (spec).
-    ///
-    /// # Note
-    ///
-    /// This function is essential for SMT term sharing optimization.
-    /// Removing it causes verification rlimit to increase significantly.
-    pub closed spec fn spec_base_addr(&self) -> int {
-        self.base_addr@
-    }
-
-    /// Returns the total size of the heap (spec).
-    pub closed spec fn spec_total_size(&self) -> int {
-        self.total_size@
-    }
-
     /// Spec helper for disjointness checking between two slabs.
     pub open spec fn spec_slabs_disjoint(s1: &SlabView, s2: &SlabView) -> bool {
         let s1_start: int = s1.data_addr;
EOF
)

time verus --crate-type lib lib.rs --time-expanded | tee result_partial.log
git restore kheap.rs
```

<details>
<summary>Result</summary>

```txt
error: function body check: Resource limit (rlimit) exceeded
 --> kheap.rs:1191:5
  |
1191 |     pub unsafe fn allocate(&mut self, size: usize) -> (result: Result<usize, Error>)

verification results:: 457 verified, 1 errors

verify-crate-time-breakdown
    total verify-time:          16140 ms   (24 threads)
      1. kheap                                         11792 ms
        total smt-run:              13159 ms, 323667132 rlimit (24 threads)
                1. kheap                                         11458 ms, 313205148 rlimit
                2. slab                                            675 ms,  2386331 rlimit
                3. bitmap                                          523 ms,  3829564 rlimit
```

</details>

**Summary of Issue 3:**

| What was removed | kheap rlimit | Change | Status |
|------------------|--------------|--------|--------|
| Baseline (nothing removed) | 20,846,250 | - | ✅ Pass |
| Only `spec_base_addr` + `spec_total_size` | 313,205,148 | **+1402%** | ❌ **Timeout** |
| All 4 functions | 37,119,890 | +78% | ✅ Pass |

**Paradox:** Removing **fewer** functions causes **worse** performance! This suggests complex interactions in SMT term sharing where having some but not all of these "alias" functions breaks certain optimizations.
