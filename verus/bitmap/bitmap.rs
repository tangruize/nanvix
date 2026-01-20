// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.
#![allow(dead_code)]

//==================================================================================================
// Error Handling
//==================================================================================================

use vstd::prelude::*;

verus! {

///
/// # Description
///
/// Error code for various adverse conditions.
///
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum ErrorCode {
    /// Invalid argument.
    InvalidArgument = 22,
    /// Out of memory.
    OutOfMemory = 12,
    /// Device or resource busy.
    ResourceBusy = 16,
    /// Bad address.
    BadAddress = 14,
}

impl ErrorCode {
    ///
    /// # Description
    ///
    /// Returns the error code as an `i32`.
    ///
    pub fn get(&self) -> i32 {
        *self as i32
    }
}

///
/// # Description
///
/// An error type that combines an error code with a reason string.
///
#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: &'static str,
}

impl Error {
    ///
    /// # Description
    ///
    /// Creates a new error.
    ///
    /// # Parameters
    ///
    /// - `code`: The error code.
    /// - `reason`: A static string describing the error reason.
    ///
    /// # Returns
    ///
    /// A new error instance.
    ///
    pub fn new(code: ErrorCode, reason: &'static str) -> Self {
        Self { code, reason }
    }
}

} // verus!

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.reason)
    }
}

//==================================================================================================
// Raw Array Storage
//==================================================================================================

///
/// # Description
///
/// A type that represents the backing storage of a [`RawArray`].
///
// TODO: these three use may be wrong from my code changes --- shan
use std::slice;
use std::{
    alloc::*,
    ptr,
};

#[derive(Debug)]
enum RawArrayStorage<T> {
    /// A storage area that is managed by [alloc::GlobalAlloc].
    Managed { ptr: ptr::NonNull<T>, len: usize },
    /// A storage area that is not managed by [alloc::GlobalAlloc].
    Unmanaged { ptr: ptr::NonNull<T>, len: usize },
}

impl<T> RawArrayStorage<T> {
    ///
    /// # Description
    ///
    /// Constructs backing storage for a raw array.
    ///
    /// # Parameters
    ///
    /// - `len`: Length of the backing storage.
    ///
    /// # Returns
    ///
    /// On success, the backing storage is returned, with all bits set to zero.
    /// On failure, an error is returned instead.
    ///
    fn new_managed(len: usize) -> Result<RawArrayStorage<T>, Error> {
        // Check if the length is invalid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid length"));
        }

        // Allocate underlying memory.
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

        // Initialize the backing storage.
        // Safety: The memory region is valid and the length is valid.
        unsafe { ptr::write_bytes(ptr.as_ptr(), 0, len) };

        Ok(RawArrayStorage::Managed { ptr, len })
    }

    ///
    /// # Description
    ///
    /// Constructs an unmanaged backing storage for a raw array.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the backing storage.
    /// - `len`: Length of the backing storage.
    ///
    /// # Returns
    ///
    /// On success, the backing storage is returned, with all bits set to zero.
    /// On failure, an error is returned instead.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// - `ptr` must be valid for both reads and writes for `len * mem::size_of::<T>()` many bytes.
    /// - `ptr` must be properly aligned.
    /// - `ptr` must point to len consecutive properly initialized values of type `T``.
    ///
    unsafe fn new_unmanaged(ptr: *mut T, len: usize) -> Result<RawArrayStorage<T>, Error> {
        // Check if the length is invalid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid length"));
        }

        // Check if memory region wraps around.
        if ptr.wrapping_add(len) < ptr {
            return Err(Error::new(ErrorCode::InvalidArgument, "wrapping memory region"));
        }

        // Check and cast provided slice.
        let ptr: ptr::NonNull<T> = match ptr::NonNull::new(ptr) {
            Some(ptr) => ptr,
            None => return Err(Error::new(ErrorCode::InvalidArgument, "invalid pointer")),
        };

        // Initialize the backing storage.
        ptr::write_bytes(ptr.as_ptr(), 0, len);

        Ok(RawArrayStorage::Unmanaged { ptr, len })
    }

    ///
    /// # Description
    ///
    /// Gets a mutable slice to the underlying data in the backing storage.
    ///
    /// # Returns
    ///
    /// A mutable slice to the underlying data in the backing storage.
    ///
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

    ///
    /// # Description
    ///
    /// Gets a slice to the underlying data in the backing storage.
    ///
    /// # Returns
    ///
    /// A slice to the underlying data in the backing storage.
    ///
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
}

//==================================================================================================
// Raw Array
//==================================================================================================

verus! {

#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
#[verifier::external_body]
struct ExRawArrayStorage<T>(crate::RawArrayStorage<T>);

///
/// # Description
///
/// A type that represent a fixed-size array.
///

#[derive(Debug)]
#[verifier::reject_recursive_types(T)]
pub struct RawArray<T> {
    /// The backing storage of the raw array.
    storage: RawArrayStorage<T>,
}


impl<T> View for RawArray<T> {
    type V = Seq<T>;

    uninterp spec fn view(&self) -> Seq<T>;
}

// These two functions are used to specify that new create all-0 memory regions
pub uninterp spec fn is_zero<T>(i: T) -> bool;

pub axiom fn axiom_u8_zero_is_0(t: u8) requires is_zero(t) ensures t == 0;


impl<T> RawArray<T> {
    ///
    /// # Description
    ///
    /// Constructs a new managed array.
    ///
    /// # Parameters
    /// - `len`: Length of the array.
    /// # Returns
    ///
    /// On success, the new managed array is returned, with all bits set to zero.
    /// On failure, an error is returned instead.
    ///
    #[verifier::external_body]
    pub fn new(len: usize) -> (result: Result<RawArray<T>, Error>)
        ensures
            result is Ok ==>
            {
                &&& result->Ok_0@.len() == len
                &&& forall|i: int| 0 <= i < result->Ok_0@.len() ==> #[trigger] is_zero(result->Ok_0@[i])
            }
    {
        Ok(RawArray {
            storage: RawArrayStorage::new_managed(len)?,
        })
    }

    ///
    /// # Description
    ///
    /// Constructs a new unmanaged array.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the backing storage.
    /// - `len`: Length of the backing storage.
    ///
    /// # Returns
    ///
    /// On success, the new unmanaged array is returned, with all bits set to zero.
    /// On failure, an error is returned instead.
    ///
    /// # Safety
    ///
    /// Behavior is undefined if any of the following conditions are violated:
    ///
    /// - `ptr` must be valid for both reads and writes for `len * mem::size_of::<T>()` many bytes.
    /// - `ptr` must be properly aligned.
    /// - `ptr` must point to len consecutive properly initialized values of type `T``.
    ///
    #[verifier::external_body]
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> (result: Result<RawArray<T>, Error>)
        ensures
            result is Ok ==> result->Ok_0@.len() == len,
    {
        Ok(RawArray {
            storage: RawArrayStorage::new_unmanaged(ptr, len)?,
        })
    }

    ///
    /// # Description
    ///
    /// Sets a value at a given index.
    ///
    /// # Parameters
    ///
    /// - `index`: Index to set the value at.
    /// - `value`: Value to set.
    ///
    #[verifier::external_body]
    pub fn set(&mut self, index: usize, value: T)
        requires
            0 <= index < old(self)@.len(),
        ensures
            self@.len() == old(self)@.len(),
            self@[index as int] == value,
            forall|i: int| 0 <= i < self@.len() && i != index ==> self@[i] == old(self)@[i],
    {
        let slice: &mut [T] = self.storage.get_mut();
        slice[index] = value;
    }

    ///
    /// # Description
    ///
    /// Returns the length of the array.
    ///
    /// # Returns
    ///
    /// The length of the array.
    ///
    #[verifier::external_body]
    pub fn len(&self) -> (result: usize)
        ensures
            result == self@.len(),
    {
        self.storage.get().len()
    }
}

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

impl<T> core::ops::DerefMut for RawArray<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.storage.get_mut()
    }
}

