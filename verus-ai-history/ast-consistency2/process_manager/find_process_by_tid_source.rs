    fn find_process_by_tid(&mut self, tid: ThreadIdentifier) -> Result<ProcessRefMut<'_>, Error> {
        if self.get_running_mut().find_thread(tid).is_some() {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.find_thread(tid).is_some()) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self
            .suspended
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self
            .interrupted
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self
            .zombies
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "thread not found";
            error!("{reason} (tid={tid:?})");
            Err(Error::new(ErrorCode::NoSuchEntry, reason))
        }
    }
