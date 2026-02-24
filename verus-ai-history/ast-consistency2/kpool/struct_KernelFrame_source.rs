pub struct KernelFrame {
    kpool: Rc<RefCell<KpoolInner>>,
    base: FrameAddress,
}
