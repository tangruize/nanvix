    pub fn copy_from_user_unaligned(
        &self,
        dst: VirtualAddress,
        src: VirtualAddress,
        size: usize,
    ) -> Result<(), Error> {
        // Check if size is invalid.
        if size == 0 {
            let reason: &str = "zero-length copy";
            error!(
                "copy_from_user_unaligned(): {reason} (dst={dst:?}, src={src:?}, size={size:?})"
            );
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if the source memory region lies entirely in user space.
        // NOTE: This check is sufficient because, by design, the kernel and user spaces do not
        // share any memory page.
        if !Self::is_user_region(src, size) {
            let reason: &str = "source memory region does not lie entirely in user space";
            error!(
                "copy_from_user_unaligned(): {reason} (dst={dst:?}, src={src:?}, size={size:?})"
            );
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }

        // Check if the destination region lies entirely in kernel space.
        // NOTE: This check is sufficient because, by design, the kernel and user spaces do not
        // share any memory page.
        if !Self::is_kernel_region(dst, size) {
            let reason: &str = "destination region does not lie entirely in kernel space";
            error!(
                "copy_from_user_unaligned(): {reason} (dst={dst:?}, src={src:?}, size={size:?})"
            );
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }

        let copy_from_user_unaligned_impl = |dry_run: bool,
                                             mut src: VirtualAddress,
                                             mut dst: VirtualAddress,
                                             mut size: usize|
         -> Result<(), Error> {
            while size > 0 {
                let vaddr: PageAligned<VirtualAddress> =
                    PageAligned::from_address(src.align_down(PAGE_ALIGNMENT))?;
                let offset: usize = src.into_raw_value() - vaddr.into_raw_value();
                let copy_size: usize = usize::min(size, mem::PAGE_SIZE - offset);

                let src_frame: FrameAddress = self.find_user_frame(vaddr)?;

                if !dry_run {
                    // Copy memory from user space to kernel space.
                    // SAFETY: The following conditions are guaranteed:
                    // - `dst.into_raw_value()` is a valid kernel-space address for `copy_size` bytes.
                    // - `src_frame.into_raw_value() + offset` is a valid user-space address for `copy_size` bytes.
                    // - Both regions are non-overlapping and accessible for the operation.
                    unsafe {
                        __phys_memcpy(
                            dst.into_raw_value() as *mut u8,
                            (src_frame.into_raw_value() + offset) as *const u8,
                            copy_size,
                        )
                    };
                }

                size -= copy_size;
                src = VirtualAddress::new(src.into_raw_value() + copy_size);
                dst = VirtualAddress::new(dst.into_raw_value() + copy_size);
            }

            Ok(())
        };

        // Run in dry-run mode first to check for errors.
        copy_from_user_unaligned_impl(true, src, dst, size)?;
        // Run in normal mode to effectively copy data.
        copy_from_user_unaligned_impl(false, src, dst, size)?;

        Ok(())
    }
