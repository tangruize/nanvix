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
// Proof Lemmas — wrapping_add Equivalence
//==================================================================================================
//
// These lemmas prove that the branching model in `increment()` computes the
// same result as `wrapping_add(1)`, which is `(x + 1) % 2^32`.

impl TimerTicks {
    /// Lemma: Our branching model for minor increment is equivalent to wrapping_add(1).
    ///
    /// # Description
    ///
    /// The original code uses `minor.wrapping_add(1)` which computes
    /// `(minor + 1) % 2^32`. Our verified model uses:
    /// ```ignore
    /// if self.minor < u32::MAX { self.minor + 1 } else { 0 }
    /// ```
    /// This lemma proves both produce the same result for all u32 inputs.
    ///
    /// See Trust Boundary T2: this equivalence bridges the structural
    /// divergence between the original `wrapping_add` and the verified
    /// explicit branch.
    pub proof fn lemma_wrapping_add_equiv(x: u32)
        ensures
            (if x < u32::MAX { (x + 1) as nat } else { 0nat })
                == Self::spec_wrapping_add_one(x),
    {
        let m: nat = Self::MINOR_MODULUS();
        assert(m == u32::MAX as nat + 1);
        if x < u32::MAX {
            // x + 1 < 2^32, so (x + 1) % 2^32 == x + 1.
            assert((x as nat + 1) < m);
            assert((x as nat + 1) % m == x as nat + 1) by(nonlinear_arith)
                requires(x as nat + 1 < m && m > 0);
        } else {
            // x == u32::MAX, so x + 1 == 2^32, and 2^32 % 2^32 == 0.
            assert(x as nat + 1 == m);
            assert(m % m == 0nat) by(nonlinear_arith)
                requires(m > 0);
        }
    }

    /// Lemma: Major wrapping is also equivalent to wrapping_add(1).
    ///
    /// # Description
    ///
    /// The original code uses `major.wrapping_add(1)` for the major counter.
    /// Our model uses `if major < u32::MAX { major + 1 } else { 0 }`.
    /// This lemma proves equivalence for the major counter as well.
    pub proof fn lemma_wrapping_add_equiv_major(x: u32)
        ensures
            (if x < u32::MAX { (x + 1) as nat } else { 0nat })
                == Self::spec_wrapping_add_one(x),
    {
        Self::lemma_wrapping_add_equiv(x);
    }
}

//==================================================================================================
// Proof Lemmas — Torn Read Consequence
//==================================================================================================

impl TimerTicks {
    /// Lemma: Demonstrates the consequence of a torn read in `get()`.
    ///
    /// # Description
    ///
    /// If the no-concurrent-writer assumption (Trust Boundary T1) is violated,
    /// a torn read can occur. This lemma models the **reverse-reordering** case
    /// where the reader observes the *new* major before the *old* minor. This
    /// could theoretically occur under weak memory models with load reordering,
    /// but **not on x86** (x86-TSO guarantees load-load order). See
    /// `lemma_torn_read_consequence_x86` for the x86-realistic scenario.
    ///
    /// **Scenario**: The actual state is `(major=M, minor=0xFFFFFFFF)`.
    /// An increment occurs between the two loads, changing the state to
    /// `(major=M+1, minor=0)`. The reader sees `(major=M+1, minor=0xFFFFFFFF)`.
    ///
    /// **Consequence**: The observed tick count is `(M+1) * 2^32 + 0xFFFFFFFF`,
    /// which is `2^32` (`MINOR_MODULUS`) ticks ahead of the actual pre-increment
    /// state `M * 2^32 + 0xFFFFFFFF`.
    ///
    /// This lemma is not used in any postcondition — it is a documentation
    /// proof that makes the torn-read risk concrete and quantifiable.
    pub proof fn lemma_torn_read_consequence(major: u32)
        requires
            major < u32::MAX,
        ensures
            ({
                // Actual state before increment.
                let actual = TimerTicks { major: major, minor: u32::MAX };
                // Torn read: reader sees new major, old minor.
                let torn = TimerTicks { major: (major + 1) as u32, minor: u32::MAX };
                // The torn read is exactly MINOR_MODULUS ticks ahead.
                torn.spec_ticks() == actual.spec_ticks() + TimerTicks::MINOR_MODULUS()
            }),
    {
        let m: nat = Self::MINOR_MODULUS();
        let old_major: nat = major as nat;
        let new_major: nat = (major + 1) as nat;
        assert(new_major == old_major + 1);
        assert(new_major * m == old_major * m + m) by(nonlinear_arith)
            requires(new_major == old_major + 1);
    }