impl<T> Drop for RawArray<T> {
    fn drop(&mut self) {
        use std::alloc::{
            dealloc,
            Layout,
        };

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
// Bitmap
//==================================================================================================

verus! {

///
/// # Description
///
/// A bitmap.
///
#[derive(Debug)]
#[verifier::ext_equal]
pub struct Bitmap {
    /// Capacity of the bitmap (in bits).
    number_of_bits: usize,
    /// Number of bits set in the bitmap.
    usage: usize,
    /// Underlying bits.
    bits: RawArray<u8>,
}

/// A view of the Bitmap as a sequence of booleans
#[verifier::ext_equal]
pub struct BitmapView {
    pub bits: Seq<bool>,
}

impl BitmapView {
    /// Returns the number of bits in the bitmap view
    pub open spec fn number_of_bits(&self) -> int {
        self.bits.len() as int
    }

    /// Returns the usage (count of set bits) in the bitmap view
    pub open spec fn usage(&self) -> int {
        Bitmap::count_set_bits_in_seq(self.bits, 0, self.bits.len() as int)
    }

    /// Returns true if the bitmap is full (all bits set)
    pub open spec fn is_full(&self) -> bool {
        self.usage() == self.number_of_bits()
    }

    /// Returns true if the bitmap is empty (no bits set)
    pub open spec fn is_empty(&self) -> bool {
        self.usage() == 0
    }
}

// Define a view for Bitmap that represents it as a sequence of booleans
impl View for Bitmap {
    type V = BitmapView;

    closed spec fn view(&self) -> BitmapView {
        BitmapView {
            bits: Self::bits_to_seq(self.bits@, self.number_of_bits as int),
        }
    }
}

impl Bitmap {
    //==================================================================================================
    // Specification Functions
    //==================================================================================================

    /// Helper spec function: convert RawArray<u8> to Seq<bool>
    pub open spec fn bits_to_seq(bytes: Seq<u8>, num_bits: int) -> Seq<bool>
        decreases num_bits
    {
        Seq::new(num_bits as nat, |i: int| Self::bit_at(bytes, i))
    }

    /// Helper spec function: get the bit value at a specific index
    pub open spec fn bit_at(bytes: Seq<u8>, bit_index: int) -> bool {
        let word: int = bit_index / (u8::BITS as int);
        let bit: int = bit_index % (u8::BITS as int);
        if 0 <= bit_index && word < bytes.len() {
            (bytes[word] & (1u8 << bit)) != 0
        } else {
            false
        }
    }

    /// Helper spec function: check if a bit at the given bit index is set
    pub closed spec fn is_bit_set(&self, bit_index: int) -> bool {
        &&& 0 <= bit_index < self@.number_of_bits()
        &&& self@.bits[bit_index]
    }

    /// Helper spec function: check if all bits in range [start, end) are set
    pub open spec fn all_bits_set_in_range(&self, start: int, end: int) -> bool {
        forall|i: int| start <= i < end ==> self.is_bit_set(i)
    }

    /// Helper spec function: check if all bits in range [start, end) are not set
    pub open spec fn all_bits_unset_in_range(&self, start: int, end: int) -> bool {
        forall|i: int| start <= i < end ==> !self.is_bit_set(i)
    }

    /// Helper spec function: check if there exists a contiguous range of n free bits starting at start
    pub open spec fn has_free_range_at(&self, start: int, n: int) -> bool {
        &&& 0 <= start
        &&& start + n <= self@.number_of_bits()
        &&& self.all_bits_unset_in_range(start, start + n)
    }

    /// Helper spec function: check if there exists a contiguous range of n free bits
    pub open spec fn exists_contiguous_free_range(&self, n: int) -> bool {
        exists|start: int| #![trigger self.has_free_range_at(start, n)]
            self.has_free_range_at(start, n)
    }

    /// Helper spec function: count set bits in a range [start, end) of a sequence
    pub closed spec fn count_set_bits_in_seq(bits: Seq<bool>, start: int, end: int) -> int
        decreases end - start when end >= start
    {
        if start >= end {
            0
        } else {
            let rest = Self::count_set_bits_in_seq(bits, start + 1, end);
            if 0 <= start < bits.len() && bits[start] { rest + 1 } else { rest }
        }
    }

    //==================================================================================================
    // Lemmas: Basic Properties
    //==================================================================================================

    /// Lemma: count in a sequence range is bounded by the range size
    proof fn lemma_count_set_bits_in_seq_bounded(bits: Seq<bool>, start: int, end: int)
        requires
            start <= end,
        ensures
            Self::count_set_bits_in_seq(bits, start, end) >= 0,
            Self::count_set_bits_in_seq(bits, start, end) <= end - start,
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
        }
    }

    //==================================================================================================
    // Lemmas: Bit Set/Unset Properties
    //==================================================================================================

    /// Lemma: if a bit in sequence is set, count in range >= 1
    proof fn lemma_bit_set_in_seq_implies_count_geq_1(bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < bits.len(),
            bits[index],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) >= 1,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
        } else {
            Self::lemma_bit_set_in_seq_implies_count_geq_1(bits, start + 1, end, index);
        }
    }

    /// Lemma: if a bit in sequence is not set, count < range size
    proof fn lemma_bit_unset_in_seq_implies_count_lt_size(bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < bits.len(),
            !bits[index],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) < end - start,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
        } else {
            Self::lemma_bit_unset_in_seq_implies_count_lt_size(bits, start + 1, end, index);
        }
    }

    //==================================================================================================
    // Lemmas: Bit Mutation Effects
    //==================================================================================================

    /// Lemma: setting a bit increases the count by 1
    proof fn lemma_set_bit_increases_count(&self, new_self: &Self, index: int)
        requires
            0 <= index < self@.number_of_bits(),
            !self.is_bit_set(index),
            new_self.is_bit_set(index),
            forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                self.is_bit_set(i) == new_self.is_bit_set(i),
            self@.number_of_bits() == new_self@.number_of_bits(),
            self@.bits.len() == new_self@.bits.len(),
        ensures
            new_self@.usage() == self@.usage() + 1,
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() && i != index
        implies self@.bits[i] == new_self@.bits[i]
        by {
            assert(self.is_bit_set(i) == new_self.is_bit_set(i));
        };

        Self::lemma_set_bit_increases_count_in_seq(self@.bits, new_self@.bits, 0, self@.number_of_bits(), index);
    }

    /// Lemma: setting a bit in a sequence increases the count by 1
    proof fn lemma_set_bit_increases_count_in_seq(old_bits: Seq<bool>, new_bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < old_bits.len(),
            0 <= index < new_bits.len(),
            old_bits.len() == new_bits.len(),
            !old_bits[index],
            new_bits[index],
            forall|i: int| start <= i < end && i != index && 0 <= i < old_bits.len() ==>
                old_bits[i] == new_bits[i],
        ensures
            Self::count_set_bits_in_seq(new_bits, start, end) == Self::count_set_bits_in_seq(old_bits, start, end) + 1,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_bits_equal_in_seq_implies_count_equal(old_bits, new_bits, start + 1, end);
        } else {
            Self::lemma_set_bit_increases_count_in_seq(old_bits, new_bits, start + 1, end, index);
        }
    }

    //==================================================================================================
    // Lemmas: Bit-level Operations
    //==================================================================================================

    /// Lemma: Helper for proving bit operations on bytes
    proof fn lemma_bit_or_effects(old_byte: u8, bit_pos: int, new_byte: u8)
        requires
            0 <= bit_pos < 8,
            new_byte == (old_byte | (1u8 << bit_pos)),
        ensures
            (new_byte & (1u8 << bit_pos)) != 0,
            forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos ==>
                (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos)),
    {
        let shift: u8 = bit_pos as u8;
        assert((new_byte & (1u8 << shift)) != 0) by (bit_vector)
            requires
                new_byte == (old_byte | (1u8 << shift)),
                0 <= shift < 8,
        ;
        assert forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos implies
            (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos))
        by {
            let other_shift: u8 = other_pos as u8;
            assert((new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift))) by (bit_vector)
                requires
                    new_byte == (old_byte | (1u8 << shift)),
                    0 <= shift < 8,
                    0 <= other_shift < 8,
                    shift != other_shift,
            ;
        }
    }

    /// Lemma: Helper for proving bit clear operations on bytes
    proof fn lemma_bit_and_not_effects(old_byte: u8, bit_pos: int, new_byte: u8)
        requires
            0 <= bit_pos < 8,
            new_byte == (old_byte & !(1u8 << bit_pos)),
        ensures
            (new_byte & (1u8 << bit_pos)) == 0,
            forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos ==>
                (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos)),
    {
        let shift: u8 = bit_pos as u8;
        assert((new_byte & (1u8 << shift)) == 0) by (bit_vector)
            requires
                new_byte == (old_byte & !(1u8 << shift)),
                0 <= shift < 8,
        ;
        assert forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos implies
            (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos))
        by {
            let other_shift: u8 = other_pos as u8;
            assert((new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift))) by (bit_vector)
                requires
                    new_byte == (old_byte & !(1u8 << shift)),
                    0 <= shift < 8,
                    0 <= other_shift < 8,
                    shift != other_shift,
            ;
        }
    }

    /// Lemma: clearing a bit decreases the count by 1
    proof fn lemma_clear_bit_decreases_count(&self, new_self: &Self, index: int)
        requires
            0 <= index < self@.number_of_bits(),
            self.is_bit_set(index),
            !new_self.is_bit_set(index),
            forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                self.is_bit_set(i) == new_self.is_bit_set(i),
            self@.number_of_bits() == new_self@.number_of_bits(),
            self@.bits.len() == new_self@.bits.len(),
        ensures
            new_self@.usage() == self@.usage() - 1,
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() && i != index
        implies self@.bits[i] == new_self@.bits[i]
        by {
            assert(self.is_bit_set(i) == new_self.is_bit_set(i));
        };

        Self::lemma_clear_bit_decreases_count_in_seq(self@.bits, new_self@.bits, 0, self@.number_of_bits(), index);
    }

    /// Lemma: clearing a bit in a sequence decreases the count by 1
    proof fn lemma_clear_bit_decreases_count_in_seq(old_bits: Seq<bool>, new_bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < old_bits.len(),
            0 <= index < new_bits.len(),
            old_bits.len() == new_bits.len(),
            old_bits[index],
            !new_bits[index],
            forall|i: int| start <= i < end && i != index && 0 <= i < old_bits.len() ==>
                old_bits[i] == new_bits[i],
        ensures
            Self::count_set_bits_in_seq(new_bits, start, end) == Self::count_set_bits_in_seq(old_bits, start, end) - 1,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_bits_equal_in_seq_implies_count_equal(old_bits, new_bits, start + 1, end);
        } else {
            Self::lemma_clear_bit_decreases_count_in_seq(old_bits, new_bits, start + 1, end, index);
        }
    }

    /// Lemma: if bits in sequences are equal in a range, counts are equal
    proof fn lemma_bits_equal_in_seq_implies_count_equal(bits1: Seq<bool>, bits2: Seq<bool>, start: int, end: int)
        requires
            start <= end,
            bits1.len() == bits2.len(),
            forall|i: int| start <= i < end && 0 <= i < bits1.len() ==>
                bits1[i] == bits2[i],
        ensures
            Self::count_set_bits_in_seq(bits1, start, end) == Self::count_set_bits_in_seq(bits2, start, end),
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_bits_equal_in_seq_implies_count_equal(bits1, bits2, start + 1, end);
        }
    }

    /// Lemma: if all bits in sequence are false, count == 0
    proof fn lemma_all_zero_in_seq_implies_count_zero(bits: Seq<bool>, start: int, end: int)
        requires
            0 <= start <= end,
            forall|i: int| start <= i < end && 0 <= i < bits.len() ==> !bits[i],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) == 0,
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_all_zero_in_seq_implies_count_zero(bits, start + 1, end);
        }
    }

    //==================================================================================================
    // Lemmas: View Synchronization
    //==================================================================================================

    /// Lemma: usage count equals the number of set bits
    pub proof fn lemma_usage_equals_count_set_bits(&self)
        requires
            self.inv(),
        ensures
            self@.usage() == Self::count_set_bits_in_seq(self@.bits, 0, self@.number_of_bits()),
    {
        // This follows directly from the definition of usage in BitmapView
    }

    /// Lemma: if bitmap is empty, no bits are set
    pub proof fn lemma_is_empty_means_no_bits_set(&self)
        requires
            self.inv(),
            self@.is_empty(),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==> !self.is_bit_set(i),
    {
        if exists|i: int| 0 <= i < self@.number_of_bits() && self.is_bit_set(i) {
            let i = choose|i: int| 0 <= i < self@.number_of_bits() && self.is_bit_set(i);
            Self::lemma_bit_set_in_seq_implies_count_geq_1(self@.bits, 0, self@.number_of_bits(), i);
        }
    }

    /// Lemma: if bitmap is full, all bits are set
    pub proof fn lemma_is_full_means_all_bits_set(&self)
        requires
            self.inv(),
            self@.is_full(),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i),
    {
        if exists|i: int| 0 <= i < self@.number_of_bits() && !self.is_bit_set(i) {
            let i = choose|i: int| 0 <= i < self@.number_of_bits() && !self.is_bit_set(i);
            Self::lemma_bit_unset_in_seq_implies_count_lt_size(self@.bits, 0, self@.number_of_bits(), i);
        }
    }

    /// Lemma: if bitmap is not full, there exists at least one unset bit
    pub proof fn lemma_not_full_means_exists_unset_bit(&self)
        requires
            self.inv(),
            !self@.is_full(),
        ensures
            exists|i: int| 0 <= i < self@.number_of_bits() && !self.is_bit_set(i),
    {
        if forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i) {
            Self::lemma_all_bits_set_means_full(self);
        }
    }

    /// Lemma: if all bits are set, bitmap is full
    proof fn lemma_all_bits_set_means_full(&self)
        requires
            self.inv(),
            forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i),
        ensures
            self@.is_full(),
        decreases self@.number_of_bits()
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() implies self@.bits[i]
        by {
            assert(self.is_bit_set(i));
        };
        Self::lemma_all_set_means_count_equals_size(self@.bits, 0, self@.number_of_bits());
    }

    /// Lemma: if all bits in range are set, count equals range size
    proof fn lemma_all_set_means_count_equals_size(bits: Seq<bool>, start: int, end: int)
        requires
            0 <= start <= end,
            end <= bits.len(),
            forall|i: int| start <= i < end ==> bits[i],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) == end - start,
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_all_set_means_count_equals_size(bits, start + 1, end);
        }
    }

    /// Lemma: setting a byte bit reflects in the boolean sequence
    proof fn lemma_byte_or_reflects_in_view(&self, new_self: &Self, word: int, bit: int)
        requires
            0 <= word < self.bits@.len(),
            0 <= bit < (u8::BITS as int),
            new_self.bits@.len() == self.bits@.len(),
            new_self.bits@[word] == (self.bits@[word] | (1u8 << bit)),
            forall|i: int| 0 <= i < self.bits@.len() && i != word ==> self.bits@[i] == new_self.bits@[i],
            self.number_of_bits == new_self.number_of_bits,
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==>
                self@.bits[i] == new_self@.bits[i] || i == word * (u8::BITS as int) + bit,
            new_self@.bits[word * (u8::BITS as int) + bit],
    {
        Self::lemma_bit_or_effects(self.bits@[word], bit, new_self.bits@[word]);
    }

    /// Lemma: clearing a byte bit reflects in the boolean sequence
    proof fn lemma_byte_and_not_reflects_in_view(&self, new_self: &Self, word: int, bit: int)
        requires
            0 <= word < self.bits@.len(),
            0 <= bit < (u8::BITS as int),
            new_self.bits@.len() == self.bits@.len(),
            new_self.bits@[word] == (self.bits@[word] & !(1u8 << bit)),
            forall|i: int| 0 <= i < self.bits@.len() && i != word ==> self.bits@[i] == new_self.bits@[i],
            self.number_of_bits == new_self.number_of_bits,
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==>
                self@.bits[i] == new_self@.bits[i] || i == word * (u8::BITS as int) + bit,
            !new_self@.bits[word * (u8::BITS as int) + bit],
    {
        Self::lemma_bit_and_not_effects(self.bits@[word], bit, new_self.bits@[word]);
    }

    /// Lemma: when all raw bytes are zero, all boolean bits are false
    proof fn lemma_zero_bytes_means_false_bits(&self)
        requires
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
            forall|i: int| 0 <= i < self.bits@.len() ==> self.bits@[i] == 0,
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==> !self@.bits[i],
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() implies !self@.bits[i] by {
            let byte_idx = i / (u8::BITS as int);
            let bit_idx = i % (u8::BITS as int);

            let bit_idx_u8 = bit_idx as u8;
            assert((0u8 & (1u8 << bit_idx_u8)) == 0) by (bit_vector)
                requires 0 <= bit_idx_u8 < 8;
        };
    }

    //==================================================================================================
    // Invariant
    //==================================================================================================

    /// Invariant: the bitmap's number_of_bits must equal bits.len() * 8
    /// and must be less than u32::MAX
    pub closed spec fn inv(&self) -> bool {
        &&& self@.number_of_bits() > 0
        &&& self@.number_of_bits() == self.bits@.len() * (u8::BITS as int)
        &&& self@.number_of_bits() < u32::MAX as int
        &&& self@.usage() <= self@.number_of_bits()
        &&& self.number_of_bits as int == self@.number_of_bits()
        &&& self.usage as int == self@.usage()
    }

    //==================================================================================================
    // Public Methods
    //==================================================================================================

    ///
    /// # Description
    ///
    /// Creates a new bitmap with a given length. The bitmap is initialized with all bits set to zero.
    ///
    /// # Parameters
    ///
    /// - `number_of_bits`: Length of the bitmap in bits.
    ///
    /// # Returns
    ///
    /// Upon success, a new bitmap is returned. Upon failure, an error is returned instead.
    ///
    pub fn new(number_of_bits: usize) -> (result: Result<Self, Error>)
        ensures
            result is Ok ==> {
                let bitmap = result->Ok_0;
                &&& bitmap.inv()
                &&& bitmap@.number_of_bits() == number_of_bits as int
                &&& bitmap@.is_empty()
                &&& forall|i: int| 0 <= i < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(i)
            },
            //TODO: should we specify Err situations?
    {
        // Check if the length is invalid.
        if number_of_bits == 0 || number_of_bits >= u32::MAX as usize {
            let reason: &str = "invalid length";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if the length is not a multiple of the number of the bitmap word.
        if number_of_bits % (u8::BITS as usize) != 0 {
            let reason: &str = "length must be a multiple of 8";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Allocate the bitmap.
        let array: RawArray<u8> = RawArray::new(number_of_bits / u8::BITS as usize)?;

        // NOTE: the bitmap is already zeroed out by RawArray::new()

        let result = Self {
            number_of_bits,
            bits: array,
            usage: 0,
        };

        proof {
            assert forall|i: int| 0 <= i < result.bits@.len() implies (result.bits@[i] == 0) by {
                axiom_u8_zero_is_0(result.bits@[i]);
            };
            result.lemma_zero_bytes_means_false_bits();
            Self::lemma_all_zero_in_seq_implies_count_zero(result@.bits, 0, result@.number_of_bits());
        }

        Ok(result)
    }

    ///
    /// # Description
    ///
    /// Creates a new bitmap from a raw array. The bitmap is initialized with
    /// all bits set to zero.
    ///
    /// # Parameters
    ///
    /// - `array`: Raw array to create the bitmap from.
    ///
    /// # Returns
    ///
    /// Upon success, a new bitmap is returned. Upon failure, an error is returned instead.
    ///
    pub fn from_raw_array(array: RawArray<u8>) -> (result: Self)
        requires
            array@.len() > 0, //TODO: is this needed?
            array@.len() <= usize::MAX / (u8::BITS as usize),
            array@.len() * (u8::BITS as usize) < u32::MAX as usize,
            forall|i: int| 0 <= i < array@.len() ==> array@[i] == 0,
        ensures
            result.inv(),
            result@.number_of_bits() == array@.len() * (u8::BITS as int),
            result@.is_empty(),
            forall|i: int| 0 <= i < result@.number_of_bits() ==> !result.is_bit_set(i),
    {
        // NOTE: no need to test if the length of the raw array is valid, as it is by construction.
        // NOTE: the bitmap is already zeroed out by RawArray::new() or RawArray::from_raw_parts()

        let result = Self {
            number_of_bits: array.len() * u8::BITS as usize,
            bits: array,
            usage: 0,
        };
        proof {
            result.lemma_zero_bytes_means_false_bits();
            Self::lemma_all_zero_in_seq_implies_count_zero(result@.bits, 0, result@.number_of_bits());
        }
        result
    }

    ///
    /// # Description
    ///
    /// Returns the number of bits in the bitmap.
    ///
    /// # Returns
    ///
    /// The number of bits in the bitmap.
    ///
    pub fn number_of_bits(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.number_of_bits(),
            result > 0, //TODO: really?
            result < u32::MAX as usize,
            // number_of_bits is read-only - no state changes //TODO: should this be specified?
    {
        self.number_of_bits
    }

    ///
    /// # Description
    ///
    /// Allocates a bit in the bitmap.
    ///
    /// # Returns
    ///
    /// Upon success, the index of the allocated bit is returned. Upon failure, an error is returned
    /// instead.
    ///
    pub fn alloc(&mut self) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let index = result->Ok_0 as int;
                &&& 0 <= index < self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.is_bit_set(index)
                &&& !old(self).is_bit_set(index)
                &&& !old(self)@.is_full()
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                &&& self@.usage() == old(self)@.usage() + 1
            },
            result is Err ==> self@.bits == old(self)@.bits,
    {
        self.alloc_range(1)
    }

    ///
    /// # Description
    ///
    /// Allocates a range of bits in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `size`: Size of the range to allocate.
    ///
    /// # Returns
    ///
    /// Upon success, the index of the allocated range is returned. Upon failure, an error is returned
    /// instead.
    ///
    #[verifier::exec_allows_no_decreases_clause]
    pub fn alloc_range(&mut self, size: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let start = result->Ok_0 as int;
                &&& 0 <= start < self@.number_of_bits()
                &&& 0 < size <= self@.number_of_bits()
                &&& start + (size as int) <= self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.all_bits_set_in_range(start, start + (size as int))
                &&& old(self).all_bits_unset_in_range(start, start + (size as int))
                &&& forall|i: int| 0 <= i < self@.number_of_bits() &&
                    (i < start || i >= start + (size as int)) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                &&& self@.usage() == old(self)@.usage() + (size as int)
            },
            result is Err ==> self@.bits == old(self)@.bits,
    {
        let ghost old_self = *self;

        // Check if the size is valid.
        if size == 0 || size > self.number_of_bits {
            let reason: &str = "invalid size";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if allocation exceeds the bitmap capacity.
        if self.usage > self.number_of_bits - size {
            let reason: &str = "allocation exceeds bitmap capacity";
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        let mut start: usize = 0;

        // Traverse the bitmap until the last possible starting bit.
        while start <= self.number_of_bits - size
            invariant
                self.inv(),
                old_self.inv(),
                old_self == old(self),
                size > 0,
                size <= self.number_of_bits,
                start <= self.number_of_bits,
                self@.bits == old(self)@.bits,
                self.usage <= self.number_of_bits - size,
        {
            // Check for fast skip/ path.
            let is_aligned: bool = start % (u8::BITS as usize) == 0;
            if is_aligned {
                let word: usize = start / u8::BITS as usize;
                // Fast skip: if the starting word is full, skip to the next word.
                if self.bits[word] == u8::MAX {
                    // Jump to next byte boundary.
                    start = start + u8::BITS as usize;
                    continue;
                }
            }

            // Check if all bits in the range are free.
            let mut free: bool = true;
            let mut offset: usize = 0;
            while offset < size
                invariant_except_break
                    start <= self.number_of_bits - size,
                    forall|i: int| 0 <= i < offset ==>
                        !#[trigger] self.is_bit_set((start + i) as int),
                invariant
                    self.inv(),
                    old_self == old(self),
                    0 < size <= self.number_of_bits,
                    offset <= size,
                    self@.bits == old(self)@.bits,
                ensures
                    start <= self.number_of_bits,
                    free ==> start <= self.number_of_bits - size &&
                        forall|i: int| 0 <= i < size ==>
                            !#[trigger] self.is_bit_set((start + i) as int),
            {
                let idx: usize = start + offset;
                let (w, b): (usize, usize) = self.index_unchecked(idx);
                if (self.bits[w] & (1 << b)) != 0 {
                    free = false;
                    start = start + offset + 1;
                    break;
                }
                offset = offset + 1;
            }
            if free {
                // At this point: forall i in [0, size): !self.is_bit_set((start + i) as int)
                proof {
                    // Establish that these bits are not set in old_self as well
                    assert forall|i: int| 0 <= i < size implies
                        !#[trigger] old_self.is_bit_set((start + i) as int)
                    by {
                        assert(!self.is_bit_set((start + i) as int));
                    };
                }

                // Allocate the range
                let ghost pre_alloc_self = *self;

                let mut offset: usize = 0;
                while offset < size
                    invariant
                        self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
                        old_self.inv(),
                        pre_alloc_self.inv(),
                        old_self == old(self),
                        self.number_of_bits == pre_alloc_self.number_of_bits,
                        self.usage == pre_alloc_self.usage,
                        0 < size <= self.number_of_bits,
                        start <= self.number_of_bits - size,
                        offset <= size,
                        forall|i: int| 0 <= i < size ==>
                            !#[trigger] pre_alloc_self.is_bit_set((start + i) as int),
                        forall|i: int| 0 <= i < size ==>
                            !#[trigger] old_self.is_bit_set((start + i) as int),
                        forall|i: int| 0 <= i < offset ==>
                            #[trigger] self.is_bit_set((start + i) as int),
                        forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset) as int)) ==>
                            #[trigger] self.is_bit_set(i) == #[trigger] pre_alloc_self.is_bit_set(i),
                        forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset) as int)) ==>
                            #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i),
                        self@.usage() == pre_alloc_self@.usage() + offset,
                {
                    let idx: usize = start + offset;
                    let (w, b): (usize, usize) = self.index_unchecked(idx);
                    let ghost loop_old_self = *self;
                    self.bits.set(w, self.bits[w] | (1 << b));

                    proof {
                        assert(loop_old_self.is_bit_set(idx as int) == pre_alloc_self.is_bit_set(idx as int));
                        loop_old_self.lemma_byte_or_reflects_in_view(self, w as int, b as int);
                        loop_old_self.lemma_set_bit_increases_count(self, idx as int);

                        assert forall|i: int| 0 <= i < offset + 1 implies
                            #[trigger] self.is_bit_set((start + i) as int)
                        by {
                            if i < offset {
                                assert(loop_old_self.is_bit_set((start + i) as int));
                            }
                        };

                        assert forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset + 1) as int)) implies
                            #[trigger] self.is_bit_set(i) == #[trigger] pre_alloc_self.is_bit_set(i)
                        by {
                            if i < start as int || i >= (start + offset) as int {
                                assert(loop_old_self.is_bit_set(i) == pre_alloc_self.is_bit_set(i));
                            }
                        };

                        assert forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset + 1) as int)) implies
                            #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i)
                        by {
                            if i < start as int || i >= (start + offset) as int {
                                assert(loop_old_self.is_bit_set(i) == old_self.is_bit_set(i));
                            }
                        };
                    }

                    offset = offset + 1;
                }

                let ghost pre_assignment_bits_set_in_range: Set<int> = Set::new(|i: int| start as int <= i < start as int + (size as int) && self.is_bit_set(i));

                proof {
                    // At loop exit: offset == size, so all bits [start, start+size) are set
                    assert forall|i: int| start as int <= i < (start + size) as int implies
                        self.is_bit_set(i)
                    by {
                        let offset_of_i = (i - start as int) as int;
                        assert(self.is_bit_set((start as int + offset_of_i) as int));
                    };

                    // Record that all bits in the range are set
                    assert forall|i: int| start as int <= i < start as int + (size as int) implies
                        pre_assignment_bits_set_in_range.contains(i)
                    by {
                        assert(self.is_bit_set(i));
                    };

                    // All bits outside [start, start+size) remain unchanged from old(self)
                    assert forall|i: int| (0 <= i < self@.number_of_bits() &&
                        (i < start as int || i >= (start + size) as int)) implies
                        self.is_bit_set(i) == old(self).is_bit_set(i)
                    by {
                        assert(self.is_bit_set(i) == pre_alloc_self.is_bit_set(i));
                    };

                    // Establish all bits [start, start+size) were not set in old(self)
                    assert forall|i: int| start as int <= i < (start + size) as int implies
                        !old(self).is_bit_set(i)
                    by {
                        let offset_of_i = (i - start as int);
                        assert(!old_self.is_bit_set((start + offset_of_i) as int));
                    };
                }

                self.usage = self.usage + size;

                proof {
                    // After assignment to self.usage, the bits array hasn't changed
                    // Since bits haven't changed, all the bit properties remain true
                    assert forall|i: int| start as int <= i < start as int + (size as int) implies
                        self.is_bit_set(i)
                    by {
                        assert(pre_assignment_bits_set_in_range.contains(i));
                    };
                }

                return Ok(start);
            }
        }

        let reason: &str = "bitmap is full";
        Err(Error::new(ErrorCode::OutOfMemory, reason))
    }

    ///
    /// # Description
    ///
    /// Sets a bit at a given index in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit to set.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned. Upon failure, an error is returned instead.
    ///
    pub fn set(&mut self, index: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& self.is_bit_set(index as int)
                &&& !old(self).is_bit_set(index as int)
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != (index as int) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                &&& self@.usage() == old(self)@.usage() + 1
            },
            result is Err ==> self == old(self),
            ((index as int) < old(self)@.number_of_bits() && !old(self).is_bit_set(index as int))
                ==> result is Ok,
    {
        // Check if the bit is already set.
        if self.test(index)? {
            let reason: &str = "bit is already set";
            return Err(Error::new(ErrorCode::ResourceBusy, reason));
        }

        proof {
            Self::lemma_bit_unset_in_seq_implies_count_lt_size(self@.bits, 0, self@.number_of_bits(), index as int);
        }

        let (word, bit): (usize, usize) = self.index(index)?;
        let ghost old_self = *self;

        self.bits.set(word, self.bits[word] | (1 << bit));

        proof {
            old_self.lemma_byte_or_reflects_in_view(self, word as int, bit as int);
            old_self.lemma_set_bit_increases_count(self, index as int);
        }

        self.usage = self.usage + 1;

        Ok(())
    }

    ///
    /// # Description
    ///
    /// Clears a bit at a given index in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit to clear.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned. Upon failure, an error is returned instead.
    ///
    pub fn clear(&mut self, index: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& !self.is_bit_set(index as int)
                &&& old(self).is_bit_set(index as int)
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != (index as int) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                &&& self@.usage() == old(self)@.usage() - 1
            },
            result is Err ==> self == old(self),
            ((index as int) < old(self)@.number_of_bits() && old(self).is_bit_set(index as int))
                ==> result is Ok,
    {
        // Check if the bit is already cleared.
        if !self.test(index)? {
            let reason: &str = "bit is already cleared";
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }

        proof {
            Self::lemma_bit_set_in_seq_implies_count_geq_1(self@.bits, 0, self@.number_of_bits(), index as int);
        }

        let (word, bit): (usize, usize) = self.index(index)?;
        let ghost old_self = *self;

        self.bits.set(word, self.bits[word] & !(1 << bit));

        proof {
            old_self.lemma_byte_and_not_reflects_in_view(self, word as int, bit as int);
            old_self.lemma_clear_bit_decreases_count(self, index as int);
        }

        self.usage = self.usage - 1;

        Ok(())
    }

    ///
    /// # Description
    ///
    /// Tests a bit at a given index in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit to test.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(true)` is returned if the bit is set, `Ok(false)` is returned otherwise.
    /// Upon failure, an error is returned instead.
    ///
    pub fn test(&self, index: usize) -> (result: Result<bool, Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& result->Ok_0 == self.is_bit_set(index as int)
            },
            (index as int) < self@.number_of_bits() ==> result is Ok,
            // test is read-only - no state changes
    {
        let (word, bit): (usize, usize) = self.index(index)?;
        let byte_val = self.bits[word];
        let result_val = (byte_val & (1 << bit)) != 0;

        Ok(result_val)
    }

    //==================================================================================================
    // Private Helper Methods
    //==================================================================================================

    ///
    /// # Description
    ///
    /// Returns the `(word, bit)` pair of a index.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit.
    ///
    /// # Returns
    ///
    /// Upon success, the `(word, bit)` pair of the index is returned. Upon
    /// failure, an error is returned instead.
    ///
    fn index(&self, index: usize) -> (result: Result<(usize, usize), Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                let (word, bit) = result->Ok_0;
                &&& word < self.bits@.len()
                &&& bit < u8::BITS as usize
                &&& word == index / (u8::BITS as usize)
                &&& bit == index % (u8::BITS as usize)
                &&& index < self.bits@.len() * (u8::BITS as usize)
            },
            (index as int) < self@.number_of_bits() ==> result is Ok,
    {
        // Check if the index is out of bounds.
        if index >= self.bits.len() * u8::BITS as usize {
            let reason: &str = "index out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        Ok(self.index_unchecked(index))
    }

    ///
    /// # Description
    ///
    /// Returns the `(word, bit)` pair of a index without checking bounds.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit.
    ///
    /// # Returns
    ///
    /// The `(word, bit)` pair of the index.
    ///
    fn index_unchecked(&self, index: usize) -> (result: (usize, usize))
        requires
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
            index < self.number_of_bits,
        ensures
            result.0 == index / (u8::BITS as usize),
            result.1 == index % (u8::BITS as usize),
            result.0 < self.bits@.len(),
            result.1 < u8::BITS as usize,
    {
        let word: usize = index / u8::BITS as usize;
        let bit: usize = index % u8::BITS as usize;
        (word, bit)
    }
}

