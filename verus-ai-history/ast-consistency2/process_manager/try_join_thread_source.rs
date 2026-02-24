    fn try_join_thread(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    ) -> Result<ZombieThread, Result<Condvar, Error>> {
        match self.find_process_mut(pid).map_err(Err)? {
            ProcessRefMut::Running(process) => process.try_join_thread(tid),
            ProcessRefMut::Runnable(_) => {
                let reason: &str = "process is runnable";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Sleeping(_) => {
                let reason: &str = "process is sleeping";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Interrupted(_) => {
                let reason: &str = "process is interrupted";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Zombie(_) => {
                let reason: &str = "process is a zombie";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
        }
    }
