    pub fn store_mutex_guard(&mut self, address: u64)
        requires
            old(self).wf(),
            old(self)@.locked_mutex_count < usize::MAX as nat,
            !old(self)@.has_mutex(address as int),
        ensures
            self@.has_mutex(address as int),
            forall|a: int| a != address as int ==>
                self@.has_mutex(a) == old(self)@.has_mutex(a),
            self@.locked_mutex_count == old(self)@.locked_mutex_count + 1,
            !self@.drop_safe(),
            self@.id == old(self)@.id,
            self@.kernel_stack == old(self)@.kernel_stack,
            self@.user_stack == old(self)@.user_stack,
            self@.user_tda == old(self)@.user_tda,
            self@.interrupt_reason == old(self)@.interrupt_reason,
            self.wf(),
    {
        proof {
            reveal(ThreadState::wf);
            let old_seq: Seq<u64> = self.locked_mutex_set@;
            let f = |v: u64| v as int;
            // to_set of push equals insert; map distributes over insert.
            old_seq.lemma_push_to_set_commute(address);
            old_seq.to_set().lemma_set_map_insert_commute(address, f);
            // Prove no_duplicates after push.
            if old_seq.contains(address) {
                assert(old_seq.to_set().contains(address));
            }
            assert(!old_seq.contains(address));
            assert(old_seq.push(address).no_duplicates());
            // Prove len: since no_dups, len == seq len (inlined vstd calls).
            {
                let f = |v: u64| v as int;
                let pushed: Seq<u64> = old_seq.push(address);
                vstd::seq_lib::seq_to_set_is_finite(pushed);
                pushed.to_set().lemma_map_finite(f);
                pushed.unique_seq_to_set();
                assert(vstd::relations::injective(f));
                assert forall |a: u64| pushed.to_set().contains(a) implies seq_to_set(pushed).contains(#[trigger] f(a)) by {}
                assert forall |b: int| (#[trigger] seq_to_set(pushed).contains(b)) implies exists |a: u64| pushed.to_set().contains(a) && f(a) == b by {}
                vstd::set_lib::lemma_map_size(pushed.to_set(), seq_to_set(pushed), f);
                vstd::seq_lib::seq_to_set_is_finite(old_seq);
                old_seq.to_set().lemma_map_finite(f);
                old_seq.unique_seq_to_set();
                assert forall |a: u64| old_seq.to_set().contains(a) implies seq_to_set(old_seq).contains(#[trigger] f(a)) by {}
                assert forall |b: int| (#[trigger] seq_to_set(old_seq).contains(b)) implies exists |a: u64| old_seq.to_set().contains(a) && f(a) == b by {}
                vstd::set_lib::lemma_map_size(old_seq.to_set(), seq_to_set(old_seq), f);
            }
        }
        self.locked_mutex_count = self.locked_mutex_count + 1;
        self.locked_mutex_set.push(address);
    }
