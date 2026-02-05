// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # RawArray - Implementation
//!
//! This file contains the implementation code for raw arrays.

use crate::libs::error::{
    Error,
    ErrorCode,
};
use vstd::prelude::*;

use std::{
    alloc::*,
    ptr,
    slice,
};

// Include specifications (spec functions, View trait, invariants).
include!("lib.spec.rs");

// Include proofs (lemmas).
include!("lib.proof.rs");

//==================================================================================================
// RawArrayStorage - Implementation Detail (outside verus!)
//==================================================================================================

enum RawArrayStorage<T> {
    Managed { ptr: ptr::NonNull<T>, len: usize },
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

    fn get_mut(&mut self) -> &mut [T] {
        match self {
            RawArrayStorage::Managed { ptr, len } => unsafe {
                slice::from_raw_parts_mut(ptr.as_ptr(), *len)
            },
            RawArrayStorage::Unmanaged { ptr, len } => unsafe {
                slice::from_raw_parts_mut(ptr.as_ptr(), *len)
            },
        }
    }

    fn get(&self) -> &[T] {
        match self {
            RawArrayStorage::Managed { ptr, len } => unsafe {
                slice::from_raw_parts(ptr.as_ptr(), *len)
            },
            RawArrayStorage::Unmanaged { ptr, len } => unsafe {
                slice::from_raw_parts(ptr.as_ptr(), *len)
            },
        }
    }

    fn storage_len(&self) -> usize {
        match self {
            RawArrayStorage::Managed { len, .. } => *len,
            RawArrayStorage::Unmanaged { len, .. } => *len,
        }
    }
}

impl<T> Drop for RawArrayStorage<T> {
    fn drop(&mut self) {
        match self {
            RawArrayStorage::Managed { ptr, len } => {
                if let Ok(layout) = Layout::array::<T>(*len) {
                    unsafe { dealloc(ptr.as_ptr() as *mut u8, layout); }
                }
            },
            RawArrayStorage::Unmanaged { .. } => (),
        }
    }
}

//==================================================================================================
// RawArray - Main Type
//==================================================================================================

verus! {

// External type specification for RawArrayStorage.
#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
#[verifier::external_body]
pub struct ExRawArrayStorage<T>(RawArrayStorage<T>);

/// A fixed-size array backed by raw memory.
#[verifier::reject_recursive_types(T)]
pub struct RawArray<T> {
    storage: RawArrayStorage<T>,
}

//==================================================================================================
// Constructor Functions
//==================================================================================================

impl<T> RawArray<T> {
    /// Constructs a new managed array with all elements zero-initialized.
    #[verifier::external_body]
    pub fn new(len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
        ensures
            result is Ok ==> {
                &&& result->Ok_0.inv()
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
            result is Err ==> result->Err_0.code == ErrorCode::OutOfMemory,
    {
        Ok(RawArray { storage: RawArrayStorage::new_managed(len)? })
    }

    /// Constructs a new unmanaged array from raw memory.
    #[verifier::external_body]
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
        ensures
            result is Ok ==> {
                &&& result->Ok_0.inv()
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
            result is Err ==> result->Err_0.code == ErrorCode::InvalidArgument,
    {
        Ok(RawArray { storage: RawArrayStorage::new_unmanaged(ptr, len)? })
    }

    /// Constructs a new raw array from a raw address.
    #[verifier::external_body]
    pub unsafe fn from_raw_addr(addr: usize, len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
            addr > 0,
        ensures
            result is Ok ==> {
                &&& result->Ok_0.inv()
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
            result is Err ==> result->Err_0.code == ErrorCode::InvalidArgument,
    {
        Self::from_raw_parts(addr as *mut T, len)
    }
}

//==================================================================================================
// Accessor Functions
//==================================================================================================

impl<T> RawArray<T> {
    /// Sets the element at index to value.
    #[verifier::external_body]
    pub fn set(&mut self, index: usize, value: T)
        requires
            old(self).in_bounds(index as int),
        ensures
            self@.len() == old(self)@.len(),
            self@[index as int] == value,
            forall|i: int| 0 <= i < self@.len() && i != index as int
                ==> self@[i] == old(self)@[i],
    {
        self.storage.get_mut()[index] = value;
    }

    /// Returns the length of the array.
    #[verifier::external_body]
    pub fn len(&self) -> (result: usize)
        ensures
            result == self@.len(),
    {
        self.storage.storage_len()
    }

    /// Gets a reference to the element at index.
    #[verifier::external_body]
    pub fn get(&self, index: usize) -> (result: &T)
        requires
            self.in_bounds(index as int),
        ensures
            *result == self@[index as int],
    {
        &self.storage.get()[index]
    }
}

//==================================================================================================
// Deref Implementation
//==================================================================================================

impl<T> core::ops::Deref for RawArray<T> {
    type Target = [T];

    #[verifier::external_body]
    fn deref(&self) -> (result: &Self::Target)
        ensures
            result@ == self@,
    {
        self.storage.get()
    }
}

} // verus!
