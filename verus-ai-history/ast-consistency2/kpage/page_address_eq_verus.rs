pub fn page_address_eq(a: &PageAddress, b: &PageAddress) -> (result: bool)
    requires
        a.inv(),
        b.inv(),
    ensures
        result == (a@.raw_value() == b@.raw_value()),
        result == a.eq_spec(b),
{
    a.raw_addr == b.raw_addr
}