//==================================================================================================
// Verifiable Test Functions
//==================================================================================================


//================================================================================
//=====These few functions are converted from test functions in mod tests=========
//=====Those functions were originally written by LLM 		       =========
//=====before the specifications/proof was developed in this file        =========
//================================================================================
//1. **test_bitmap_new** → **test_bitmap_new_verified**
//2. **test_bitmap_alloc** → **test_bitmap_alloc_verified**
//3. **test_bitmap_set_clear** → **test_bitmap_set_clear_verified**
//4. **test_bitmap_alloc_range** → **test_bitmap_alloc_range_verified**


/// Verifiable test: creating a new bitmap should set number_of_bits correctly
fn test_bitmap_new_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(bitmap) = result {
        assert(bitmap@.number_of_bits() == number_of_bits as int);
    }
}

/// Verifiable test: allocating a bit should return a valid index and set the bit
fn test_bitmap_alloc_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc();
        if let Ok(index) = alloc_result {
            // The allocated index should be within bounds
            assert(index < number_of_bits);
            // The bit at the allocated index should be set
            assert(bitmap.is_bit_set(index as int));
        }
    }
}

/// Verifiable test: setting and clearing a bit should work correctly
fn test_bitmap_set_clear_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set the bit
        let set_result = bitmap.set(index);
        if let Ok(()) = set_result {
            // The bit should be set
            assert(bitmap.is_bit_set(index as int));

            // Clear the bit
            let clear_result = bitmap.clear(index);
            if let Ok(()) = clear_result {
                // The bit should be cleared
                assert(!bitmap.is_bit_set(index as int));
            }
        }
    }
}

