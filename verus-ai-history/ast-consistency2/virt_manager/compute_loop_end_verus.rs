pub fn compute_loop_end(start: usize, size: usize) -> (result: usize)
    requires
        size > 0,
        start as int + size as int - 1 <= usize::MAX as int,
    ensures
        result as int == spec_loop_end(start as int, size as int),
{
    start + (size - 1)
}
