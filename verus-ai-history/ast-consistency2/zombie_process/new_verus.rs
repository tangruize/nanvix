    pub fn new(
        pid: u64,
        zombie_ids: Vec<u64>,
        status: i64,
        zombie_count: u64,
    ) -> (result: ZombieProcess)
        requires
            zombie_count as nat == zombie_ids@.len(),
            zombie_ids@.len() >= 1,
            forall|i: int, j: int| 0 <= i < j < zombie_ids@.len()
                ==> zombie_ids@[i] != zombie_ids@[j],
        ensures
            result@.pid == pid as int,
            result@.zombie_thread_ids =~= Seq::new(zombie_ids@.len(), |i: int| zombie_ids@[i] as int),
            result@.status == status as int,
            result.inv(),
    {
        ZombieProcess {
            pid,
            zombie_thread_ids: zombie_ids,
            status,
            zombie_count,
        }
    }