/// Verifiable test: allocating a range should allocate contiguous bits
fn test_bitmap_alloc_range_verified(number_of_bits: usize, size: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size <= number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc_range(size);
        if let Ok(start_index) = alloc_result {
            // The start index should be within valid range
            assert(start_index + size <= number_of_bits);

            // All bits in the range should be set
            assert(bitmap.all_bits_set_in_range(start_index as int, (start_index + size) as int));
        }
    }
}


//==========================================================
//=====These few functions were written by LLM 	 =========
//=====after it converted the above test functions =========
//==========================================================
// test_bitmap_multiple_alloc_verified(number_of_bits: usize)
// test_bitmap_clear_and_realloc_verified(number_of_bits: usize, index: usize)
// test_bitmap_usage_tracking_verified(number_of_bits: usize)
// test_bitmap_alloc_range_preserves_others_verified(number_of_bits: usize, size: usize, test_index: usize)
// test_bitmap_number_of_bits_constant_verified(number_of_bits: usize, index: usize)
// test_bitmap_double_set_fails_verified(number_of_bits: usize, index: usize)
// test_bitmap_double_clear_fails_verified(number_of_bits: usize, index: usize)


/// Verifiable test: multiple allocations should not overlap
fn test_bitmap_multiple_alloc_verified(number_of_bits: usize)
    requires
        number_of_bits >= 16,  // Need at least 2 bits
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc1 = bitmap.alloc();
        if let Ok(index1) = alloc1 {
            let alloc2 = bitmap.alloc();
            if let Ok(index2) = alloc2 {
                // The two allocated indices should be different
                assert(index1 != index2);
                // Both bits should be set
                assert(bitmap.is_bit_set(index1 as int));
                assert(bitmap.is_bit_set(index2 as int));
            }
        }
    }
}

