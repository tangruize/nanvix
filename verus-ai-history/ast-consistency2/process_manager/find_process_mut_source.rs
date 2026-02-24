    fn find_process_mut(&mut self, pid: ProcessIdentifier) -> Result<ProcessRefMut<'_>, Error> {
        if self.get_running_mut().state().pid() == pid {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self.suspended.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self.zombies.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
