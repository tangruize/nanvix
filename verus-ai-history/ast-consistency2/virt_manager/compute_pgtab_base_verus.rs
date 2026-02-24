pub fn compute_pgtab_base(vaddr: usize) -> (result: usize)
    ensures
        result as int == spec_pgtab_base(vaddr as int),
        result <= vaddr,
        result as int % INIT_PGTAB_ALIGNMENT as int == 0,
{
    proof {
        VirtProofs::lemma_align_down_le(vaddr as int, INIT_PGTAB_ALIGNMENT as int);
        VirtProofs::lemma_align_down_aligned(vaddr as int, INIT_PGTAB_ALIGNMENT as int);
    }
    virt_align_down(vaddr, INIT_PGTAB_ALIGNMENT)
}
