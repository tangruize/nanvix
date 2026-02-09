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

    //==============================================================================================
    // now() Arithmetic Specs
    //==============================================================================================

    /// Constant: nanoseconds per second (1,000,000,000).
    pub open spec fn NANOSECONDS_PER_SECOND() -> nat {
        1_000_000_000
    }

    /// Spec function: checks if a timer frequency is valid (non-zero).
    pub open spec fn spec_timer_freq_valid(timer_freq: u32) -> bool {
        timer_freq > 0
    }

    /// Spec function: computes the nanosecond component of the current time.
    ///
    /// # Description
    ///
    /// Models the original expression:
    /// `(minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq)`.
    pub open spec fn spec_compute_nanoseconds(minor_ticks: u32, timer_freq: u32) -> nat
        recommends timer_freq > 0,
    {
        (minor_ticks as nat % timer_freq as nat) * (Self::NANOSECONDS_PER_SECOND() / timer_freq as nat)
    }

    /// Spec function: computes the seconds component of the current time.
    ///
    /// # Description
    ///
    /// Models the original expression:
    /// `(((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64)`.
    pub open spec fn spec_compute_seconds(major_ticks: u32, minor_ticks: u32, timer_freq: u32) -> nat
        recommends timer_freq > 0,
    {
        let total_ticks: nat = major_ticks as nat * Self::MINOR_MODULUS() + minor_ticks as nat;
        total_ticks / timer_freq as nat
    }

    /// Spec function: checks if nanoseconds are valid for SystemTime::new().
    ///
    /// # Description
    ///
    /// SystemTime::new(seconds, nanoseconds) returns None iff
    /// nanoseconds >= NANOSECONDS_PER_SECOND.
    pub open spec fn spec_nanoseconds_valid(nanoseconds: nat) -> bool {
        nanoseconds < Self::NANOSECONDS_PER_SECOND()
    }

    //==============================================================================================
    // now() Composed Specs
    //==============================================================================================

    /// Spec function: full `now()` result as a (seconds, nanoseconds) pair.
    ///
    /// # Description
    ///
    /// Models the complete `now()` computation from the original source.
    /// Given a TimerTicks state and a timer frequency, computes both the
    /// seconds and nanoseconds components.
    pub open spec fn spec_now(&self, timer_freq: u32) -> (nat, nat)
        recommends timer_freq > 0,
    {
        (
            Self::spec_compute_seconds(self.major, self.minor, timer_freq),
            Self::spec_compute_nanoseconds(self.minor, timer_freq),
        )
    }

    /// Spec function: the seconds component of `now()` is consistent with `ticks()`.
    ///
    /// # Description
    ///
    /// The seconds value equals `spec_ticks() / timer_freq`, which is the
    /// same as `ticks() / timer_freq` at the exec level.
    pub open spec fn spec_seconds_consistent_with_ticks(&self, timer_freq: u32) -> bool
        recommends timer_freq > 0,
    {
        Self::spec_compute_seconds(self.major, self.minor, timer_freq) == self.spec_ticks() / timer_freq as nat
    }

    //==============================================================================================
    // get() Consistency Specs
    //==============================================================================================

    /// Spec function: a (major, minor) pair from `get()` is consistent with `spec_ticks()`.
    ///
    /// # Description
    ///
    /// Given a pair returned by `get()`, the combined tick count equals
    /// `major * MINOR_MODULUS + minor`, which is exactly `spec_ticks()`.
    /// This establishes that the pair is a consistent snapshot.
    ///
    /// # Note on Atomics
    ///
    /// The original `get()` loads `major` and `minor` in two separate atomic
    /// loads. Under the single-writer assumption (timer interrupt handler on
    /// one core), tearing cannot occur because the writer is not concurrent
    /// with the reader. This consistency property is assumed, not proved —
    /// see Trust Boundary T1.
    pub open spec fn spec_get_consistent(&self, major: u32, minor: u32) -> bool {
        major as nat * Self::MINOR_MODULUS() + minor as nat == self.spec_ticks()
    }

    //==============================================================================================
    // timer_handler Behavioral Specs
    //==============================================================================================

    /// Spec function: models the observable effect of one `timer_handler()` call.
    ///
    /// # Description
    ///
    /// The timer handler's only effect on the clock state is to call
    /// `increment()` exactly once. The result state has ticks equal to
    /// `spec_next_ticks()` of the pre-state. The handler's other side effects
    /// (VM pause check via volatile read, context switch via `ProcessManager::giveup()`)
    /// are HAL/scheduler interactions that do not modify the clock counter.
    pub open spec fn spec_timer_handler_effect(&self, post: &TimerTicks) -> bool {
        post.spec_ticks() == self.spec_next_ticks()
    }

    //==============================================================================================
    // Timer Frequency Specs
    //==============================================================================================

    /// Spec function: platform timer frequency guarantee.
    ///
    /// # Description
    ///
    /// On all supported platforms, the timer frequency is positive:
    /// - `#[cfg(feature = "pit")]`: `pit::get_timer_frequency()` returns the PIT
    ///   oscillator frequency divided by the programmed divisor, which is always > 0.
    /// - `#[cfg(not(feature = "pit"))]`: the fallback sets `timer_freq = 1`.
    ///
    /// This spec captures the platform invariant that justifies the
    /// `timer_freq > 0` precondition on `compute_nanoseconds` and
    /// `compute_seconds`.
    pub open spec fn spec_platform_timer_freq_valid(timer_freq: u32) -> bool {
        timer_freq > 0
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