    /// Lemma: x86-realistic torn read — reader sees old major, new minor.
    ///
    /// # Description
    ///
    /// On x86 (Nanvix's only target), loads are ordered (x86-TSO): a load of
    /// `major` followed by a load of `minor` cannot observe the minor from a
    /// *later* store than the major's. The only torn-read scenario is:
    ///
    /// 1. Reader loads `major` → gets `M` (pre-increment value).
    /// 2. Timer interrupt fires: state goes from `(M, 0xFFFFFFFF)` to `(M+1, 0)`.
    /// 3. Reader loads `minor` → gets `0` (post-increment value).
    /// 4. Reader observes `(M, 0)`.
    ///
    /// **Consequence**: The observed tick count is `M * 2^32 + 0`, which is
    /// `MINOR_MODULUS` (`2^32`) ticks *behind* the actual post-increment
    /// state `(M+1) * 2^32 + 0`. Equivalently, it is `u32::MAX` ticks
    /// behind the pre-increment state `M * 2^32 + 0xFFFFFFFF`.
    ///
    /// This is the realistic torn-read hazard for Nanvix's x86 target.
    pub proof fn lemma_torn_read_consequence_x86(major: u32)
        requires
            major < u32::MAX,
        ensures
            ({
                // Actual state after increment (what the handler just wrote).
                let actual_post = TimerTicks { major: (major + 1) as u32, minor: 0u32 };
                // Torn read: reader sees old major, new minor.
                let torn = TimerTicks { major: major, minor: 0u32 };
                // The torn read is exactly MINOR_MODULUS ticks behind post-increment.
                actual_post.spec_ticks() == torn.spec_ticks() + TimerTicks::MINOR_MODULUS()
            }),
            ({
                // Alternatively: torn read is u32::MAX ticks behind pre-increment.
                let actual_pre = TimerTicks { major: major, minor: u32::MAX };
                let torn = TimerTicks { major: major, minor: 0u32 };
                actual_pre.spec_ticks() == torn.spec_ticks() + u32::MAX as nat
            }),
    {
        let m: nat = Self::MINOR_MODULUS();
        let old_major: nat = major as nat;
        let new_major: nat = (major + 1) as nat;
        assert(new_major == old_major + 1);
        // Post-increment ticks = (M+1) * M + 0, torn ticks = M * M + 0.
        assert(new_major * m == old_major * m + m) by(nonlinear_arith)
            requires(new_major == old_major + 1);
        // Pre-increment ticks = M * M + 0xFFFFFFFF, torn ticks = M * M + 0.
        assert(m == u32::MAX as nat + 1);
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

    /// Lemma: SystemTime::new() succeeds for all outputs of now().
    ///
    /// # Description
    ///
    /// Proves that the (seconds, nanoseconds) pair produced by `now()` always
    /// satisfies `spec_system_time_new_succeeds`, i.e., `nanoseconds <
    /// NANOSECONDS_PER_SECOND`. This means `SystemTime::new(seconds, nanoseconds)`
    /// returns `Some(...)`, and the `unreachable!()` in the original code is
    /// truly unreachable.
    pub proof fn lemma_system_time_new_succeeds(&self, timer_freq: u32)
        requires
            timer_freq > 0,
        ensures
            Self::spec_system_time_new_succeeds(
                Self::spec_compute_nanoseconds(self.minor, timer_freq),
            ),
    {
        Self::lemma_nanoseconds_in_range(self.minor, timer_freq);
        Self::lemma_nanoseconds_fits_u32(self.minor, timer_freq);
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
    /// Verus cannot model. It returns a ghost `u32` value representing the
    /// PIT frequency, with the postcondition that this value is positive.
    ///
    /// Unlike a parameterized axiom, this form cannot be misused: the
    /// caller receives a value satisfying `freq > 0` but cannot choose
    /// which value it is.
    ///
    /// **Lower-bound guarantee only:** This axiom guarantees positivity
    /// (`freq > 0`) but does not constrain the returned value to equal the
    /// actual PIT frequency. If a future proof needs to reason about the
    /// specific frequency value (e.g., to bound time resolution), a
    /// stronger axiom binding the return value to `pit::get_timer_frequency()`
    /// would be required.
    ///
    /// This models the HAL call `pit::get_timer_frequency()` from the
    /// original `now()`. The link is:
    /// ```ignore
    /// #[cfg(feature = "pit")]
    /// let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
    /// ```
    /// The axiom's postcondition (`freq > 0`) is the minimum guarantee
    /// needed by `compute_nanoseconds` and `compute_seconds`.
    #[verifier::external_body]
    pub proof fn axiom_pit_timer_freq_valid() -> (freq: u32)
        ensures
            freq > 0,
    {
        unimplemented!()
    }
}

//==================================================================================================
// Proof Lemmas — Snapshot Consistency Axiom
//==================================================================================================

impl TimerTicks {
    /// Axiom: The no-concurrent-writer assumption holds.
    ///
    /// # Description
    ///
    /// This `external_body` axiom introduces
    /// `spec_no_concurrent_writer_assumption()` into the proof environment.
    /// It models the system-level guarantee that:
    ///
    /// - The timer interrupt handler is the only writer to `major`/`minor`.
    /// - Callers of `get()` run with interrupts disabled (or on the same
    ///   core as the handler), preventing torn reads.
    ///
    /// Code paths that depend on snapshot consistency must either:
    /// 1. Invoke this axiom directly, or
    /// 2. Receive the predicate from `get()`'s postcondition.
    ///
    /// This is Trust Boundary T1. The assumption cannot be proved within
    /// the clock module because it depends on the interrupt controller
    /// configuration and the kernel's scheduling discipline.
    #[verifier::external_body]
    pub proof fn axiom_no_concurrent_writer()
        ensures
            Self::spec_no_concurrent_writer_assumption(),
    {
    }
}

} // verus!
