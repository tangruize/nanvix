fn vec_push_all(dst: &mut Vec<u64>, src: &Vec<u64>)
    ensures
        dst@ =~= old(dst)@.add(src@),
{
    let src_len: usize = src.len();
    let mut i: usize = 0;
    while i < src_len
        invariant
            0 <= i <= src_len,
            src_len == src@.len(),
            dst@ =~= old(dst)@.add(src@.subrange(0, i as int)),
        decreases src_len - i,
    {
        dst.push(src[i]);
        proof {
            assert(src@.subrange(0, i as int).push(src@[i as int])
                =~= src@.subrange(0, (i + 1) as int));
        }
        i = i + 1;
    }
    proof {
        assert(src@.subrange(0, src_len as int) =~= src@);
    }
}
