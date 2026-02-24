pub fn layout_to_slab_size(size: usize) -> (result: Result<SlabSize, Error>)
    ensures
        result is Ok ==> {
            let slab_size = result->Ok_0;
            &&& size > 0
            &&& size as int <= slab_size.spec_as_int()
            &&& spec_layout_to_slab_size(size as int) == Some(slab_size)
        },
        result is Err ==> spec_layout_to_slab_size(size as int).is_none(),
{
    match size {
        1..=8 => Ok(SlabSize::Slab8),
        9..=16 => Ok(SlabSize::Slab16),
        17..=32 => Ok(SlabSize::Slab32),
        33..=64 => Ok(SlabSize::Slab64),
        65..=128 => Ok(SlabSize::Slab128),
        129..=256 => Ok(SlabSize::Slab256),
        257..=512 => Ok(SlabSize::Slab512),
        4096 => Ok(SlabSize::Slab4096),
        _ => Err(Error::new(ErrorCode::InvalidArgument, "unsupported allocation size")),
    }
}
