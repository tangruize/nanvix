    pub fn alloc_kpage(&mut self) -> (result: Result<KernelPage, Error>)
        requires
            old(self).inv(),
            old(self)@.has_kpool_capacity(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& result.unwrap().inv()
            },
    {
        let kframe: KernelFrame = self.kpool.alloc()?;
        proof {
            // Connect kpool.alloc() postcondition to KernelPage::new() precondition.
            assert(kframe.spec_is_aligned());
        }
        let kpage: KernelPage = KernelPage::new(kframe);
        proof {
            // KernelPage::new() postcondition guarantees result.inv().
            assert(kpage.inv());
        }
        Ok(kpage)
    }