/// Verifiable test: clearing and re-allocating should work
fn test_bitmap_clear_and_realloc_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set a bit
        let set_result = bitmap.set(index);
        if let Ok(()) = set_result {
            let ghost bitmap_after_set = bitmap;

            // Clear the bit
            let clear_result = bitmap.clear(index);
            if let Ok(()) = clear_result {
                // The bit should be cleared
                assert(!bitmap.is_bit_set(index as int));

                // Usage should be back to 0 (since we started with 0, set 1, then cleared 1)
                assert(bitmap@.usage() == 0);
            }
        }
    }
}

/// Verifiable test: usage tracking is correct
fn test_bitmap_usage_tracking_verified(number_of_bits: usize)
    requires
        number_of_bits >= 24,  // Need at least 3 bits
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Initially empty
        assert(bitmap@.is_empty());
        assert(bitmap@.usage() == 0);

        // Allocate first bit
        let alloc1 = bitmap.alloc();
        if let Ok(_) = alloc1 {
            assert(bitmap@.usage() == 1);

            // Allocate second bit
            let alloc2 = bitmap.alloc();
            if let Ok(_) = alloc2 {
                assert(bitmap@.usage() == 2);

                // Allocate third bit
                let alloc3 = bitmap.alloc();
                if let Ok(index3) = alloc3 {
                    assert(bitmap@.usage() == 3);

                    // Clear one bit
                    let clear_result = bitmap.clear(index3);
                    if let Ok(()) = clear_result {
                        assert(bitmap@.usage() == 2);
                    }
                }
            }
        }
    }
}

