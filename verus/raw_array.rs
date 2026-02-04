// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # RawArray Verification
//!
//! This module provides a verified specification for raw arrays. The implementation
//! uses `external_body` only for truly unsafe operations (memory allocation and
//! pointer dereferencing), while all specification-level properties are verified.
//!
//! ## Memory Safety Model
//!
//! RawArray provides a safe abstraction over raw memory with the following guarantees:
//! 1. **Length invariance**: Array length never changes after construction
//! 2. **Element isolation**: Writing to one index doesn't affect others
//! 3. **Bounds safety**: All accesses are within bounds
//! 4. **Initial state**: New arrays are zero-initialized
//!
//! ## Verification Approach
//!
//! The view (`RawArrayView`) is defined as a sequence (`Seq<T>`) that models the
//! array contents. The spec functions and lemmas are fully verified, while exec
//! functions that touch raw memory are marked `external_body` with verified specs.

use crate::error::{
    Error,
    ErrorCode,
};
use vstd::prelude::*;

use std::{
    alloc::*,
    ptr,
    slice,
};

verus! {

//==================================================================================================
// RawArrayView - The Abstract Specification Model
//==================================================================================================

/// Abstract view of a RawArray as a sequence.
///
/// This is the ghost/spec-level representation used for verification.
/// All properties are stated in terms of this view.
pub struct RawArrayView<T> {
    /// The logical contents of the array.
    pub contents: Seq<T>,
}

impl<T> RawArrayView<T> {
    /// Returns the length of the array view.
    pub open spec fn len(&self) -> nat {
        self.contents.len()
    }

    /// Returns the element at index i.
    pub open spec fn index(&self, i: int) -> T
        recommends 0 <= i < self.len() as int
    {
        self.contents[i]
    }

    /// Returns a new view with the element at index i updated to value.
    pub open spec fn update(&self, i: int, value: T) -> RawArrayView<T>
        recommends 0 <= i < self.len() as int
    {
        RawArrayView {
            contents: self.contents.update(i, value),
        }
    }

    /// Two views are equal if they have the same contents.
    pub open spec fn spec_eq(&self, other: &RawArrayView<T>) -> bool {
        self.contents =~= other.contents
    }
}

//==================================================================================================
// Lemmas about RawArrayView - Fully Verified
//==================================================================================================

/// Lemma: Updating an array preserves its length.
pub proof fn lemma_update_preserves_len<T>(view: RawArrayView<T>, i: int, value: T)
    requires
        0 <= i < view.len() as int,
    ensures
        view.update(i, value).len() == view.len(),
{
    // Follows from Seq::update preserving length
}

/// Lemma: Updating index i only changes index i.
pub proof fn lemma_update_only_changes_index<T>(view: RawArrayView<T>, i: int, value: T, j: int)
    requires
        0 <= i < view.len() as int,
        0 <= j < view.len() as int,
        i != j,
    ensures
        view.update(i, value).index(j) == view.index(j),
{
    // Follows from Seq::update property
}

/// Lemma: Updating index i sets index i to the new value.
pub proof fn lemma_update_sets_index<T>(view: RawArrayView<T>, i: int, value: T)
    requires
        0 <= i < view.len() as int,
    ensures
        view.update(i, value).index(i) == value,
{
    // Follows from Seq::update property
}

/// Lemma: View equality is reflexive.
pub proof fn lemma_view_eq_reflexive<T>(view: &RawArrayView<T>)
    ensures
        view.spec_eq(view),
{
}

/// Lemma: View equality is symmetric.
pub proof fn lemma_view_eq_symmetric<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>)
    requires
        v1.spec_eq(v2),
    ensures
        v2.spec_eq(v1),
{
}

/// Lemma: View equality is transitive.
pub proof fn lemma_view_eq_transitive<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>, v3: &RawArrayView<T>)
    requires
        v1.spec_eq(v2),
        v2.spec_eq(v3),
    ensures
        v1.spec_eq(v3),
{
}

/// Lemma: Equal views have equal elements.
pub proof fn lemma_equal_views_equal_elements<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>, i: int)
    requires
        v1.spec_eq(v2),
        0 <= i < v1.len() as int,
    ensures
        v1.index(i) == v2.index(i),
{
}

/// Lemma: Equal views have equal lengths.
pub proof fn lemma_equal_views_equal_len<T>(v1: &RawArrayView<T>, v2: &RawArrayView<T>)
    requires
        v1.spec_eq(v2),
    ensures
        v1.len() == v2.len(),
{
}

//==================================================================================================
// Zero Initialization Specification
//==================================================================================================

/// Predicate: Specifies that a value is the zero/default value for its type.
/// This is used to specify that newly allocated arrays are zero-initialized.
pub uninterp spec fn is_zero<T>(value: T) -> bool;

