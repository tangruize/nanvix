    pub unsafe fn handled(&mut self, ret: KcallResult) -> Result<(), Error> {
        self.ret = ret;
        self.handled.up()
    }
