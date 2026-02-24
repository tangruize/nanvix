    fn forge_user_context(
        mm: &mut VirtMemoryManager,
        vmem: &mut Vmem,
        args: &ThreadCreateArgs,
        enable_interrupts: bool,
    ) -> Result<(KernelStack, ContextInformation), Error> {
        trace!("args={args:?}, enable_interrupts={enable_interrupts:?}",);

        unsafe extern "C" {
            pub fn __leave_kernel_to_user_mode();
        }

        // Assert pre-conditions (these should have been checked by the caller).
        debug_assert!(Vmem::is_user_region(args.user_stack_base, args.user_stack_size));
        debug_assert!(Vmem::is_user_addr(args.user_fn));

        let kernel_func: VirtualAddress =
            VirtualAddress::from_raw_value(__leave_kernel_to_user_mode as usize);

        // Alloc kernel pages for the kernel stack. If we fail beyond this point, `kernel_stack`
        // gets dropped as soon as we exit this scope and underlying pages are released.
        let kernel_stack: KernelStack = KernelStack::new(mm)?;

        let cr3: u32 = vmem.pgdir().physical_address()?.into_raw_value() as u32;
        let esp: u32 = unsafe {
            hal::arch::forge_user_stack(
                kernel_stack.top().into_raw_value() as *mut u8,
                args.user_stack_base.into_raw_value() + args.user_stack_size,
                args.user_fn.into_raw_value(),
                args.user_fn_arg0,
                args.user_fn_arg1,
                kernel_func.into_raw_value(),
                enable_interrupts,
            )
        } as u32;
        let esp0: u32 = kernel_stack.top().into_raw_value() as u32;

        trace!("cr3={:#x}, esp={:#x}, ebp={:#x}", cr3, esp, esp0);
        let context: ContextInformation = ContextInformation::new(cr3, esp, esp0);

        Ok((kernel_stack, context))
    }
