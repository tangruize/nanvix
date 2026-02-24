    pub fn from_sleeping(
        pid: u64,
        sleeping_ids: Vec<u64>,
        interrupted_ids: Vec<u64>,
        zombie_ids: Vec<u64>,
    ) -> (result: InterruptedProcess)
        requires
            interrupted_ids@.len() >= 1,
            Self::spec_no_duplicates(interrupted_ids@),
            Self::spec_no_duplicates(sleeping_ids@),
            Self::spec_no_duplicates(zombie_ids@),
            Self::spec_seqs_disjoint(interrupted_ids@, sleeping_ids@),
            Self::spec_seqs_disjoint(interrupted_ids@, zombie_ids@),
            Self::spec_seqs_disjoint(sleeping_ids@, zombie_ids@),
        ensures
            result@.pid == pid as int,
            result@.sleeping_thread_ids =~= spec_u64_seq_as_int(sleeping_ids@),
            result@.interrupted_thread_ids =~= spec_u64_seq_as_int(interrupted_ids@),
            result@.zombie_thread_ids =~= spec_u64_seq_as_int(zombie_ids@),
            result.wf(),
    {
        proof {
            reveal(InterruptedProcess::wf);
        }
        InterruptedProcess {
            pid,
            sleeping_thread_ids: sleeping_ids,
            interrupted_thread_ids: interrupted_ids,
            zombie_thread_ids: zombie_ids,
        }
    }