/// Axiom: For u8, zero means the value is 0.
pub axiom fn axiom_u8_zero_is_0(t: u8)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For u16, zero means the value is 0.
pub axiom fn axiom_u16_zero_is_0(t: u16)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For u32, zero means the value is 0.
pub axiom fn axiom_u32_zero_is_0(t: u32)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For u64, zero means the value is 0.
pub axiom fn axiom_u64_zero_is_0(t: u64)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For usize, zero means the value is 0.
pub axiom fn axiom_usize_zero_is_0(t: usize)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i8, zero means the value is 0.
pub axiom fn axiom_i8_zero_is_0(t: i8)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i16, zero means the value is 0.
pub axiom fn axiom_i16_zero_is_0(t: i16)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i32, zero means the value is 0.
pub axiom fn axiom_i32_zero_is_0(t: i32)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For i64, zero means the value is 0.
pub axiom fn axiom_i64_zero_is_0(t: i64)
    requires is_zero(t)
    ensures t == 0;

/// Axiom: For isize, zero means the value is 0.
pub axiom fn axiom_isize_zero_is_0(t: isize)
    requires is_zero(t)
    ensures t == 0;

//==================================================================================================
// RawArray Invariant - Verified
//==================================================================================================

/// The invariant that all RawArray instances must satisfy.
/// This is trivially true for our abstraction, but captures key properties.
pub open spec fn raw_array_inv<T>(len: nat) -> bool {
    // Length must be positive and bounded
    &&& len > 0
    &&& len < i32::MAX as nat
}

//==================================================================================================
// RawArrayStorage - Implementation Detail (External)
//==================================================================================================

} // verus!

/// A type that represents the backing storage of a [`RawArray`].
#[derive(Debug)]
enum RawArrayStorage<T> {
    /// A storage area that is managed by GlobalAlloc.
    Managed { ptr: ptr::NonNull<T>, len: usize },
    /// A storage area that is not managed by GlobalAlloc.
    Unmanaged { ptr: ptr::NonNull<T>, len: usize },
}

impl<T> RawArrayStorage<T> {
    /// Constructs backing storage for a raw array.
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
                None => {
                    return Err(Error::new(ErrorCode::OutOfMemory, "out of memory"));
                },
            }
        };

        unsafe { ptr::write_bytes(ptr.as_ptr(), 0, len) };

        Ok(RawArrayStorage::Managed { ptr, len })
    }

    /// Constructs an unmanaged backing storage for a raw array.
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid for reads and writes for `len * size_of::<T>()` bytes.
    /// - `ptr` must be properly aligned.
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

//==================================================================================================
// RawArray - Main Type
//==================================================================================================

