    fn drop(&mut self) {
        if let Err(e) = self.kpool.borrow_mut().free(self.base) {
            error!("failed to free kernel page pool: {:?}", e)
        }
    }
