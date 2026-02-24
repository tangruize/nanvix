fn vec_remove_at(v: &Vec<u64>, skip: usize) -> (result: Vec<u64>)
    requires
        skip < v@.len(),
    ensures
        result@ =~= v@.subrange(0, skip as int).add(
            v@.subrange(skip as int + 1, v@.len() as int)),
{
    let mut result: Vec<u64> = Vec::new();
    let len: usize = v.len();
    let mut i: usize = 0;
    while i < skip
        invariant
            0 <= i <= skip,
            skip < len,
            len == v@.len(),
            result@ =~= v@.subrange(0, i as int),
        decreases skip - i,
    {
        result.push(v[i]);
        proof {
            assert(v@.subrange(0, i as int).push(v@[i as int])
                =~= v@.subrange(0, (i + 1) as int));
        }
        i = i + 1;
    }
    proof {
        assert(v@.subrange(skip as int + 1, (skip + 1) as int).len() == 0);
        assert(v@.subrange(0, skip as int).add(
            v@.subrange(skip as int + 1, (skip + 1) as int))
            =~= v@.subrange(0, skip as int));
    }
    i = skip + 1;
    while i < len
        invariant
            skip < len,
            len == v@.len(),
            skip as int + 1 <= i as int,
            i <= len,
            result@ =~= v@.subrange(0, skip as int).add(
                v@.subrange(skip as int + 1, i as int)),
        decreases len - i,
    {
        result.push(v[i]);
        proof {
            assert(v@.subrange(skip as int + 1, i as int).push(v@[i as int])
                =~= v@.subrange(skip as int + 1, (i + 1) as int));
            assert(v@.subrange(0, skip as int).add(
                v@.subrange(skip as int + 1, i as int)).push(v@[i as int])
                =~= v@.subrange(0, skip as int).add(
                    v@.subrange(skip as int + 1, (i + 1) as int)));
        }
        i = i + 1;
    }
    result
}
