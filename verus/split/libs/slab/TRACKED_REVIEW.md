# Review: Slab Allocator Tracked Permission Verification

## Grade: A-

## Files Reviewed

- `verus/split/libs/slab/lib.rs` (exec)
- `verus/split/libs/slab/lib.spec.rs` (spec)
- `verus/split/libs/slab/lib.proof.rs` (proof)
- `verus/split/libs/slab/lib.test.rs` (test)

## Issues Found

### High

#### H1: `index_perm` 在 `wf()` 中缺少范围约束

- **Location**: `lib.spec.rs` — `SlabPerms::wf()` (line ~215-218)
- **Description**: `wf()` 对 `index_perm` 只检查了 `self.index_perm.provenance() == prov`，但**没有验证 `index_perm` 覆盖的内存范围**是否为 `[addr, data_addr)` 即真正的 index 区域。`from_raw_parts` 内部正确地分割了区域，但 `wf()` spec 不强制这一点。理论上一个调用者可以用同 provenance 但覆盖不同区域的 `PointsToRaw` 替换 `index_perm`，仍然满足 `wf()`。
- **Suggested Fix**: 在 `SlabView` 中加入 `index_addr: int` 和 `num_index_blocks: int` 字段，并在 `wf()` 中要求 `index_perm.is_range(index_addr, num_index_blocks * block_size)`。同时更新 `Slab` 的 `View` 实现以填充这些字段。

#### H2: `allocate`/`deallocate` 合约中未显式保证 `index_perm` 不变

- **Location**: `lib.rs` — `allocate` ensures (line ~404-436) 和 `deallocate` ensures (line ~492-526)
- **Description**: `take_block_perm` 和 `put_block_perm` 的 ensures 中有 `self.index_perm == old(self).index_perm`，但 `allocate`/`deallocate` 的顶层合约仅保证 `perms.wf(self@, old(perms).index_perm.provenance())`，没有显式说 `perms.index_perm == old(perms).index_perm`。调用者无法从合约中直接推出 index_perm 没变。
- **Suggested Fix**: 在 `allocate` 和 `deallocate` 的 ensures 中加入：
  ```rust
  &&& perms.index_perm == old(perms).index_perm
  ```

### Medium

#### M1: `perms_wf` 未约束 map 域的精确性

- **Location**: `lib.spec.rs` — `SlabView::perms_wf()` (line ~182-193)
- **Description**: `perms_wf` 说"free block 的 i 有权限"和"allocated block 的 i 没有权限"，但没有排除 `perms.dom()` 中包含 `i >= num_data_blocks` 或 `i < 0` 的情况。一个包含多余 entries 的 map 仍然满足 `perms_wf`。
- **Suggested Fix**: 加入第三条约束使 dom 精确等于 free block 集合：
  ```rust
  &&& forall|i: int| #![trigger perms.dom().contains(i)]
      perms.dom().contains(i) ==> (0 <= i < self.num_data_blocks && !self.is_allocated(i))
  ```

#### M2: `_padding` 权限在 `from_raw_parts` 中被丢弃

- **Location**: `lib.rs` — `from_raw_parts` proof block (line ~341)
  ```rust
  let tracked (used_perm, _padding) = mem.split(used_range);
  ```
- **Description**: 当 `len` 不是 `total_num_blocks * block_size` 的整数倍时，尾部 padding 字节的权限被静默丢弃。虽然不影响 soundness（permission leak 是安全的），但这部分内存永远无法被回收。
- **Suggested Fix**: 考虑将 `_padding` 也作为 `SlabPerms` 的一个字段保存：
  ```rust
  pub tracked struct SlabPerms {
      pub free_perms: Map<int, PointsToRaw>,
      pub index_perm: PointsToRaw,
      pub padding_perm: PointsToRaw,  // 新增：尾部未使用的 padding 区域权限
  }
  ```
  或者在 `wf()` / doc comment 中明确说明这是预期的、有意的丢弃行为。

#### M3: `allocate` 错误路径未显式保证 `perms` 不变

- **Location**: `lib.rs` — `allocate` ensures (line ~433)
- **Description**: 错误分支的 ensures 只有 `self@ == old(self)@`，没有说 `perms` 不变。虽然 Rust 借用检查器保证 `&mut SlabPerms` 被借用期间不会被外部修改，且 `lemma_alloc_error_preserves_state` 不修改 perms，但显式写出来会更安全且对调用者更友好。
- **Suggested Fix**: 在 `allocate` 的 ensures 中加入：
  ```rust
  result is Err ==> (
      self@ == old(self)@
      && !old(self)@.can_allocate()
      && perms.free_perms == old(perms).free_perms
      && perms.index_perm == old(perms).index_perm
  ),
  ```

### Low

