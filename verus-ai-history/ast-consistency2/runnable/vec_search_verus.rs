fn vec_search(v: &Vec<i64>, target: i64) -> (result: (bool, usize))
    ensures
        result.0 == (exists|i: int| 0 <= i < v@.len() && #[trigger] v@[i] == target),
        result.0 ==> (0 <= result.1 < v@.len() && v@[result.1 as int] == target),
{
    let mut i: usize = 0;
    while i < v.len()
        invariant
            0 <= i <= v@.len(),
            forall|j: int| 0 <= j < i as int ==> #[trigger] v@[j] != target,
        decreases v@.len() - i,
    {
        if v[i] == target {
            return (true, i);
        }
        i = i + 1;
    }
    (false, 0)
}
