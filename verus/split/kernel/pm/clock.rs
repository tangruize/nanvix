// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # TimerTicks Implementation
//!
//! A 64-bit tick counter split into two 32-bit halves (major, minor).
//!
//! ## Verified Properties
//!
//! - A new counter starts at zero ticks.
//! - Any (u32, u32) pair is well-formed (ticks <= u64::MAX).
//! - `increment()` increases ticks by exactly 1, or wraps from u64::MAX to 0.
//! - `get()` returns the current (major, minor) values.
//! - `ticks()` correctly combines major and minor into a u64.
//! - Monotonicity: each non-wrapping increment increases the tick count.
//!
//! ## Verification Model
//!
//! The original implementation uses `core::sync::atomic::AtomicU32` for the
//! major and minor counters with a single-writer assumption. For verification,
//! we model these as plain `u32` fields and use `&mut self` for state
//! transitions. This is a sequential model that verifies the counter
//! arithmetic without reasoning about atomicity.
//!
//! **This verified code is a specification model, not a runtime replacement.**
//! The kernel uses the original `src/kernel/src/pm/clock.rs` (with `AtomicU32`
//! and `wrapping_add`) at runtime.
//!
//! ## Verification Scope
//!
//! This verification proves **counter arithmetic correctness**:
//! - The split (major, minor) representation correctly models a 64-bit counter.
//! - Wrapping behavior at u64::MAX is handled correctly.
//! - The `ticks()` combination does not overflow.
//!
//! Explicitly **out of scope**:
//! - **Concurrency**: The sequential model does not capture concurrent access.
//! - **Timer interrupt handling**: `timer_handler()` depends on HAL and
//!   ProcessManager, which are external.
//! - **SystemTime conversion**: `now()` depends on platform-specific timer
//!   frequency and SystemTime, which are external.
//!
//! ## Trust Boundaries
//!
//! - `AtomicU32` is modeled as plain `u32` (sequential single-writer assumption).
//! - `wrapping_add(1)` is modeled as explicit branching on u32::MAX.
//! - `timer_handler()` and `now()` are not included as they depend on
//!   external OS state (InterruptNumber, ProcessManager, SystemTime).

use vstd::prelude::*;

// Include specifications.
include!("clock.spec.rs");

