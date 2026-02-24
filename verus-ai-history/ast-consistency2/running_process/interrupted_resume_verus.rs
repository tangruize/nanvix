fn interrupted_resume(ip: InterruptedProcess) -> (result: RunnableProcess)
    requires
        ip.inv(),
    ensures
        result.spec_pid() == ip.spec_pid(),
        result.inv(),
        // Sleeping threads are passed through resume() into the result.
        result.sleeping_thread_ids@ == ip.sleeping_thread_ids@,
        // Zombie threads are passed through resume() into the result.
        result.zombie_thread_ids@ == ip.zombie_thread_ids@,
        // Exactly one interrupted thread became ready: the front of the deque.
        result.ready_thread_ids@.len() == 1,
        result.ready_thread_ids@[0] == ip.interrupted_thread_ids@[0],
        // Remaining interrupted threads are the tail (IDs preserved exactly).
        result.interrupted_thread_ids@ ==
            ip.interrupted_thread_ids@.subrange(
                1, ip.interrupted_thread_ids@.len() as int),
        // Thread count conservation (follows from the above).
        result.ready_thread_ids@.len() + result.interrupted_thread_ids@.len()
            == ip.interrupted_thread_ids@.len(),
{
    unimplemented!()
}
