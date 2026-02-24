fn vec_remove_at(v: &Vec<i64>, skip_idx: usize) -> (result: Vec<i64>)
    requires
        0 <= skip_idx < v@.len(),
    ensures
        result@.len() == v@.len() - 1,
        result@ == v@.subrange(0, skip_idx as int).add(
            v@.subrange(skip_idx as int + 1, v@.len() as int)),
{
    let mut result: Vec<i64> = Vec::new();
    let mut j: usize = 0;
    while j < v.len()
        invariant
            0 <= j <= v@.len(),
            0 <= skip_idx < v@.len(),
            j <= skip_idx ==> result@.len() == j as nat,
            j > skip_idx ==> result@.len() == (j - 1) as nat,
            j <= skip_idx ==> result@ == v@.subrange(0, j as int),
            j > skip_idx ==> result@ == v@.subrange(0, skip_idx as int).add(
                v@.subrange(skip_idx as int + 1, j as int)),
        decreases v@.len() - j,
    {
        if j != skip_idx {
            result.push(v[j]);
        }
        j = j + 1;
    }
    result
}