// Include proofs.
include!("clock.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A 64-bit tick counter split into two 32-bit halves.
///
/// # Description
///
/// TimerTicks counts the number of timer interrupts since system startup.
/// The counter is split into major (high 32 bits) and minor (low 32 bits)
/// to model the original atomic implementation.
///
/// # Representation
///
/// Fields are `pub` for Verus spec reasoning. The original uses AtomicU32.
pub struct TimerTicks {
    /// Low 32 bits of the tick counter.
    pub minor: u32,
    /// High 32 bits of the tick counter.
    pub major: u32,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl TimerTicks {
    /// Creates a new TimerTicks counter initialized to zero.
    ///
    /// # Description
    ///
    /// Models the original `const fn new()` which creates AtomicU32 pairs.
    ///
    /// # Returns
    ///
    /// A new TimerTicks with both halves set to zero.
    pub fn new() -> (result: Self)
        ensures
            result.minor == 0,
            result.major == 0,
            result.spec_ticks() == 0,
            result.spec_is_zero(),
            result@ == TimerTicks::spec_new_view(),
            result.wf(),
    {
        TimerTicks { minor: 0, major: 0 }
    }

    /// Returns the current (major, minor) tick counts.
    ///
    /// # Description
    ///
    /// Models the original `get()` which loads from AtomicU32.
    ///
    /// # Returns
    ///
    /// A tuple of (major, minor) tick counts.
    pub fn get(&self) -> (result: (u32, u32))
        ensures
            result.0 == self.major,
            result.1 == self.minor,
            result.0 as nat == self.spec_major(),
            result.1 as nat == self.spec_minor(),
    {
        (self.major, self.minor)
    }

    /// Increments the tick counter by one.
    ///
    /// # Description
    ///
    /// Models `wrapping_add(1)` on the minor counter. When the minor counter
    /// wraps to 0, the major counter is also incremented (wrapping if at max).
    ///
    /// # Returns
    ///
    /// The new minor tick value after incrementing.
    pub fn increment(&mut self) -> (result: u32)
        requires
            old(self).wf(),
        ensures
            self.wf(),
            result == self.minor,
            old(self).spec_is_max() ==> self.spec_ticks() == 0,
            !old(self).spec_is_max() ==> self.spec_ticks() == old(self).spec_ticks() + 1,
            self.spec_ticks() == old(self).spec_next_ticks(),
    {
        if self.minor < u32::MAX {
            self.minor = self.minor + 1;
            proof {
                assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                // old(self).minor < u32::MAX, so old(self).spec_minor() <= u32::MAX - 1.
                assert(old(self).spec_minor() < u32::MAX as nat);
                assert(old(self).spec_major() <= u32::MAX as nat);
                Self::lemma_nat_mul_le_mono(
                    old(self).spec_major(), u32::MAX as nat, Self::MINOR_MODULUS(),
                );
                // Now: old.major * M <= u32::MAX * M.
                // old.ticks = old.major * M + old.minor <= u32::MAX * M + (u32::MAX - 1) < u64::MAX.
                assert(!old(self).spec_is_max());
                self.lemma_always_wf();
            }
        } else {
            self.minor = 0;
            if self.major < u32::MAX {
                self.major = self.major + 1;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    // old(self).major < u32::MAX, so old(self) is not at max.
                    assert(old(self).spec_major() < u32::MAX as nat);
                    Self::lemma_nat_mul_le_mono(
                        old(self).spec_major(), (u32::MAX - 1) as nat, Self::MINOR_MODULUS(),
                    );
                    // old.major * M <= (u32::MAX - 1) * M, so old.ticks < u64::MAX.
                    assert(old(self).spec_ticks() < u64::MAX as nat);
                    assert(!old(self).spec_is_max());
                    // (old.major + 1) * M = old.major * M + M = old.major * M + u32::MAX + 1.
                    assert(self.spec_ticks() == (old(self).spec_major() + 1) * Self::MINOR_MODULUS());
                    self.lemma_always_wf();
                }
            } else {
                self.major = 0;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    assert(old(self).spec_ticks() == u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat);
                    assert(old(self).spec_is_max());
                    assert(self.spec_ticks() == 0);
                    self.lemma_always_wf();
                }
            }
        }
        self.minor
    }

    /// Returns the combined 64-bit tick count.
    ///
    /// # Description
    ///
    /// Combines major and minor into a single u64: `major * 2^32 + minor`.
    /// Models the original standalone `ticks()` function.
    ///
    /// # Returns
    ///
    /// The combined tick count as u64.
    pub fn ticks(&self) -> (result: u64)
        ensures
            result as nat == self.spec_ticks(),
    {
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        (self.major as u64) * 0x1_0000_0000u64 + (self.minor as u64)
    }

    /// Checks if the counter is at the maximum value.
    ///
    /// # Returns
    ///
    /// `true` if ticks == u64::MAX, `false` otherwise.
    pub fn is_max(&self) -> (result: bool)
        ensures
            result == self.spec_is_max(),
    {
        proof {
            Self::lemma_max_ticks_value();
        }
        self.minor == u32::MAX && self.major == u32::MAX
    }

    /// Checks if the counter is zero.
    ///
    /// # Returns
    ///
    /// `true` if ticks == 0, `false` otherwise.
    pub fn is_zero(&self) -> (result: bool)
        ensures
            result == self.spec_is_zero(),
    {
        proof {
            assert(Self::MINOR_MODULUS() > 0);
            if self.major > 0 {
                assert(self.spec_major() >= 1);
                assert(self.spec_major() * Self::MINOR_MODULUS() >= Self::MINOR_MODULUS());
                assert(self.spec_ticks() >= Self::MINOR_MODULUS());
            }
        }
        self.minor == 0 && self.major == 0
    }
}

} // verus!
