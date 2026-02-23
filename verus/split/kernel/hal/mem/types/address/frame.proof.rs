// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

impl FrameNumber {

    /// Lemma: Proves inv() from the public bound on value.
    ///
    /// # Description
    ///
    /// Bridges the gap between the publicly visible field `value` and the
    /// closed `inv()` spec, allowing callers that have proven the bound
    /// to establish the invariant without seeing its definition.
    pub proof fn lemma_inv_from_bound(&self)
        requires self.value as int <= MAX_FRAME_NUMBER as int,
        ensures self.inv(),
    {}
}

impl FrameAddress {

    /// Lemma: Proves inv() from the publicly visible alignment condition.
    ///
    /// # Description
    ///
    /// Bridges the gap between `spec_is_aligned()` (open) and `inv()` (closed),
    /// allowing callers to establish the invariant from alignment knowledge.
    pub proof fn lemma_inv_from_aligned(&self)
        requires self.raw_addr as int % FRAME_SIZE as int == 0,
        ensures self.inv(),
    {}
}

impl PageAlignedPhysAddr {

    /// Lemma: Proves inv() from the publicly visible alignment condition.
    ///
    /// # Description
    ///
    /// Bridges the gap between alignment knowledge and the closed `inv()` spec.
    pub proof fn lemma_inv_from_aligned(&self)
        requires self.raw_addr as int % FRAME_SIZE as int == 0,
        ensures self.inv(),
    {}
}

impl TruncatedMemoryRegion {

    /// Lemma: If inv() holds, then frame_count > 0.
    ///
    /// # Description
    ///
    /// This lemma reveals the key property that inv() implies frame_count > 0,
    /// which is needed by callers since inv() is closed.
    pub proof fn lemma_inv_implies_frame_count_positive(&self)
        requires self.inv(),
        ensures
            self.spec_frame_count() > 0,
            self.spec_size() > 0,
    {
        // From inv(): size > 0 and size % FRAME_SIZE == 0.
        // Therefore size >= FRAME_SIZE, so size / FRAME_SIZE >= 1.
    }
}

} // verus!
