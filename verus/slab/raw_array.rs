// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Raw Array Storage
//==================================================================================================

use vstd::prelude::*;
use crate::error::{Error, ErrorCode};

use std::{
    alloc::*,
    ptr,
};

#[derive(Debug)]
enum RawArrayStorage<T> {
    /// A storage area that is managed by GlobalAlloc.
    Managed { ptr: ptr::NonNull<T>, len: usize },
    /// A storage area that is not managed by GlobalAlloc.
    Unmanaged { ptr: ptr::NonNull<T>, len: usize },
}

impl<T> RawArrayStorage<T> {
    fn new_managed(len: usize) -> Result<RawArrayStorage<T>, Error> {
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid length"));
        }
        let layout: Layout = match Layout::array::<T>(len) {
            Ok(layout) => layout,
            Err(_) => return Err(Error::new(ErrorCode::InvalidArgument, "invalid layout")),
        };
        let ptr: ptr::NonNull<T> = {
            let ptr: *mut u8 = unsafe { alloc(layout) };
            match ptr::NonNull::new(ptr as *mut T) {
                Some(p) => p,
                None => return Err(Error::new(ErrorCode::OutOfMemory, "out of memory")),
            }
        };
        unsafe { ptr::write_bytes(ptr.as_ptr(), 0, len) };
        Ok(RawArrayStorage::Managed { ptr, len })
    }

    unsafe fn new_unmanaged(ptr: *mut T, len: usize) -> Result<RawArrayStorage<T>, Error> {
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid length"));
        }
        if ptr.wrapping_add(len) < ptr {
            return Err(Error::new(ErrorCode::InvalidArgument, "wrapping memory region"));
        }
        let ptr: ptr::NonNull<T> = match ptr::NonNull::new(ptr) {
            Some(ptr) => ptr,
            None => return Err(Error::new(ErrorCode::InvalidArgument, "invalid pointer")),
        };
        ptr::write_bytes(ptr.as_ptr(), 0, len);
        Ok(RawArrayStorage::Unmanaged { ptr, len })
    }
}

verus! {

#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
#[verifier::external_body]
struct ExRawArrayStorage<T>(RawArrayStorage<T>);

#[derive(Debug)]
#[verifier::reject_recursive_types(T)]
pub struct RawArray<T> {
    storage: RawArrayStorage<T>,
}

impl<T> View for RawArray<T> {
    type V = Seq<T>;
    uninterp spec fn view(&self) -> Seq<T>;
}

pub uninterp spec fn is_zero<T>(i: T) -> bool;
pub axiom fn axiom_u8_zero_is_0(t: u8) requires is_zero(t) ensures t == 0;

impl<T> RawArray<T> {
    #[verifier::external_body]
    pub fn len(&self) -> (result: usize)
        ensures result == self@.len()
    {
        match &self.storage {
            RawArrayStorage::Managed { len, .. } | RawArrayStorage::Unmanaged { len, .. } => *len,
        }
    }

    /// Creates a new managed RawArray with the given length.
    #[verifier::external_body]
    pub fn new(len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
        ensures
            result is Ok ==> {
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
    {
        Ok(RawArray {
            storage: RawArrayStorage::new_managed(len)?,
        })
    }

    /// Creates a new unmanaged RawArray from raw pointer and length.
    #[verifier::external_body]
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
        ensures
            result is Ok ==> {
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
    {
        Ok(RawArray {
            storage: RawArrayStorage::new_unmanaged(ptr, len)?,
        })
    }

    /// Creates a new unmanaged RawArray from raw address (usize) and length.
    /// This is a helper for Verus verification since Verus doesn't support usize->ptr casts.
    #[verifier::external_body]
    pub unsafe fn from_raw_addr(addr: usize, len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
            addr > 0,
        ensures
            result is Ok ==> {
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
    {
        Self::from_raw_parts(addr as *mut T, len)
    }
}

} // verus!
