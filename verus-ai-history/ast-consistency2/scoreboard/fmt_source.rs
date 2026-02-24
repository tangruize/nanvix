    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "KcallArgs {{ pid: {:?}, tid: {:?}, number: {}, arg0: {:#010x}, arg1: {:#010x}, arg2: \
             {:#010x}, arg3: {:#010x} }}",
            self.pid, self.tid, self.number, self.arg0, self.arg1, self.arg2, self.arg3
        )
    }