/// Verifiable test: alloc_range preserves bits outside the allocated range
fn test_bitmap_alloc_range_preserves_others_verified(number_of_bits: usize, size: usize, test_index: usize)
    requires
        number_of_bits >= 16,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size < number_of_bits,
        test_index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set a bit first
        let set_result = bitmap.set(test_index);
        if let Ok(()) = set_result {
            // Allocate a range
            let alloc_result = bitmap.alloc_range(size);
            if let Ok(start_index) = alloc_result {
                // If test_index is outside the allocated range, it should still be set
                if test_index < start_index || test_index >= start_index + size {
                    assert(bitmap.is_bit_set(test_index as int));
                }
            }
        }
    }
}

/// Verifiable test: number_of_bits remains constant across operations
fn test_bitmap_number_of_bits_constant_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let ghost initial_bits = bitmap@.number_of_bits();
        assert(initial_bits == number_of_bits as int);

        // After allocation
        let alloc_result = bitmap.alloc();
        if let Ok(_) = alloc_result {
            assert(bitmap@.number_of_bits() == initial_bits);

            // After setting a bit
            let set_result = bitmap.set(index);
            match set_result {
                Ok(()) => {
                    assert(bitmap@.number_of_bits() == initial_bits);
                },
                Err(_) => {
                    // If set failed (bit already set), number_of_bits should still be the same
                    assert(bitmap@.number_of_bits() == initial_bits);
                }
            }
        }
    }
}

