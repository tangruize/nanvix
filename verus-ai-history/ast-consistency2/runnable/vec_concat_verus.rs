fn vec_concat(v1: &Vec<i64>, v2: &Vec<i64>) -> (result: Vec<i64>)
    ensures
        result@ == v1@.add(v2@),
        result@.len() == v1@.len() + v2@.len(),
{
    let mut result: Vec<i64> = Vec::new();
    let mut i: usize = 0;
    while i < v1.len()
        invariant
            0 <= i <= v1@.len(),
            result@ == v1@.subrange(0, i as int),
            result@.len() == i as nat,
        decreases v1@.len() - i,
    {
        result.push(v1[i]);
        i = i + 1;
    }
    proof {
        assert(result@ =~= v1@.subrange(0, v1@.len() as int));
        assert(v1@.subrange(0, v1@.len() as int) =~= v1@);
    }
    let mut j: usize = 0;
    while j < v2.len()
        invariant
            0 <= j <= v2@.len(),
            result@ == v1@.add(v2@.subrange(0, j as int)),
            result@.len() == v1@.len() + j as nat,
        decreases v2@.len() - j,
    {
        result.push(v2[j]);
        proof {
            assert(v2@.subrange(0, j as int).push(v2@[j as int])
                =~= v2@.subrange(0, j + 1));
        }
        j = j + 1;
    }
    proof {
        assert(v2@.subrange(0, v2@.len() as int) =~= v2@);
    }
    result
}
