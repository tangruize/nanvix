    fn drop(&mut self) {
        debug!("{:?}", &self);
        while let Some(kpage) = self.kpages.pop() {
            drop(kpage);
        }
    }
