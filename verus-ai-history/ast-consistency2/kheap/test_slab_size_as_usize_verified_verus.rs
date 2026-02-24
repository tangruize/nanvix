fn test_slab_size_as_usize_verified()
{
    let s8: SlabSize = SlabSize::Slab8;
    let v8: usize = s8.as_usize();
    proof { assert(v8 == 8); }

    let s16: SlabSize = SlabSize::Slab16;
    let v16: usize = s16.as_usize();
    proof { assert(v16 == 16); }

    let s32: SlabSize = SlabSize::Slab32;
    let v32: usize = s32.as_usize();
    proof { assert(v32 == 32); }

    let s64: SlabSize = SlabSize::Slab64;
    let v64: usize = s64.as_usize();
    proof { assert(v64 == 64); }

    let s128: SlabSize = SlabSize::Slab128;
    let v128: usize = s128.as_usize();
    proof { assert(v128 == 128); }

    let s256: SlabSize = SlabSize::Slab256;
    let v256: usize = s256.as_usize();
    proof { assert(v256 == 256); }

    let s512: SlabSize = SlabSize::Slab512;
    let v512: usize = s512.as_usize();
    proof { assert(v512 == 512); }

    let s4096: SlabSize = SlabSize::Slab4096;
    let v4096: usize = s4096.as_usize();
    proof { assert(v4096 == 4096); }
}
