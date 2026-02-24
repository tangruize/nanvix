pub struct VirtMemoryManager {
    /// Physical memory manager.
    physman: Rc<RefCell<PhysMemoryManager>>,
}