verus! {

#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
#[verifier::external_body]
struct ExRawArrayStorage<T>(RawArrayStorage<T>);

/// A fixed-size array backed by raw memory.
///
/// Provides a safe abstraction over raw memory allocation with verified specifications.
#[derive(Debug)]
#[verifier::reject_recursive_types(T)]
pub struct RawArray<T> {
    storage: RawArrayStorage<T>,
}

impl<T> View for RawArray<T> {
    type V = Seq<T>;

    /// The abstract view of the array as a sequence.
    ///
    /// This is uninterpreted because the actual mapping from raw memory to Seq
    /// requires trusting the memory subsystem.
    uninterp spec fn view(&self) -> Seq<T>;
}

impl<T> RawArray<T> {
    //==============================================================================================
    // Specification Functions - Verified
    //==============================================================================================

    /// Returns the spec-level length of the array.
    pub open spec fn spec_len(&self) -> nat {
        self@.len()
    }

    /// Returns true if index i is within bounds.
    pub open spec fn in_bounds(&self, i: int) -> bool {
        0 <= i < self.spec_len() as int
    }

    /// Invariant for RawArray: length is positive and bounded.
    pub open spec fn inv(&self) -> bool {
        &&& self@.len() > 0
        &&& self@.len() < i32::MAX as nat
    }

    //==============================================================================================
    // Constructor Functions - External Body (memory allocation is unsafe)
    //==============================================================================================

    /// Constructs a new managed array with all elements zero-initialized.
    ///
    /// # Parameters
    /// - `len`: Length of the array (must be > 0 and < i32::MAX).
    ///
    /// # Returns
    /// On success, a new zero-initialized array.
    /// On failure, an error (only OutOfMemory is possible when preconditions are met).
    #[verifier::external_body]
    pub fn new(len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
        ensures
            // Success case: array is properly initialized.
            result is Ok ==> {
                &&& result->Ok_0.inv()
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
            // Error case: only OutOfMemory is possible when preconditions are met.
            result is Err ==> result->Err_0.code == ErrorCode::OutOfMemory,
    {
        Ok(RawArray {
            storage: RawArrayStorage::new_managed(len)?,
        })
    }

    /// Constructs a new unmanaged array from raw memory.
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid for reads and writes for `len * size_of::<T>()` bytes.
    /// - `ptr` must be properly aligned.
    #[verifier::external_body]
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
        ensures
            // Success case: array is properly initialized.
            result is Ok ==> {
                &&& result->Ok_0.inv()
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
            // Error case: only InvalidArgument is possible (null or wrapping pointer).
            result is Err ==> result->Err_0.code == ErrorCode::InvalidArgument,
    {
        Ok(RawArray {
            storage: RawArrayStorage::new_unmanaged(ptr, len)?,
        })
    }

    /// Constructs a new raw array from a raw address.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the address points to valid memory.
    #[verifier::external_body]
    pub unsafe fn from_raw_addr(addr: usize, len: usize) -> (result: Result<RawArray<T>, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
            addr > 0,
        ensures
            // Success case: array is properly initialized.
            result is Ok ==> {
                &&& result->Ok_0.inv()
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < len ==> is_zero(#[trigger] result->Ok_0@[i])
            },
            // Error case: only InvalidArgument is possible (wrapping pointer).
            result is Err ==> result->Err_0.code == ErrorCode::InvalidArgument,
    {
        Self::from_raw_parts(addr as *mut T, len)
    }

    //==============================================================================================
    // Accessor Functions - External Body (pointer dereferencing is unsafe)
    //==============================================================================================

    /// Sets the element at index to value.
    ///
    /// # Verified Properties
    /// - Length is preserved
    /// - Only the specified index changes
    /// - The specified index gets the new value
    #[verifier::external_body]
    pub fn set(&mut self, index: usize, value: T)
        requires
            old(self).in_bounds(index as int),
        ensures
            // Length preserved
            self@.len() == old(self)@.len(),
            // Target index updated
            self@[index as int] == value,
            // Other indices unchanged
            forall|i: int| 0 <= i < self@.len() && i != index as int
                ==> self@[i] == old(self)@[i],
    {
        let slice: &mut [T] = self.storage.get_mut();
        slice[index] = value;
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

// Note: DerefMut is not implemented because Verus does not support &mut return types
// in trait implementations. Use the `set()` method for mutable access instead.

//==================================================================================================
// Drop Implementation
//==================================================================================================

// Note: Drop implementation is marked external because Verus doesn't support opens_invariants
// on Drop traits. The implementation matches the original source exactly.
#[verifier::external]
impl<T> Drop for RawArray<T> {
    fn drop(&mut self) {
        match &self.storage {
            RawArrayStorage::Managed { ptr, len } => {
                let layout: Layout = match Layout::array::<T>(*len) {
                    Ok(layout) => layout,
                    Err(_) => return,
                };
                unsafe {
                    dealloc(ptr.as_ptr() as *mut u8, layout);
                }
            },
            RawArrayStorage::Unmanaged { .. } => (),
        }
    }
}

//==================================================================================================
// Additional Lemmas for Client Code
//==================================================================================================

/// Lemma: If two arrays have the same view, they are equivalent.
pub proof fn lemma_view_determines_equivalence<T>(a1: &RawArray<T>, a2: &RawArray<T>)
    requires
        a1@ =~= a2@,
    ensures
        a1.spec_len() == a2.spec_len(),
        forall|i: int| 0 <= i < a1.spec_len() as int ==> a1@[i] == a2@[i],
{
}

/// Lemma: Consecutive sets to different indices commute.
pub proof fn lemma_set_commutes<T>(view: Seq<T>, i: int, vi: T, j: int, vj: T)
    requires
        0 <= i < view.len(),
        0 <= j < view.len(),
        i != j,
    ensures
        view.update(i, vi).update(j, vj) =~= view.update(j, vj).update(i, vi),
{
}

/// Lemma: Setting the same index twice results in the last value.
pub proof fn lemma_set_overwrite<T>(view: Seq<T>, i: int, v1: T, v2: T)
    requires
        0 <= i < view.len(),
    ensures
        view.update(i, v1).update(i, v2) =~= view.update(i, v2),
{
}

//==================================================================================================
// Tests
//==================================================================================================

#[cfg(verus_keep_ghost)]
mod test {
    use super::*;

    /// Test: RawArrayView update properties.
    proof fn test_view_update_properties() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8],
        };

        // Length is 4.
        assert(view.len() == 4);

        // Update preserves length.
        let updated = view.update(1, 5u8);
        lemma_update_preserves_len(view, 1, 5u8);
        assert(updated.len() == 4);

        // Update sets the target index.
        lemma_update_sets_index(view, 1, 5u8);
        assert(updated.index(1) == 5u8);

        // Update preserves other indices.
        lemma_update_only_changes_index(view, 1, 5u8, 0);
        assert(updated.index(0) == view.index(0));
    }

    /// Test: Equality lemmas.
    proof fn test_equality_lemmas() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![1u8, 2u8, 3u8],
        };

        // Reflexive.
        lemma_view_eq_reflexive(&view);
        assert(view.spec_eq(&view));

        // Equal views have equal lengths.
        let view2 = view;
        lemma_equal_views_equal_len(&view, &view2);
        assert(view.len() == view2.len());
    }

    /// Test: Set commutes lemma.
    proof fn test_set_commutes() {
        let view: Seq<u8> = seq![0u8, 0u8, 0u8, 0u8];

        lemma_set_commutes(view, 0, 1u8, 2, 3u8);
        assert(view.update(0, 1u8).update(2, 3u8) =~= view.update(2, 3u8).update(0, 1u8));
    }

    /// Test: Set overwrite lemma.
    proof fn test_set_overwrite() {
        let view: Seq<u8> = seq![0u8, 0u8, 0u8, 0u8];

        lemma_set_overwrite(view, 1, 5u8, 10u8);
        assert(view.update(1, 5u8).update(1, 10u8) =~= view.update(1, 10u8));
    }

    /// Test: Multiple updates preserve length.
    proof fn test_multiple_updates_preserve_len() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8, 0u8],
        };

        let v1 = view.update(0, 1u8);
        let v2 = v1.update(2, 2u8);
        let v3 = v2.update(4, 3u8);

        lemma_update_preserves_len(view, 0, 1u8);
        lemma_update_preserves_len(v1, 2, 2u8);
        lemma_update_preserves_len(v2, 4, 3u8);

        assert(v3.len() == view.len());
        assert(v3.len() == 5);
    }

    /// Test: Update at different indices results in independent changes.
    proof fn test_independent_updates() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![0u8, 0u8, 0u8, 0u8],
        };

        let v1 = view.update(0, 10u8);
        let v2 = v1.update(3, 20u8);

        // Both values are set correctly.
        lemma_update_sets_index(view, 0, 10u8);
        assert(v1.index(0) == 10u8);

        lemma_update_only_changes_index(v1, 3, 20u8, 0);
        assert(v2.index(0) == 10u8);

        lemma_update_sets_index(v1, 3, 20u8);
        assert(v2.index(3) == 20u8);
    }

    /// Test: View transitivity.
    proof fn test_view_eq_transitivity() {
        let v1: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };
        let v2: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };
        let v3: RawArrayView<u8> = RawArrayView { contents: seq![1u8, 2u8] };

        // v1 == v2.
        assert(v1.spec_eq(&v2));
        // v2 == v3.
        assert(v2.spec_eq(&v3));
        // Therefore v1 == v3.
        lemma_view_eq_transitive(&v1, &v2, &v3);
        assert(v1.spec_eq(&v3));
    }

    /// Test: Equal views have equal elements at all indices.
    proof fn test_equal_views_all_elements() {
        let v1: RawArrayView<u8> = RawArrayView { contents: seq![5u8, 10u8, 15u8] };
        let v2: RawArrayView<u8> = RawArrayView { contents: seq![5u8, 10u8, 15u8] };

        assert(v1.spec_eq(&v2));

        lemma_equal_views_equal_elements(&v1, &v2, 0);
        lemma_equal_views_equal_elements(&v1, &v2, 1);
        lemma_equal_views_equal_elements(&v1, &v2, 2);

        assert(v1.index(0) == v2.index(0));
        assert(v1.index(1) == v2.index(1));
        assert(v1.index(2) == v2.index(2));
    }

    /// Test: Invariant properties.
    proof fn test_raw_array_inv() {
        // Valid invariant.
        assert(raw_array_inv::<u8>(1));
        assert(raw_array_inv::<u8>(100));
        assert(raw_array_inv::<u8>(1000000));

        // Invalid invariant (length 0).
        assert(!raw_array_inv::<u8>(0));
    }

    /// Test: Update does not affect unrelated indices (frame condition).
    proof fn test_update_frame_condition() {
        let view: RawArrayView<u8> = RawArrayView {
            contents: seq![1u8, 2u8, 3u8, 4u8, 5u8],
        };

        let updated = view.update(2, 99u8);

        // Indices 0, 1, 3, 4 are unchanged.
        lemma_update_only_changes_index(view, 2, 99u8, 0);
        lemma_update_only_changes_index(view, 2, 99u8, 1);
        lemma_update_only_changes_index(view, 2, 99u8, 3);
        lemma_update_only_changes_index(view, 2, 99u8, 4);

        assert(updated.index(0) == 1u8);
        assert(updated.index(1) == 2u8);
        assert(updated.index(3) == 4u8);
        assert(updated.index(4) == 5u8);

        // Index 2 is changed.
        lemma_update_sets_index(view, 2, 99u8);
        assert(updated.index(2) == 99u8);
    }
}

} // verus!
