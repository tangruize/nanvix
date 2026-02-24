pub fn virt_align_down(addr: usize, alignment: usize) -> (result: usize)
    requires
        alignment > 0,
    ensures
        result as int == spec_align_down(addr as int, alignment as int),
        result <= addr,
        result as int % alignment as int == 0,
{
    proof {
        VirtProofs::lemma_align_down_le(addr as int, alignment as int);
        VirtProofs::lemma_align_down_aligned(addr as int, alignment as int);
    }
    (addr / alignment) * alignment
}