/// Verifiable test: setting an already-set bit should fail
fn test_bitmap_double_set_fails_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set the bit
        let set_result1 = bitmap.set(index);
        if let Ok(()) = set_result1 {
            // Try to set the same bit again
            let set_result2 = bitmap.set(index);
            // This should fail because the bit is already set
            assert(set_result2 is Err);
            // The bit should still be set
            assert(bitmap.is_bit_set(index as int));
        }
    }
}

/// Verifiable test: clearing an already-clear bit should fail
fn test_bitmap_double_clear_fails_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // The bit is initially clear, try to clear it
        let clear_result = bitmap.clear(index);
        // This should fail because the bit is already clear
        assert(clear_result is Err);
        // The bit should still be clear
        assert(!bitmap.is_bit_set(index as int));
    }
}

//================================================================================
//=====These few functions are converted from test functions in src/test.rs  =====
//=====Those test functions were originally written by Nanvix authors       ======
//================================================================================
/*
*1. **test_set_and_clear_all_bits** → **test_set_and_clear_all_bits_verified**
*2. **test_alloc_and_clear_all_bits** → **test_alloc_and_clear_all_bits_verified**
*3. **test_alloc_range_across_word_boundary** → **test_alloc_range_across_word_boundary_verified**
*4. **test_alloc_range_too_large** → **test_alloc_range_too_large_verified**
*5. **test_alloc_range_zero** → **test_alloc_range_zero_verified**
*6. **test_alloc_random_ranges** → **test_alloc_range_and_clear_verified**
*7. **test_alloc_random_bits_in_partial_bitmap** → **test_alloc_in_partial_bitmap_verified**
*8. **test_alloc_random_ranges_in_partial_bitmap** →
   **test_alloc_range_in_partial_bitmap_verified**
*/
/// Verifiable test: setting all bits and then clearing all bits
fn test_set_and_clear_all_bits_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set all bits
        let mut i: usize = 0;
        while i < number_of_bits
            invariant
                0 <= i <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
                forall|j: int| 0 <= j < i ==> bitmap.is_bit_set(j),
            decreases number_of_bits - i,
        {
            let set_result = bitmap.set(i);
            if let Ok(()) = set_result {
                i = i + 1;
            } else {
                // If set fails, we break out (this shouldn't happen for a valid index)
                break;
            }
        }

        // If we set all bits successfully
        if i == number_of_bits {
            // All bits should be set
            assert(bitmap.all_bits_set_in_range(0, number_of_bits as int));

            // Clear all bits
            let mut j: usize = 0;
            while j < number_of_bits
                invariant
                    0 <= j <= number_of_bits,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|k: int| 0 <= k < j ==> !bitmap.is_bit_set(k),
                decreases number_of_bits - j,
            {
                let clear_result = bitmap.clear(j);
                if let Ok(()) = clear_result {
                    j = j + 1;
                } else {
                    // If clear fails, we break out
                    break;
                }
            }

            // If we cleared all bits successfully
            if j == number_of_bits {
                // All bits should be cleared
                assert(bitmap.all_bits_unset_in_range(0, number_of_bits as int));
            }
        }
    }
}

/// Verifiable test: allocating all bits and then clearing all bits
fn test_alloc_and_clear_all_bits_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Allocate all bits
        let mut count: usize = 0;
        while count < number_of_bits
            invariant
                0 <= count <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
                bitmap@.usage() == count as int,
            decreases number_of_bits - count,
        {
            let alloc_result = bitmap.alloc();
            if let Ok(_) = alloc_result {
                count = count + 1;
            } else {
                // If alloc fails, we break out (this shouldn't happen if we haven't allocated all bits)
                break;
            }
        }

        // If we allocated all bits successfully
        if count == number_of_bits {
            // Usage should equal number_of_bits, so bitmap is full
            assert(bitmap@.usage() == number_of_bits as int);
            assert(bitmap@.is_full());

            // By lemma, all bits should be set
            proof {
                bitmap.lemma_is_full_means_all_bits_set();
            }
            assert(bitmap.all_bits_set_in_range(0, number_of_bits as int));

            // Clear all bits
            let mut j: usize = 0;
            while j < number_of_bits
                invariant
                    0 <= j <= number_of_bits,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|k: int| 0 <= k < j ==> !bitmap.is_bit_set(k),
                decreases number_of_bits - j,
            {
                let clear_result = bitmap.clear(j);
                if let Ok(()) = clear_result {
                    j = j + 1;
                } else {
                    // If clear fails, we break out
                    break;
                }
            }

            // If we cleared all bits successfully
            if j == number_of_bits {
                // All bits should be cleared
                assert(bitmap.all_bits_unset_in_range(0, number_of_bits as int));
            }
        }
    }
}

