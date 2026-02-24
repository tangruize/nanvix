    fn find_process(&self, pid: ProcessIdentifier) -> Result<ProcessRef<'_>, Error> {
        if self.get_running().state().pid() == pid {
            Ok(ProcessRef::Running(self.get_running()))
        } else if let Some(process) = self.ready.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Runnable(process))
        } else if let Some(process) = self.suspended.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Interrupted(process))
        } else if let Some(process) = self.zombies.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
