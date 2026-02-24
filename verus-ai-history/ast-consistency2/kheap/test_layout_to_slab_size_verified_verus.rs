fn test_layout_to_slab_size_verified()
{
    // Test each size category.
    let r1: Result<SlabSize, Error> = layout_to_slab_size(1);
    assert(r1 is Ok);
    assert(r1->Ok_0 == SlabSize::Slab8);

    let r8: Result<SlabSize, Error> = layout_to_slab_size(8);
    assert(r8 is Ok);
    assert(r8->Ok_0 == SlabSize::Slab8);

    let r9: Result<SlabSize, Error> = layout_to_slab_size(9);
    assert(r9 is Ok);
    assert(r9->Ok_0 == SlabSize::Slab16);

    let r16: Result<SlabSize, Error> = layout_to_slab_size(16);
    assert(r16 is Ok);
    assert(r16->Ok_0 == SlabSize::Slab16);

    let r17: Result<SlabSize, Error> = layout_to_slab_size(17);
    assert(r17 is Ok);
    assert(r17->Ok_0 == SlabSize::Slab32);

    let r64: Result<SlabSize, Error> = layout_to_slab_size(64);
    assert(r64 is Ok);
    assert(r64->Ok_0 == SlabSize::Slab64);

    let r128: Result<SlabSize, Error> = layout_to_slab_size(128);
    assert(r128 is Ok);
    assert(r128->Ok_0 == SlabSize::Slab128);

    let r256: Result<SlabSize, Error> = layout_to_slab_size(256);
    assert(r256 is Ok);
    assert(r256->Ok_0 == SlabSize::Slab256);

    let r512: Result<SlabSize, Error> = layout_to_slab_size(512);
    assert(r512 is Ok);
    assert(r512->Ok_0 == SlabSize::Slab512);

    let r4096: Result<SlabSize, Error> = layout_to_slab_size(4096);
    assert(r4096 is Ok);
    assert(r4096->Ok_0 == SlabSize::Slab4096);

    // Invalid sizes.
    let r0: Result<SlabSize, Error> = layout_to_slab_size(0);
    assert(r0 is Err);

    let r513: Result<SlabSize, Error> = layout_to_slab_size(513);
    assert(r513 is Err);

    let r1000: Result<SlabSize, Error> = layout_to_slab_size(1000);
    assert(r1000 is Err);
}
