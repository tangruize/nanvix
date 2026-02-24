pub fn check_pgtab_monotonicity(prev_vaddr: usize, curr_vaddr: usize) -> (result: bool)
    requires
        prev_vaddr <= curr_vaddr,
    ensures
        result == true,
{
    proof {
        VirtProofs::lemma_sorted_addrs_sorted_pgtab_bases(prev_vaddr as int, curr_vaddr as int);
    }
    let prev_base: usize = compute_pgtab_base(prev_vaddr);
    let curr_base: usize = compute_pgtab_base(curr_vaddr);
    curr_base >= prev_base
}
