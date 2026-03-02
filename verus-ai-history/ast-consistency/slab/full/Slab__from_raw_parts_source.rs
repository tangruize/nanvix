    pub unsafe fn from_raw_parts(
        addr: *mut u8,
        len: usize,
        block_size: usize,
    ) -> Result<Slab, Error> {
        // Check if length is invalid valid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid slab length"));
        }

        // Check if the memory region wraps around.
        if addr.wrapping_add(len) < addr {
            return Err(Error::new(ErrorCode::InvalidArgument, "wrapping memory region"));
        }

        // Check if block size is valid.
        if block_size == 0 || block_size >= i32::MAX as usize || block_size > len {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid block size"));
        }

        // Check if the `block_size` is a power of two.
        if block_size & (block_size - 1) != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "block size is not a power of two"));
        }

        // Check if `start_addr` is aligned to `block_size`.
        if !(addr as usize).is_multiple_of(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned start address"));
        }

        // Compute layout of the slab allocator.
        let total_num_blocks: usize = len / block_size;
        // info!("total number of blocks: {:?}", total_num_blocks);
        if !total_num_blocks.is_multiple_of(u8::BITS as usize) {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid number of blocks"));
        }
        let index_len: usize = total_num_blocks / u8::BITS as usize;
        // info!("index length: {:?}", index_len);
        let num_index_blocks: usize = (index_len / block_size)
            + if index_len.is_multiple_of(block_size) {
                0
            } else {
                1
            };
        // info!("number of index blocks: {:?}", num_index_blocks);
        if num_index_blocks > total_num_blocks {
            return Err(Error::new(ErrorCode::InvalidArgument, "insufficient blocks for index"));
        }
        let num_data_blocks: usize = total_num_blocks - num_index_blocks;
        // info!("number of data blocks: {:?}", num_data_blocks);
        let data_addr: *mut u8 = addr.add(num_index_blocks * block_size);

        // Check if `data_addr` is aligned to `block_size`.
        if !(data_addr as usize).is_multiple_of(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned data address"));
        }

        // Instantiate index.
        let storage: RawArray<u8> = RawArray::from_raw_parts(addr, index_len)?;
        let mut index: Bitmap = Bitmap::from_raw_array(storage)?;

        // NOTE: The index is initialized with all blocks free, thus if we fail beyond this point
        // the memory region is left in a modified state.

        // Initialize index.
        for i in 0..num_index_blocks {
            index.set(i)?;
        }

        Ok(Slab {
            num_index_blocks,
            num_data_blocks,
            block_size,
            data_addr,
            index,
        })
    }
