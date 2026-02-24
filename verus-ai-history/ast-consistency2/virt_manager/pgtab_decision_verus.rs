pub fn pgtab_decision(last_base: Option<usize>, vaddr: usize) -> (result: PgtabDecision)
    requires
        last_base.is_some() ==> last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
    ensures
        // When no previous base, always create new.
        last_base.is_none() ==> matches!(result, PgtabDecision::CreateNew),
        // Precise branching when previous base exists.
        last_base.is_some() && spec_pgtab_base(vaddr as int) > last_base.unwrap() as int
            ==> matches!(result, PgtabDecision::CreateNew),
        last_base.is_some() && spec_pgtab_base(vaddr as int) == last_base.unwrap() as int
            ==> matches!(result, PgtabDecision::Reuse),
        last_base.is_some() && spec_pgtab_base(vaddr as int) < last_base.unwrap() as int
            ==> matches!(result, PgtabDecision::Overlap),
        // Combined: non-overlapping case is ok.
        last_base.is_some() && spec_pgtab_base(vaddr as int) >= last_base.unwrap() as int
            ==> result.spec_is_ok(),
{
    let curr_base: usize = compute_pgtab_base(vaddr);
    match last_base {
        None => PgtabDecision::CreateNew,
        Some(prev) => {
            if curr_base > prev {
                PgtabDecision::CreateNew
            } else if curr_base == prev {
                PgtabDecision::Reuse
            } else {
                PgtabDecision::Overlap
            }
        },
    }
}
