// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {


/// Number of entries in the kernel red zone (spec version).
pub spec const SPEC_NUM_ENTRIES: int = KREDZONE_SIZE as int / ENTRY_SIZE as int;

//==================================================================================================

/// Checks if an index is valid (within bounds).
pub open spec fn spec_is_valid_index(index: int) -> bool {
    0 <= index < SPEC_NUM_ENTRIES
}


impl KernelRedZoneView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the number of entries in the red zone.
    pub open spec fn len(&self) -> nat {
        self.contents.len()
    }

    /// Returns the value at index i.
    pub open spec fn index(&self, i: int) -> usize
        recommends 0 <= i < self.len() as int
    {
        self.contents[i]
    }

    /// Returns a new view with the element at index i updated to value.
    pub open spec fn update(&self, i: int, value: usize) -> KernelRedZoneView
        recommends 0 <= i < self.len() as int
    {
        KernelRedZoneView {
            contents: self.contents.update(i, value),
        }
    }

    /// Checks if an index is within bounds.
    pub open spec fn in_bounds(&self, i: int) -> bool {
        0 <= i < self.len() as int
    }

    //==============================================================================================
    // Well-formedness
    //==============================================================================================

    /// Returns true if the view represents a well-formed kernel red zone.
    pub open spec fn is_well_formed(&self) -> bool {
        self.len() == SPEC_NUM_ENTRIES as nat
    }
}

//==================================================================================================

/// Ghost state representing the abstract kernel red zone.
///
/// Since the kernel red zone is a global static variable accessed via `extern "C"`,
/// we cannot thread tracked ghost state through the `store`/`load` functions directly.
/// Instead, this struct provides an abstract model for **caller-side reasoning**.
///
/// ## Usage Pattern
///
/// Callers who need to reason about sequences of store/load operations can:
///
/// 1. Maintain a `ghost view: KernelRedZoneView` that tracks expected state.
/// 2. After calling `store(i, v)`, update ghost state: `view = spec_store_effect(view, i, v)`.
/// 3. Before calling `load(i)`, assert expected value: `spec_load_result(view, i)`.
///
/// The correctness lemmas (`lemma_store_then_load`, etc.) prove that if the ghost
/// state is maintained correctly and the trust assumptions hold, the abstract
/// reasoning is valid.
///
/// ## Why Not Tracked Parameters?
///
/// The original implementation uses an `extern "C"` static variable, which:
/// - Has no Rust ownership model (it's assembly-defined).
/// - Cannot accept tracked/ghost parameters (fixed C ABI).
/// - Is accessed via raw pointer arithmetic with volatile operations.
///
/// Therefore, we document the trust boundary rather than attempting to force
/// ghost state through an incompatible interface.
#[verifier::ext_equal]
pub tracked struct KernelRedZoneGhost {
    pub ghost view: KernelRedZoneView,
}


impl KernelRedZoneGhost {
    /// Returns true if the ghost state is well-formed.
    pub open spec fn inv(&self) -> bool {
        self.view.is_well_formed()
    }

    /// Returns the abstract view.
    pub open spec fn view(&self) -> KernelRedZoneView {
        self.view
    }
}

//==================================================================================================

/// Specification function: Effect of a store operation on abstract state.
///
/// Given the current view and a valid store operation, returns the resulting view.
pub open spec fn spec_store_effect(
    view: KernelRedZoneView,
    index: int,
    value: usize,
) -> KernelRedZoneView
    recommends
        view.is_well_formed(),
        spec_is_valid_index(index),
{
    view.update(index, value)
}


/// Specification function: Result of a load operation on abstract state.
///
/// Given the current view and a valid load operation, returns the value.
pub open spec fn spec_load_result(view: KernelRedZoneView, index: int) -> usize
    recommends
        view.is_well_formed(),
        spec_is_valid_index(index),
{
    view.index(index)
}

} // verus!