/// Verifiable test: allocating a range across word (byte) boundary
fn test_alloc_range_across_word_boundary_verified(number_of_bits: usize, start: usize, size: usize)
    requires
        number_of_bits >= 16,  // Need at least 2 bytes
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size <= u8::BITS as usize,
        start + size <= number_of_bits,
        start % (u8::BITS as usize) != 0 || size % (u8::BITS as usize) != 0,  // Ensure it crosses boundary
        ((start as int) / (u8::BITS as int)) != (((start + size - 1) as int) / (u8::BITS as int)),  // Must span multiple bytes
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set all bits except those in the range [start, start+size)
        let mut i: usize = 0;
        while i < start
            invariant
                0 <= i <= start <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
                forall|j: int| 0 <= j < i ==> bitmap.is_bit_set(j),
                forall|j: int| i <= j < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(j),
            decreases start - i,
        {
            let set_result = bitmap.set(i);
            if let Ok(()) = set_result {
                i = i + 1;
            } else {
                break;
            }
        }

        // Only proceed if first loop completed
        if i == start {
            // Set bits after the range
            let end_range: usize = start + size;
            let mut k: usize = end_range;
            while k < number_of_bits
                invariant
                    0 <= start < end_range <= k <= number_of_bits,
                    end_range == start + size,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|j: int| 0 <= j < start ==> bitmap.is_bit_set(j),
                    forall|j: int| start <= j < end_range ==> !bitmap.is_bit_set(j),
                    forall|j: int| end_range <= j < k ==> bitmap.is_bit_set(j),
                    forall|j: int| k <= j < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(j),
                decreases number_of_bits - k,
            {
                let set_result = bitmap.set(k);
                if let Ok(()) = set_result {
                    k = k + 1;
                } else {
                    break;
                }
            }

            // Now allocate the range if second loop completed
            if k == number_of_bits {
                // All bits outside [start, end_range) are set, so the range should be free
                assert(bitmap.all_bits_unset_in_range(start as int, end_range as int));

                let alloc_result = bitmap.alloc_range(size);
                if let Ok(allocated_start) = alloc_result {
                    // The allocated range should be within bounds and all bits should be set
                    assert(allocated_start + size <= number_of_bits);
                    assert(bitmap.all_bits_set_in_range(allocated_start as int, (allocated_start + size) as int));
                }
            }
        }
    }
}

/// Verifiable test: allocating a range larger than the bitmap should fail
fn test_alloc_range_too_large_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let size = number_of_bits + 1;
        let alloc_result = bitmap.alloc_range(size);
        // Allocating more than number_of_bits should fail
        assert(alloc_result is Err);
    }
}

/// Verifiable test: allocating a range of size 0 should fail
fn test_alloc_range_zero_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc_range(0);
        // Allocating 0 bits should fail (size must be > 0)
        assert(alloc_result is Err);
    }
}

/// Verifiable test: allocating a range, verifying it's allocated, then clearing it
fn test_alloc_range_and_clear_verified(number_of_bits: usize, size: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size <= number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc_range(size);
        if let Ok(start) = alloc_result {
            // Verify the range is allocated
            assert(bitmap.all_bits_set_in_range(start as int, (start + size) as int));

            // Clear the range
            let mut i: usize = start;
            let end = start + size;
            while i < end
                invariant
                    start <= i <= end,
                    end == start + size,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|j: int| start <= j < i ==> !bitmap.is_bit_set(j),
                decreases end - i,
            {
                let clear_result = bitmap.clear(i);
                if let Ok(()) = clear_result {
                    i = i + 1;
                } else {
                    break;
                }
            }

            // If we cleared all bits successfully
            if i == end {
                // All bits in the range should be cleared
                assert(bitmap.all_bits_unset_in_range(start as int, end as int));
            }
        }
    }
}

/// Verifiable test: allocating a bit in a partially filled bitmap
fn test_alloc_in_partial_bitmap_verified(number_of_bits: usize, set_index: usize)
    requires
        number_of_bits >= 16,  // Need at least 2 bits
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        set_index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set one bit to partially fill the bitmap
        let set_result = bitmap.set(set_index);
        if let Ok(()) = set_result {
            // The set bit should be marked as set
            assert(bitmap.is_bit_set(set_index as int));

            // Allocate a new bit
            let alloc_result = bitmap.alloc();
            if let Ok(index) = alloc_result {
                // The allocated bit should be set
                assert(bitmap.is_bit_set(index as int));
                // The allocated bit should be different from the manually set bit (if possible)
                // Note: This may not always be true if the only free bit was set_index
                // But since we started with an empty bitmap and set one bit, there should be other free bits

                // Clear the allocated bit
                let clear_result = bitmap.clear(index);
                if let Ok(()) = clear_result {
                    assert(!bitmap.is_bit_set(index as int));
                    // The originally set bit should still be set (if it wasn't the one we allocated)
                    if index != set_index {
                        assert(bitmap.is_bit_set(set_index as int));
                    }
                }
            }
        }
    }
}

/// Verifiable test: allocating a range in a partially filled bitmap, then clearing it
fn test_alloc_range_in_partial_bitmap_verified(number_of_bits: usize, start: usize, size: usize)
    requires
        number_of_bits >= 16,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size <= u8::BITS as usize,
        start + size <= number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set all bits except [start, start+size)
        let mut i: usize = 0;
        while i < start
            invariant
                0 <= i <= start <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
                forall|j: int| 0 <= j < i ==> bitmap.is_bit_set(j),
                forall|j: int| i <= j < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(j),
            decreases start - i,
        {
            let set_result = bitmap.set(i);
            if let Ok(()) = set_result {
                i = i + 1;
            } else {
                break;
            }
        }

        if i == start {
            let end_range = start + size;
            let mut k: usize = end_range;
            while k < number_of_bits
                invariant
                    0 <= start < end_range <= k <= number_of_bits,
                    end_range == start + size,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|j: int| 0 <= j < start ==> bitmap.is_bit_set(j),
                    forall|j: int| start <= j < end_range ==> !bitmap.is_bit_set(j),
                    forall|j: int| end_range <= j < k ==> bitmap.is_bit_set(j),
                    forall|j: int| k <= j < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(j),
                decreases number_of_bits - k,
            {
                let set_result = bitmap.set(k);
                if let Ok(()) = set_result {
                    k = k + 1;
                } else {
                    break;
                }
            }

            if k == number_of_bits {
                // All bits outside [start, end_range) are set
                assert(bitmap.all_bits_unset_in_range(start as int, end_range as int));

                // Allocate the range
                let alloc_result = bitmap.alloc_range(size);
                if let Ok(allocated_start) = alloc_result {
                    // The allocated range should be set
                    assert(bitmap.all_bits_set_in_range(allocated_start as int, (allocated_start + size) as int));

                    // Clear all bits
                    let mut m: usize = 0;
                    while m < number_of_bits
                        invariant
                            0 <= m <= number_of_bits,
                            bitmap.inv(),
                            bitmap@.number_of_bits() == number_of_bits as int,
                            forall|j: int| 0 <= j < m ==> !bitmap.is_bit_set(j),
                        decreases number_of_bits - m,
                    {
                        let clear_result = bitmap.clear(m);
                        if let Ok(()) = clear_result {
                            m = m + 1;
                        } else {
                            break;
                        }
                    }

                    if m == number_of_bits {
                        assert(bitmap.all_bits_unset_in_range(0, number_of_bits as int));
                    }
                }
            }
        }
    }
}


} //Verus

//==================================================================================================
// Tests
//==================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_new() {
        let bitmap: Bitmap = Bitmap::new(64).unwrap();
        assert_eq!(bitmap.number_of_bits(), 64);
    }

    #[test]
    fn test_bitmap_alloc() {
        let mut bitmap: Bitmap = Bitmap::new(64).unwrap();
        let index: usize = bitmap.alloc().unwrap();
        assert_eq!(index, 0);
        assert!(bitmap.test(index).unwrap());
    }

    #[test]
    fn test_bitmap_set_clear() {
        let mut bitmap: Bitmap = Bitmap::new(64).unwrap();
        bitmap.set(10).unwrap();
        assert!(bitmap.test(10).unwrap());
        bitmap.clear(10).unwrap();
        assert!(!bitmap.test(10).unwrap());
    }

    #[test]
    fn test_bitmap_alloc_range() {
        let mut bitmap: Bitmap = Bitmap::new(64).unwrap();
        let index: usize = bitmap.alloc_range(8).unwrap();
        assert_eq!(index, 0);
        for i in 0..8 {
            assert!(bitmap.test(i).unwrap());
        }
    }
}
