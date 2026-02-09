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
//! - **`now()` arithmetic safety**: The nanosecond computation
//!   `(minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq)`
//!   does not overflow `u32` and produces a value strictly less than
//!   `NANOSECONDS_PER_SECOND`, ensuring `SystemTime::new()` never returns `None`.
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
//! - The `now()` nanosecond computation is safe and satisfies
//!   `SystemTime::new()`'s precondition (`nanoseconds < NANOSECONDS_PER_SECOND`).
//!
//! Explicitly **out of scope**:
//! - **Concurrency**: The sequential model does not capture concurrent access.
//! - **SystemTime construction**: `SystemTime::new()` is external; we model
//!   only its precondition (`nanoseconds < NANOSECONDS_PER_SECOND`).
//! - **Platform-specific timer frequency**: The actual `timer_freq` value is
//!   determined at runtime; we prove safety for all `timer_freq > 0`.
//!
//! ## API Divergence
//!
//! **`&self` → `&mut self` for `increment()`:** The original `increment(&self)`
//! uses `AtomicU32` interior mutability, allowing shared-reference access. The
//! verified model uses `&mut self` because Verus requires exclusive references
//! for state mutation. This `&mut self` requirement is strictly stronger than
//! the original's `&self` + single-writer assumption. The verified model does
//! **not** prove absence of data races — it proves sequential arithmetic
//! correctness under the assumption that only one writer exists (the timer
//! interrupt handler on a single core).
//!
//! **Global singleton not modeled:** The original code uses a `static TIMER_TICKS`
//! global variable. The verified model operates on arbitrary `TimerTicks`
//! instances, not the global singleton. The singleton access pattern is trusted.
//!
//! **`pub` fields:** Fields `minor` and `major` are `pub` in the verified version
//! (required for Verus `pub open spec fn` access) but private in the original.
//! Since `wf()` is universally true (`lemma_always_wf`), this does not introduce
//! unsoundness, but it weakens encapsulation. The original enforces that only
//! `new()` and `increment()` create/modify `TimerTicks` values.
//!
//! ## Trust Boundaries
//!
//! - **T1: AtomicU32 → plain u32.** Atomics are modeled as plain fields under
//!   the single-writer assumption. The original's memory ordering semantics
//!   (`Ordering::Relaxed`) are not modeled.
//! - **T2: `wrapping_add(1)` → explicit branching.** Modeled as `if minor < MAX`
//!   branching rather than hardware wrapping.
//! - **T3: `timer_handler()`.** This function is **trusted glue code**: it calls
//!   `increment()` exactly once per timer interrupt and does not modify
//!   `major`/`minor` through any other path. Its HAL dependencies
//!   (`InterruptNumber`, `ProcessManager`, platform-specific `#[cfg]` blocks)
//!   are external to the verification. The verified `increment()` contract
//!   establishes what each call achieves, but the handler-level invariant
//!   (single-call-per-interrupt) is assumed, not proved.
//! - **T4: `SystemTime::new()`.** Modeled only via its precondition
//!   (`nanoseconds < NANOSECONDS_PER_SECOND`). The actual SystemTime type is
//!   external.

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
/// Fields are `pub` for Verus spec reasoning (required by `pub open spec fn`).
/// The original uses `AtomicU32` with private fields. External construction of
/// arbitrary `TimerTicks` values is possible in the verified model but not in
/// the original. Since `wf()` is universally true (see `lemma_always_wf`),
/// this does not introduce unsoundness.
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
    /// # API Divergence
    ///
    /// The original uses `&self` with `AtomicU32` interior mutability. This
    /// verified model uses `&mut self` because Verus requires exclusive
    /// references for state mutation. The `&mut self` requirement is strictly
    /// stronger than `&self` + single-writer: it proves correctness under
    /// exclusive access but does not prove absence of data races.
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
                assert(old(self).spec_minor() < u32::MAX as nat);
                assert(old(self).spec_major() <= u32::MAX as nat);
                Self::lemma_nat_mul_le_mono(
                    old(self).spec_major(), u32::MAX as nat, Self::MINOR_MODULUS(),
                );
                assert(!old(self).spec_is_max());
                assert(self.spec_ticks() == old(self).spec_ticks() + 1);
                assert(self.spec_ticks() == old(self).spec_next_ticks());
                self.lemma_always_wf();
            }
        } else {
            self.minor = 0;
            if self.major < u32::MAX {
                self.major = self.major + 1;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    assert(old(self).spec_major() < u32::MAX as nat);
                    Self::lemma_nat_mul_le_mono(
                        old(self).spec_major(), (u32::MAX - 1) as nat, Self::MINOR_MODULUS(),
                    );
                    assert(old(self).spec_ticks() < u64::MAX as nat);
                    assert(!old(self).spec_is_max());
                    assert(self.spec_ticks() == (old(self).spec_major() + 1) * Self::MINOR_MODULUS());
                    assert(self.spec_ticks() == old(self).spec_ticks() + 1);
                    assert(self.spec_ticks() == old(self).spec_next_ticks());
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
                    assert(self.spec_ticks() == old(self).spec_next_ticks());
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
    /// # Description
    ///
    /// Verification-only helper; not present in the original source.
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
    /// # Description
    ///
    /// Verification-only helper; not present in the original source.
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

    /// Computes the nanosecond component of the current time.
    ///
    /// # Description
    ///
    /// Models the arithmetic from the original `now()` function:
    /// `(minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq)`.
    ///
    /// This function proves that the computation:
    /// - Does not overflow `u32`.
    /// - Produces a result strictly less than `NANOSECONDS_PER_SECOND` (1,000,000,000),
    ///   which satisfies the precondition of `SystemTime::new()`.
    ///
    /// # Parameters
    ///
    /// - `minor_ticks`: The current minor tick count.
    /// - `timer_freq`: The timer frequency in Hz. Must be > 0.
    ///
    /// # Returns
    ///
    /// The nanosecond component, guaranteed < `NANOSECONDS_PER_SECOND`.
    pub fn compute_nanoseconds(minor_ticks: u32, timer_freq: u32) -> (result: u32)
        requires
            timer_freq > 0,
        ensures
            result as nat == Self::spec_compute_nanoseconds(minor_ticks, timer_freq),
            Self::spec_nanoseconds_valid(result as nat),
            result < 1_000_000_000u32,
    {
        proof {
            Self::lemma_nanoseconds_in_range(minor_ticks, timer_freq);
        }
        (minor_ticks % timer_freq) * (1_000_000_000u32 / timer_freq)
    }

    /// Computes the seconds component of the current time.
    ///
    /// # Description
    ///
    /// Models the seconds computation from the original `now()` function:
    /// `(((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64)`.
    ///
    /// # Parameters
    ///
    /// - `major_ticks`: The current major tick count.
    /// - `minor_ticks`: The current minor tick count.
    /// - `timer_freq`: The timer frequency in Hz. Must be > 0.
    ///
    /// # Returns
    ///
    /// The seconds component.
    pub fn compute_seconds(major_ticks: u32, minor_ticks: u32, timer_freq: u32) -> (result: u64)
        requires
            timer_freq > 0,
        ensures
            result as nat == Self::spec_compute_seconds(major_ticks, minor_ticks, timer_freq),
    {
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        let total_ticks: u64 = (major_ticks as u64) * 0x1_0000_0000u64 + (minor_ticks as u64);
        total_ticks / (timer_freq as u64)
    }
}

} // verus!
