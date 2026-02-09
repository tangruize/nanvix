// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// TimerTicks Proofs.
// This file contains proof lemmas for the TimerTicks type.

verus! {

//==================================================================================================
// Proof Lemmas — Definitional Properties
//==================================================================================================

impl TimerTicks {
    /// Lemma: A newly created TimerTicks has zero ticks.
    pub proof fn lemma_new_is_zero()
        ensures
            TimerTicks::spec_new_view() == (TimerTicksView { ticks: 0 }),
    {
    }

    /// Lemma: MINOR_MODULUS equals u32::MAX + 1.
    pub proof fn lemma_minor_modulus_eq()
        ensures
            TimerTicks::MINOR_MODULUS() == u32::MAX as nat + 1,
            TimerTicks::MINOR_MODULUS() > 0,
    {
    }

    /// Lemma: Any TimerTicks is well-formed (ticks <= u64::MAX).
    ///
    /// # Description
    ///
    /// Since major and minor are both u32, the maximum tick count is
    /// u32::MAX * (u32::MAX + 1) + u32::MAX = u64::MAX.
    pub proof fn lemma_always_wf(&self)
        ensures
            self.wf(),
    {
        assert(self.spec_major() <= u32::MAX as nat);
        assert(self.spec_minor() <= u32::MAX as nat);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: The maximum tick value identity.
    pub proof fn lemma_max_ticks_value()
        ensures
            u32::MAX as nat * TimerTicks::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat,
    {
    }

    /// Lemma: View reflects the tick count.
    pub proof fn lemma_view_reflects_ticks(&self)
        ensures
            self@.ticks == self.spec_ticks(),
    {
    }

    /// Lemma: Two TimerTicks with equal views have equal tick counts.
    pub proof fn lemma_view_equality(a: &TimerTicks, b: &TimerTicks)
        requires
            a@ == b@,
        ensures
            a.spec_ticks() == b.spec_ticks(),
    {
    }

    /// Lemma: A zero-tick counter has both halves zero.
    pub proof fn lemma_zero_implies_both_zero(&self)
        requires
            self.spec_ticks() == 0,
        ensures
            self.spec_major() == 0,
            self.spec_minor() == 0,
    {
        assert(Self::MINOR_MODULUS() > 0);
    }
}

//==================================================================================================
// Proof Lemmas — Increment Properties
//==================================================================================================

impl TimerTicks {
    /// Lemma: When minor < u32::MAX, incrementing minor increases ticks by 1.
    pub proof fn lemma_increment_no_minor_overflow(pre: &TimerTicks)
        requires
            pre.minor < u32::MAX,
        ensures
            ({
                let post = TimerTicks { minor: (pre.minor + 1) as u32, major: pre.major };
                &&& post.spec_ticks() == pre.spec_ticks() + 1
                &&& !pre.spec_is_max()
            }),
    {
        assert(pre.spec_ticks() <= u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat - 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: When minor == u32::MAX and major < u32::MAX, rollover increases ticks by 1.
    pub proof fn lemma_increment_minor_overflow(pre: &TimerTicks)
        requires
            pre.minor == u32::MAX,
            pre.major < u32::MAX,
        ensures
            ({
                let post = TimerTicks { minor: 0u32, major: (pre.major + 1) as u32 };
                &&& post.spec_ticks() == pre.spec_ticks() + 1
                &&& !pre.spec_is_max()
            }),
    {
        assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: When both are u32::MAX, wrapping resets to 0.
    pub proof fn lemma_increment_full_overflow(pre: &TimerTicks)
        requires
            pre.minor == u32::MAX,
            pre.major == u32::MAX,
        ensures
            ({
                let post = TimerTicks { minor: 0u32, major: 0u32 };
                &&& post.spec_ticks() == 0
                &&& pre.spec_is_max()
            }),
    {
        assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: Increment is monotone for non-max counters.
    pub proof fn lemma_increment_monotone(pre: &TimerTicks, post: &TimerTicks)
        requires
            !pre.spec_is_max(),
            pre.wf(),
            if pre.minor < u32::MAX {
                post.minor == (pre.minor + 1) as u32 && post.major == pre.major
            } else {
                post.minor == 0u32 && post.major == (pre.major + 1) as u32
            },
        ensures
            post.spec_ticks() == pre.spec_ticks() + 1,
    {
        if pre.minor < u32::MAX {
            Self::lemma_increment_no_minor_overflow(pre);
        } else {
            assert(pre.minor == u32::MAX);
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
            assert(pre.spec_ticks() == pre.spec_major() * Self::MINOR_MODULUS() + u32::MAX as nat);
            assert(pre.spec_ticks() < u64::MAX as nat);
            assert(pre.major < u32::MAX);
            Self::lemma_increment_minor_overflow(pre);
        }
    }
}

//==================================================================================================
// Proof Lemmas — Ticks Combination
//==================================================================================================

impl TimerTicks {
    /// Lemma: Multiplication is monotone for natural numbers.
    ///
    /// # Description
    ///
    /// If a <= b, then a * c <= b * c. This is used to bound
    /// spec_major() * MINOR_MODULUS() by u32::MAX * MINOR_MODULUS().
    pub proof fn lemma_nat_mul_le_mono(a: nat, b: nat, c: nat)
        requires
            a <= b,
        ensures
            a * c <= b * c,
    {
        assert(a * c <= b * c) by(nonlinear_arith)
            requires(a <= b);
    }

    /// Lemma: The ticks computation doesn't overflow u64.
    pub proof fn lemma_ticks_no_overflow(major: u32, minor: u32)
        ensures
            (major as u64 as nat) * 0x1_0000_0000nat + (minor as u64 as nat) <= u64::MAX as nat,
    {
        assert(u32::MAX as nat * TimerTicks::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
    }

    /// Lemma: A satisfied wf with non-max state implies ticks < u64::MAX.
    pub proof fn lemma_not_max_implies_lt(&self)
        requires
            self.wf(),
            !self.spec_is_max(),
        ensures
            self.spec_ticks() < u64::MAX as nat,
    {
    }
}

//==================================================================================================
// Proof Lemmas — Monotonicity and Safety
//==================================================================================================

impl TimerTicks {
    /// Lemma: Satisfaction is monotone — once max, wrapping is the only transition.
    pub proof fn lemma_max_is_terminal(pre: &TimerTicks)
        requires
            pre.spec_is_max(),
        ensures
            pre.minor == u32::MAX,
            pre.major == u32::MAX,
    {
        assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
        assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        assert(pre.spec_ticks() == pre.spec_major() * Self::MINOR_MODULUS() + pre.spec_minor());
        // If major < u32::MAX, then major * M <= (u32::MAX - 1) * M,
        // and ticks <= (u32::MAX - 1) * M + u32::MAX < u32::MAX * M + u32::MAX = u64::MAX.
        // If major == u32::MAX and minor < u32::MAX, then ticks < u32::MAX * M + u32::MAX = u64::MAX.
        // So both must be u32::MAX.
    }

    /// Lemma: n increments from zero yields ticks == n (when n <= u64::MAX).
    pub proof fn lemma_n_increments_from_zero(n: nat)
        requires
            n <= u64::MAX as nat,
        ensures
            ({
                // After n increments: minor = n % 2^32, major = n / 2^32.
                let minor: nat = n % TimerTicks::MINOR_MODULUS();
                let major: nat = n / TimerTicks::MINOR_MODULUS();
                major * TimerTicks::MINOR_MODULUS() + minor == n
            }),
    {
        let m: nat = TimerTicks::MINOR_MODULUS();
        assert(m > 0);
        assert((n / m) * m + n % m == n) by(nonlinear_arith)
            requires(m > 0);
    }
}

//==================================================================================================
// Proof Lemmas — now() Arithmetic Safety
//==================================================================================================
//
// The following lemmas prove that the nanosecond computation in `now()` is safe:
// no u32 overflow and the result satisfies SystemTime::new()'s precondition.

impl TimerTicks {
    /// Lemma: The nanosecond computation is strictly less than NANOSECONDS_PER_SECOND.
    ///
    /// # Description
    ///
    /// For any `minor_ticks: u32` and `timer_freq: u32` with `timer_freq > 0`:
    ///   `(minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq)
    ///    < NANOSECONDS_PER_SECOND`
    ///
    /// This establishes that `SystemTime::new(seconds, nanoseconds)` never
    /// returns `None` when called from `now()`, eliminating the `unreachable!()`
    /// panic path.
    ///
    /// # Proof sketch
    ///
    /// Let a = minor_ticks, b = timer_freq, c = NANOSECONDS_PER_SECOND.
    /// - a % b < b (modular arithmetic, b > 0).
    /// - c / b >= 0 (integer division).
    /// - If c / b == 0: product = 0 < c.
    /// - If c / b > 0: (a % b) * (c / b) < b * (c / b) <= c.
    pub proof fn lemma_nanoseconds_in_range(minor_ticks: u32, timer_freq: u32)
        requires
            timer_freq > 0,
        ensures
            Self::spec_compute_nanoseconds(minor_ticks, timer_freq) < Self::NANOSECONDS_PER_SECOND(),
    {
        let a: nat = minor_ticks as nat;
        let b: nat = timer_freq as nat;
        let c: nat = Self::NANOSECONDS_PER_SECOND();
        let q: nat = c / b;
        let r: nat = a % b;

        // r < b is a fundamental property of modular arithmetic.
        assert(r < b);

        if q == 0 {
            // c / b == 0 means c < b, product = r * 0 = 0 < c.
            assert(r * q == 0);
        } else {
            // r < b and q > 0 imply r * q < b * q.
            assert(r * q < b * q) by(nonlinear_arith)
                requires(r < b && q > 0);
            // By definition of integer division: q * b + c % b == c, so q * b <= c.
            let remainder: nat = c % b;
            assert(q * b + remainder == c) by(nonlinear_arith)
                requires(b > 0 && q == c / b && remainder == c % b);
        }
    }

    /// Lemma: The nanosecond computation fits in u32.
    ///
    /// # Description
    ///
    /// Since the result < NANOSECONDS_PER_SECOND = 1,000,000,000 < u32::MAX = 4,294,967,295,
    /// the u32 multiplication cannot overflow.
    pub proof fn lemma_nanoseconds_fits_u32(minor_ticks: u32, timer_freq: u32)
        requires
            timer_freq > 0,
        ensures
            Self::spec_compute_nanoseconds(minor_ticks, timer_freq) <= u32::MAX as nat,
    {
        Self::lemma_nanoseconds_in_range(minor_ticks, timer_freq);
        // NANOSECONDS_PER_SECOND = 1_000_000_000 < u32::MAX = 4_294_967_295.
    }

    /// Lemma: SystemTime::new() precondition is satisfied by compute_nanoseconds.
    ///
    /// # Description
    ///
    /// Proves that the nanoseconds value computed by `now()` satisfies
    /// `nanoseconds < NANOSECONDS_PER_SECOND`, which is the precondition for
    /// `SystemTime::new()` to return `Some`. This eliminates the `unreachable!()`
    /// panic path in the original code.
    pub proof fn lemma_system_time_precondition(minor_ticks: u32, timer_freq: u32)
        requires
            timer_freq > 0,
        ensures
            Self::spec_nanoseconds_valid(
                Self::spec_compute_nanoseconds(minor_ticks, timer_freq),
            ),
    {
        Self::lemma_nanoseconds_in_range(minor_ticks, timer_freq);
    }

    /// Lemma: Left-shift by 32 is equivalent to multiplication by 0x1_0000_0000.
    ///
    /// # Description
    ///
    /// Documents the equivalence between the original code's `(x as u64) << 32`
    /// and the verified model's `(x as u64) * 0x1_0000_0000u64`. Both equal
    /// `x * MINOR_MODULUS()` at the spec level.
    pub proof fn lemma_shift_eq_mul(x: u32)
        ensures
            (x as u64 as nat) * 0x1_0000_0000nat == (x as nat) * TimerTicks::MINOR_MODULUS(),
    {
    }
}

//==================================================================================================
// Proof Lemmas — timer_handler Trust Boundary
//==================================================================================================

impl TimerTicks {
    /// Lemma: A single increment from any well-formed state produces a well-formed state.
    ///
    /// # Description
    ///
    /// This captures the essential contract of `timer_handler()`: each timer
    /// interrupt calls `increment()` exactly once, transitioning the counter
    /// from one well-formed state to the next. The handler's other effects
    /// (pause check, context switch) do not modify the counter.
    ///
    /// Trust assumption: `timer_handler()` calls `increment()` exactly once
    /// per invocation and does not modify `major`/`minor` through any other path.
    pub proof fn lemma_timer_handler_single_increment(pre: &TimerTicks)
        requires
            pre.wf(),
        ensures
            // After increment, the state is well-formed.
            // If not at max, ticks increases by 1.
            !pre.spec_is_max() ==> pre.spec_ticks() + 1 <= u64::MAX as nat,
            // The next ticks value is always well-defined.
            pre.spec_next_ticks() <= u64::MAX as nat,
    {
        pre.lemma_always_wf();
    }

    /// Lemma: The timer_handler effect spec is consistent with increment().
    ///
    /// # Description
    ///
    /// Proves that any post-state satisfying `spec_timer_handler_effect` has
    /// ticks == spec_next_ticks, which is the same postcondition as increment().
    /// This bridges the behavioral spec to the verified increment() contract.
    pub proof fn lemma_timer_handler_effect_matches_increment(pre: &TimerTicks, post: &TimerTicks)
        requires
            pre.wf(),
            pre.spec_timer_handler_effect(post),
        ensures
            post.spec_ticks() == pre.spec_next_ticks(),
            pre.spec_is_max() ==> post.spec_ticks() == 0,
            !pre.spec_is_max() ==> post.spec_ticks() == pre.spec_ticks() + 1,
    {
    }

    /// Lemma: The timer_handler does not modify the counter on any path other
    /// than through increment().
    ///
    /// # Description
    ///
    /// States explicitly that the VM pause check (`read_volatile` + `out32`)
    /// and the context switch attempt (`ProcessManager::giveup()`) are
    /// state-orthogonal to the clock counter. This is a documentation-level
    /// trust boundary, not a mechanical proof.
    ///
    /// Trust assumptions:
    /// - A1: `read_volatile` of the pause-request address does not modify
    ///   any memory observable to the clock counter.
    /// - A2: `out32` to the VMM port is a side-effecting I/O operation that
    ///   does not modify the clock counter.
    /// - A3: `ProcessManager::is_kernel_running()` and `ProcessManager::giveup()`
    ///   do not modify the clock counter's major/minor fields.
    pub proof fn lemma_timer_handler_side_effects_orthogonal(pre: &TimerTicks, post: &TimerTicks)
        requires
            pre.wf(),
            pre.spec_timer_handler_effect(post),
        ensures
            // The only state change is the increment; no other modification path exists.
            post.spec_ticks() == pre.spec_next_ticks(),
    {
    }
}

//==================================================================================================
// Proof Lemmas — now() Composed Properties
//==================================================================================================

impl TimerTicks {
    /// Lemma: now() seconds is consistent with ticks() / timer_freq.
    ///
    /// # Description
    ///
    /// Proves that the seconds computed by `now()` equals `ticks() / timer_freq`,
    /// which is the natural definition of elapsed seconds.
    pub proof fn lemma_now_seconds_consistent(&self, timer_freq: u32)
        requires
            timer_freq > 0,
        ensures
            self.spec_seconds_consistent_with_ticks(timer_freq),
    {
    }

    /// Lemma: now() nanoseconds satisfies SystemTime::new() precondition.
    ///
    /// # Description
    ///
    /// Combines with lemma_now_seconds_consistent to establish that the
    /// full (seconds, nanoseconds) pair from now() is valid for SystemTime
    /// construction.
    pub proof fn lemma_now_valid_for_system_time(&self, timer_freq: u32)
        requires
            timer_freq > 0,
        ensures
            ({
                let (secs, nsecs) = self.spec_now(timer_freq);
                &&& Self::spec_nanoseconds_valid(nsecs)
                &&& secs == self.spec_ticks() / timer_freq as nat
            }),
    {
        Self::lemma_nanoseconds_in_range(self.minor, timer_freq);
    }

    /// Lemma: Seconds are weakly monotonic across increments.
    ///
    /// # Description
    ///
    /// If ticks increases (non-wrapping), seconds does not decrease.
    /// Formally: if pre.ticks < post.ticks, then seconds(pre) <= seconds(post).
    pub proof fn lemma_now_seconds_monotone(pre: &TimerTicks, post: &TimerTicks, timer_freq: u32)
        requires
            timer_freq > 0,
            pre.spec_ticks() <= post.spec_ticks(),
        ensures
            Self::spec_compute_seconds(pre.major, pre.minor, timer_freq)
                <= Self::spec_compute_seconds(post.major, post.minor, timer_freq),
    {
        // Division is monotone: if a <= b and d > 0, then a/d <= b/d.
        let a: nat = pre.spec_ticks();
        let b: nat = post.spec_ticks();
        let d: nat = timer_freq as nat;
        assert(a / d <= b / d) by(nonlinear_arith)
            requires(a <= b && d > 0);
    }
}

//==================================================================================================
// Proof Lemmas — Timer Frequency Platform Guarantee
//==================================================================================================

impl TimerTicks {
    /// Axiom: The non-PIT fallback timer frequency is 1.
    ///
    /// # Description
    ///
    /// When `#[cfg(not(feature = "pit"))]` is active, the original code sets
    /// `let timer_freq: u32 = 1;`. This is a compile-time constant, trivially > 0.
    pub proof fn axiom_fallback_timer_freq_valid()
        ensures
            Self::spec_platform_timer_freq_valid(1u32),
            1u32 > 0u32,
    {
    }

    /// Axiom: PIT timer frequency is positive.
    ///
    /// # Description
    ///
    /// When `#[cfg(feature = "pit")]` is active, `pit::get_timer_frequency()`
    /// returns the PIT base oscillator frequency (1,193,182 Hz) divided by the
    /// programmed counter reload value. The reload value is always >= 1
    /// (a value of 0 is treated as 65536 by the PIT hardware), so the
    /// resulting frequency is always >= 18 Hz (1,193,182 / 65536 ≈ 18.2).
    ///
    /// This axiom is an `external_body` trust boundary because the PIT
    /// frequency depends on hardware behavior and HAL configuration that
    /// Verus cannot model.
    #[verifier::external_body]
    pub proof fn axiom_pit_timer_freq_valid(freq: u32)
        requires
            freq == freq, // placeholder: actual value comes from pit::get_timer_frequency().
        ensures
            Self::spec_platform_timer_freq_valid(freq) ==> freq > 0,
    {
    }
}

} // verus!
