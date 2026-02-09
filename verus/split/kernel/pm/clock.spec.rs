// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// TimerTicks Specification.
// This file contains spec functions for the TimerTicks type.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a TimerTicks counter.
///
/// # Description
///
/// Represents the observable state of the timer as a single combined tick count.
/// The split (major, minor) representation is an implementation detail.
#[verifier::ext_equal]
pub struct TimerTicksView {
    /// Combined 64-bit tick count.
    pub ticks: nat,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl TimerTicks {
    /// Spec function: the modulus for the minor counter (2^32 = u32::MAX + 1).
    ///
    /// # Description
    ///
    /// This is the number of minor ticks per major tick epoch.
    /// Equal to 0x1_0000_0000 = 4294967296.
    pub open spec fn MINOR_MODULUS() -> nat {
        u32::MAX as nat + 1
    }

    /// Spec function: returns the minor tick count.
    pub open spec fn spec_minor(&self) -> nat {
        self.minor as nat
    }

    /// Spec function: returns the major tick count.
    pub open spec fn spec_major(&self) -> nat {
        self.major as nat
    }

    /// Spec function: returns the combined 64-bit tick count.
    ///
    /// # Description
    ///
    /// The abstract tick count is `major * 2^32 + minor`.
    pub open spec fn spec_ticks(&self) -> nat {
        self.spec_major() * Self::MINOR_MODULUS() + self.spec_minor()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// # Description
    ///
    /// The tick count is representable as a u64. This is always true for
    /// valid (u32, u32) pairs, as proved by `lemma_always_wf`.
    pub open spec fn wf(&self) -> bool {
        self.spec_ticks() <= u64::MAX as nat
    }

    /// Spec function: whether the counter is at its maximum value (u64::MAX).
    pub open spec fn spec_is_max(&self) -> bool {
        self.spec_ticks() == u64::MAX as nat
    }

    /// Spec function: whether the counter is zero.
    pub open spec fn spec_is_zero(&self) -> bool {
        self.spec_ticks() == 0
    }

    /// Spec function: the expected tick count after one increment.
    ///
    /// # Description
    ///
    /// If at max, wraps to 0. Otherwise, increases by 1.
    pub open spec fn spec_next_ticks(&self) -> nat {
        if self.spec_is_max() { 0 } else { self.spec_ticks() + 1 }
    }

    /// Spec function: the view of a newly created TimerTicks.
    pub open spec fn spec_new_view() -> TimerTicksView {
        TimerTicksView { ticks: 0 }
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for TimerTicks {
    type V = TimerTicksView;

    open spec fn view(&self) -> TimerTicksView {
        TimerTicksView { ticks: self.spec_ticks() }
    }
}

} // verus!
