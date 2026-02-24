struct CondvarInner {
    sleeping: RefCell<LinkedList<(ProcessIdentifier, ThreadIdentifier)>>,
}