#### L1: `lemma_bitwise_implies_is_pow2` 是唯一的 `external_body`

- **Location**: `lib.proof.rs` (line ~964-967)
- **Description**: 这是 bitwise 操作到数学 `is_pow2` 的桥接引理，属于合理的信任边界。但应有文档说明为何需要 `external_body` 以及其数学正确性的依据。
- **Suggested Fix**: 为该函数添加 doc comment 说明：
  ```rust
  /// Trusted bridge: bitwise check `n & (n - 1) == 0` implies `is_pow2(n)`.
  ///
  /// # Trust Justification
  ///
  /// Verus does not natively support bitwise operation reasoning.
  /// This is a well-known mathematical property: for n > 0,
  /// n & (n - 1) == 0 iff n is a power of two. See Hacker's Delight, Chapter 2.
  #[verifier::external_body]
  proof fn lemma_bitwise_implies_is_pow2(n: usize)
  ```

#### L2: `join_block_perms` 定义但未使用

- **Location**: `lib.proof.rs` (line ~1845-1900)
- **Description**: `split_into_blocks` 的逆操作，作为 proof artifact 有价值（可能在后续 slab 销毁时需要），但当前是 dead code。
- **Suggested Fix**: 保留，但加注释说明用途：
  ```rust
  /// Joins per-block permissions back into a contiguous PointsToRaw.
  /// Inverse of split_into_blocks.
  /// NOTE: Currently unused. Reserved for future slab destruction / memory reclamation.
  ```

#### L3: 测试未覆盖 exhaustion 场景

- **Location**: `lib.test.rs`
- **Description**: 没有验证"分配到耗尽 → 释放 → 重新分配"的完整循环。`lemma_dealloc_from_full_enables_alloc` 存在但未在测试中被使用。
- **Suggested Fix**: 添加一个测试，用循环分配到 `allocate` 返回 Err（即 `!can_allocate()`），然后释放一个块并验证 `can_allocate()` 恢复、下一次 allocate 成功。

#### L4: 冗余测试

- **Location**: `lib.test.rs`
- **Description**: `test_slab_from_raw_parts_verified` 和 `test_slab_creation_verified` 几乎完全相同，可合并。

## Positive Observations

- **线性类型用得恰到好处**: `PointsToRaw` 作为 tracked 资源在 `allocate` 返回给调用者、`deallocate` 从调用者收回，形成闭环。Double-free 在类型系统层面被阻止，不需要运行时检查。
- **`SlabPerms` 设计简洁**: `free_perms: Map<int, PointsToRaw>` + `index_perm: PointsToRaw` 的分离很清晰。`take_block_perm`/`put_block_perm` 的 ensures 精确描述了 map 变化。
- **`split_into_blocks` 递归分割正确**: 正确的递归 proof fn，有 `decreases n` 子句，将一整块 `PointsToRaw` 分成 per-block 权限，是整个 tracked 架构的关键基础。
- **Provenance 一致性追踪完整**: 所有权限操作始终保持 `provenance` 相同，确保内存来自同一分配。
- **Frame 条件完整**: `allocate`/`deallocate` 的 ensures 都精确描述了哪些字段不变（`num_data_blocks`、`block_size`、`data_addr`），以及 `allocated_blocks` 的精确变化（`insert`/`remove`）。
- **抽象层次恰当**: 使用 `Set<int>` 代替 bitmap 实现细节，`perms_wf` 使用 `Map<int, PointsToRaw>` 而非裸指针算术。
- **零 `assume`/`admit`**: 核心模块没有任何未验证的假设。

## Scoring Breakdown

| 评估维度 | 得分 | 说明 |
|---------|------|------|
| Coverage (函数覆盖) | 5/5 | 所有三个核心函数都有完整的 tracked 合约 |
| Soundness (无 assume/admit) | 5/5 | 唯一的 external_body 是合理的信任边界 |
| Spec 精确度 | 3.5/5 | index_perm range 未约束，perms map 域不精确 |
| Tracked 权限流转 | 4.5/5 | 核心流转完整，但 index_perm 不变性和错误路径 perms 保持未显式约束 |
| 抽象质量 | 5/5 | 使用 Set/Map 抽象，隐藏 bitmap 实现细节 |
| 测试覆盖 | 3.5/5 | 缺少 exhaustion 场景，有冗余测试 |
| **总分** | **A-** | |

## Recommended Fix Priority

1. **H1** — `index_perm` range 约束（最重要的 spec gap）
2. **H2** — `index_perm` 不变性在合约中显式声明
3. **M1** — `perms_wf` map 域精确约束
4. **M3** — 错误路径 perms 不变性
5. **M2** — padding 权限处理或文档说明
6. **L1-L4** — 文档和测试改进
