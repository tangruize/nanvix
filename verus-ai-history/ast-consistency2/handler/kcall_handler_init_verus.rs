pub fn kcall_handler_init() -> (history: Ghost<Seq<HarvestOutcome>>)
    ensures
        spec_loop_invariant(history@),
        history@.len() == 0,
{
    event_init();
    proof { lemma_loop_invariant_base(); }
    Ghost(Seq::empty())
}
