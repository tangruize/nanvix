// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

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
