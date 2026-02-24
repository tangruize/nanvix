    pub fn post_message(
        &mut self,
        receiver: MessageReceiver,
        message: Message,
    ) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = match receiver.as_id() {
            Ok(pid) => pm.find_process_mut(pid)?,
            Err(tid) => pm.find_process_by_tid(tid)?,
        };
        process.state_mut().post_message(message);
        pm.number_buffered_messages += 1;
        Ok(())
    }
