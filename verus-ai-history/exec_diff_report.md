# Verus Exec Code Diff Report

This report compares executable code between original Nanvix source
files and their Verus-verified counterparts.

## Summary

| Severity | Count |
|----------|-------|
| Critical | 66 |
| High     | 119 |
| Medium   | 115 |
| Low      | 1 |

## Module: capability

- **Source:** `src/kernel/src/pm/process/capability.rs`
- **Verus:** `verus/split/kernel/pm/process/capability.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 4 critical, 0 high, 2 medium. Structs: 1 diffs. Functions: 6 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `Capabilities` | added_field | **critical** | New exec field added: pub bits: u8 |

### Function Differences

#### 🟢 `to_mask` — added_in_verus (low)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `new` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `default` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🔴 `set` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,5 @@
 {
-        self.0 |= 1 << capability as u8;
+        let mask: u8 = Self::to_mask(capability);
+        self.bits = self.bits | mask;
+        }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.0 |= 1 << capability as u8;
    }
```

**Verus:**
```rust
{
        let mask: u8 = Self::to_mask(capability);
        let ghost pre = *self;
        self.bits = self.bits | mask;

        proof {
            Capabilities::lemma_set_then_has(pre, capability);
            if pre.wf() {
                Capabilities::lemma_set_preserves_wf(pre, capability);
            }
        }
    }
```
</details>

#### 🔴 `clear` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,5 @@
 {
-        self.0 &= !(1 << capability as u8);
+        let mask: u8 = Self::to_mask(capability);
+        self.bits = self.bits & !mask;
+        }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.0 &= !(1 << capability as u8);
    }
```

**Verus:**
```rust
{
        let mask: u8 = Self::to_mask(capability);
        let ghost pre = *self;
        self.bits = self.bits & !mask;

        proof {
            Capabilities::lemma_clear_then_not_has(pre, capability);
            if pre.wf() {
                Capabilities::lemma_clear_preserves_wf(pre, capability);
            }
        }
    }
```
</details>

#### 🔴 `has` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,4 @@
 {
-        (self.0 & (1 << capability as u8)) != 0
+        let mask: u8 = Self::to_mask(capability);
+        (self.bits & mask) != 0u8
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        (self.0 & (1 << capability as u8)) != 0
    }
```

**Verus:**
```rust
{
        let mask: u8 = Self::to_mask(capability);
        (self.bits & mask) != 0u8
    }
```
</details>

## Module: clock

- **Source:** `src/kernel/src/pm/clock.rs`
- **Verus:** `verus/split/kernel/pm/clock.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 5 critical, 7 high, 11 medium. Structs: 4 diffs. Functions: 19 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `TimerTicks` | visibility_changed | **medium** | Field 'minor' changed from private to pub (Verus constraint). |
| `TimerTicks` | type_changed | **high** | Field 'minor' type changed: 'AtomicU32' → 'u32'. |
| `TimerTicks` | visibility_changed | **medium** | Field 'major' changed from private to pub (Verus constraint). |
| `TimerTicks` | type_changed | **high** | Field 'major' type changed: 'AtomicU32' → 'u32'. |

### Function Differences

#### 🟠 `timer_handler` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟡 `is_max` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `is_zero` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `compute_nanoseconds` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `compute_seconds` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `timer_handler_model` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `standalone_ticks` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `standalone_now` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `now_fallback_model` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `now_pit_model` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,6 +1,3 @@
 {
-        Self {
-            minor: AtomicU32::new(0),
-            major: AtomicU32::new(0),
-        }
+        TimerTicks { minor: 0, major: 0 }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        Self {
            minor: AtomicU32::new(0),
            major: AtomicU32::new(0),
        }
    }
```

**Verus:**
```rust
{
        TimerTicks { minor: 0, major: 0 }
    }
```
</details>

#### 🔴 `get` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,3 @@
 {
-        (self.major.load(ORDER), self.minor.load(ORDER))
+        (self.major, self.minor)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        (self.major.load(ORDER), self.minor.load(ORDER))
    }
```

**Verus:**
```rust
{
        (self.major, self.minor)
    }
```
</details>

#### 🟠 `increment` — signature_changed (high)

Parameters differ: source='(&self)' vs verus='(&mut self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn increment(&self) -> u32 {
        // Safely increment `TIMER_TICKS`, assuming single-writer.
        let minor: u32 = self.minor.load(ORDER);
        let new_minor: u32 = minor.wrapping_add(1);
        self.minor.store(new_minor, ORDER);

        // Check if the minor tick overflowed.
        if new_minor == 0 {
            // Safely increment `TIMER_TICKS_MAJOR`, assuming single-writer.
            let major: u32 = self.major.load(ORDER);
            self.major.store(major.wrapping_add(1), ORDER);
        }

        new_minor
    }
```

**Verus:**
```rust
pub fn increment(&mut self) -> (result: u32)
        requires
            // Note: wf() is universally true for any (u32, u32) pair (see
            // lemma_always_wf). This precondition is retained for documentation
            // and forward-compatibility if wf() is ever strengthened.
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
                // self.major unchanged, self.minor = old(self).minor + 1.
                assert(self.spec_major() == old(self).spec_major());
                assert(self.spec_minor() == old(self).spec_minor() + 1);
                // spec_ticks = major * M + minor = old.major * M + (old.minor + 1).
                assert(self.spec_ticks() == old(self).spec_major() * Self::MINOR_MODULUS() + old(self).spec_minor() + 1);
                self.lemma_always_wf();
            }
        } else {
            self.minor = 0;
            if self.major < u32::MAX {
                self.major = self.major + 1;
                proof {
                    assert(Self::MINOR_MODULUS() == u32::MAX as nat + 1);
                    assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
                    assert(old(self).spec_major(
```
</details>

#### 🔴 `increment` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,15 +1,13 @@
 {
-        // Safely increment `TIMER_TICKS`, assuming single-writer.
-        let minor: u32 = self.minor.load(ORDER);
-        let new_minor: u32 = minor.wrapping_add(1);
-        self.minor.store(new_minor, ORDER);
-
-        // Check if the minor tick overflowed.
-        if new_minor == 0 {
-            // Safely increment `TIMER_TICKS_MAJOR`, assuming single-writer.
-            let major: u32 = self.major.load(ORDER);
-            self.major.store(major.wrapping_add(1), ORDER);
+        if self.minor < u32::MAX {
+            self.minor = self.minor + 1;
+        } else {
+            self.minor = 0;
+            if self.major < u32::MAX {
+                self.major = self.major + 1;
+            } else {
+                self.major = 0;
+            }
         }
-
-        new_minor
+        self.minor
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        // Safely increment `TIMER_TICKS`, assuming single-writer.
        let minor: u32 = self.minor.load(ORDER);
        let new_minor: u32 = minor.wrapping_add(1);
        self.minor.store(new_minor, ORDER);

        // Check if the minor tick overflowed.
        if new_minor == 0 {
            // Safely increment `TIMER_TICKS_MAJOR`, assuming single-writer.
            let major: u32 = self.major.load(ORDER);
            self.major.store(major.wrapping_add(1), ORDER);
        }

        new_minor
    }
```

**Verus:**
```rust
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
                // self.major unchanged, self.minor = old(self).minor + 1.
                assert(self.spec_major() == old(self).spec_major());
                assert(self.spec_minor() == old(self).spec_minor() + 1);
                // spec_ticks = major * M + minor = old.major * M + (old.minor + 1).
                assert(self.spec_ticks() == old(self).spec_major() * Self::MINOR_MODULUS() + old(self).spec_minor() + 1);
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
                    // self.major = old.major + 1, self.minor = 0.
                    let om: nat = old(self).spec_major();
                    let m: nat = Self::MINOR_MODULUS();
                    assert((om + 1) * m == om * m + m) by(nonlinear_arith);
                    self.lemma_always_wf(
```
</details>

#### 🟠 `ticks` — signature_changed (high)

Parameters differ: source='()' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn ticks() -> u64 {
    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
    ((major_ticks as u64) << 32) + (minor_ticks as u64)
}
```

**Verus:**
```rust
pub fn ticks(&self) -> (result: u64)
        ensures
            result as nat == self.spec_ticks(),
    {
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        (self.major as u64) * 0x1_0000_0000u64 + (self.minor as u64)
    }
```
</details>

#### 🔴 `ticks` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,4 +1,3 @@
 {
-    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
-    ((major_ticks as u64) << 32) + (minor_ticks as u64)
-}
+        (self.major as u64) * 0x1_0000_0000u64 + (self.minor as u64)
+    }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
    ((major_ticks as u64) << 32) + (minor_ticks as u64)
}
```

**Verus:**
```rust
{
        proof {
            assert(u32::MAX as nat * Self::MINOR_MODULUS() + u32::MAX as nat == u64::MAX as nat);
        }
        (self.major as u64) * 0x1_0000_0000u64 + (self.minor as u64)
    }
```
</details>

#### 🟠 `now` — signature_changed (high)

Parameters differ: source='()' vs verus='(&self, timer_freq: u32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn now() -> SystemTime {
    #[cfg(feature = "pit")]
    let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
    #[cfg(not(feature = "pit"))]
    let timer_freq: u32 = 1;

    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
    let seconds: u64 = (((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64);
    let nanoseconds: u32 = (minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq);

    match SystemTime::new(seconds, nanoseconds) {
        Some(time) => time,
        None => {
            // SAFETY: This should not happen because `ticks` should be always in a valid range of `SystemTime`.
            unreachable!(
                "now(): failed to get system time (major_ticks={major_ticks:?}, \
                 minor_ticks={minor_ticks:?}, timer_freq={timer_freq:?})"
            )
        },
    }
}
```

**Verus:**
```rust
pub fn now(&self, timer_freq: u32) -> (result: (u64, u32))
        requires
            timer_freq > 0,
            Self::spec_no_concurrent_writer_assumption(),
        ensures
            result.0 as nat == Self::spec_compute_seconds(self.major, self.minor, timer_freq),
            result.1 as nat == Self::spec_compute_nanoseconds(self.minor, timer_freq),
            Self::spec_nanoseconds_valid(result.1 as nat),
            result.1 < 1_000_000_000u32,
            result.0 as nat == self.spec_ticks() / timer_freq as nat,
            self.spec_now(timer_freq) == (result.0 as nat, result.1 as nat),
    {
        let (major_ticks, minor_ticks): (u32, u32) = self.get();
        let seconds: u64 = Self::compute_seconds(major_ticks, minor_ticks, timer_freq);
        let nanoseconds: u32 = Self::compute_nanoseconds(minor_ticks, timer_freq);
        (seconds, nanoseconds)
    }
```
</details>

#### 🟠 `now` — return_type_changed (high)

Return type differs: source='SystemTime' vs verus='(u64, u32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn now() -> SystemTime {
    #[cfg(feature = "pit")]
    let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
    #[cfg(not(feature = "pit"))]
    let timer_freq: u32 = 1;

    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
    let seconds: u64 = (((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64);
    let nanoseconds: u32 = (minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq);

    match SystemTime::new(seconds, nanoseconds) {
        Some(time) => time,
        None => {
            // SAFETY: This should not happen because `ticks` should be always in a valid range of `SystemTime`.
            unreachable!(
                "now(): failed to get system time (major_ticks={major_ticks:?}, \
                 minor_ticks={minor_ticks:?}, timer_freq={timer_freq:?})"
            )
        },
    }
}
```

**Verus:**
```rust
pub fn now(&self, timer_freq: u32) -> (result: (u64, u32))
        requires
            timer_freq > 0,
            Self::spec_no_concurrent_writer_assumption(),
        ensures
            result.0 as nat == Self::spec_compute_seconds(self.major, self.minor, timer_freq),
            result.1 as nat == Self::spec_compute_nanoseconds(self.minor, timer_freq),
            Self::spec_nanoseconds_valid(result.1 as nat),
            result.1 < 1_000_000_000u32,
            result.0 as nat == self.spec_ticks() / timer_freq as nat,
            self.spec_now(timer_freq) == (result.0 as nat, result.1 as nat),
    {
        let (major_ticks, minor_ticks): (u32, u32) = self.get();
        let seconds: u64 = Self::compute_seconds(major_ticks, minor_ticks, timer_freq);
        let nanoseconds: u32 = Self::compute_nanoseconds(minor_ticks, timer_freq);
        (seconds, nanoseconds)
    }
```
</details>

#### 🔴 `now` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,21 +1,6 @@
 {
-    #[cfg(feature = "pit")]
-    let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
-    #[cfg(not(feature = "pit"))]
-    let timer_freq: u32 = 1;
-
-    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
-    let seconds: u64 = (((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64);
-    let nanoseconds: u32 = (minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq);
-
-    match SystemTime::new(seconds, nanoseconds) {
-        Some(time) => time,
-        None => {
-            // SAFETY: This should not happen because `ticks` should be always in a valid range of `SystemTime`.
-            unreachable!(
-                "now(): failed to get system time (major_ticks={major_ticks:?}, \
-                 minor_ticks={minor_ticks:?}, timer_freq={timer_freq:?})"
-            )
-        },
+        let (major_ticks, minor_ticks): (u32, u32) = self.get();
+        let seconds: u64 = Self::compute_seconds(major_ticks, minor_ticks, timer_freq);
+        let nanoseconds: u32 = Self::compute_nanoseconds(minor_ticks, timer_freq);
+        (seconds, nanoseconds)
     }
-}

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
    #[cfg(feature = "pit")]
    let timer_freq: u32 = crate::hal::platform::pit::get_timer_frequency();
    #[cfg(not(feature = "pit"))]
    let timer_freq: u32 = 1;

    let (major_ticks, minor_ticks): (u32, u32) = TIMER_TICKS.get();
    let seconds: u64 = (((major_ticks as u64) << 32) + (minor_ticks as u64)) / (timer_freq as u64);
    let nanoseconds: u32 = (minor_ticks % timer_freq) * (NANOSECONDS_PER_SECOND / timer_freq);

    match SystemTime::new(seconds, nanoseconds) {
        Some(time) => time,
        None => {
            // SAFETY: This should not happen because `ticks` should be always in a valid range of `SystemTime`.
            unreachable!(
                "now(): failed to get system time (major_ticks={major_ticks:?}, \
                 minor_ticks={minor_ticks:?}, timer_freq={timer_freq:?})"
            )
        },
    }
}
```

**Verus:**
```rust
{
        let (major_ticks, minor_ticks): (u32, u32) = self.get();
        let seconds: u64 = Self::compute_seconds(major_ticks, minor_ticks, timer_freq);
        let nanoseconds: u32 = Self::compute_nanoseconds(minor_ticks, timer_freq);
        (seconds, nanoseconds)
    }
```
</details>

## Module: process_manager

- **Source:** `src/kernel/src/pm/process/manager/mod.rs`
- **Verus:** `verus/split/kernel/pm/process/manager/process_manager.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 29 critical, 79 high, 74 medium. Structs: 15 diffs. Functions: 167 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `PidSet` | added_struct | **high** | New struct added in verus that doesn't exist in source. |
| `ProcessManager` | removed_struct | **critical** | Struct exists in source but missing in verus. |
| `ProcessManagerInner` | added_field | **critical** | New exec field added: pub running_pid: i32 |
| `ProcessManagerInner` | added_field | **critical** | New exec field added: pub ready_count: usize |
| `ProcessManagerInner` | added_field | **critical** | New exec field added: pub suspended_count: usize |
| `ProcessManagerInner` | added_field | **critical** | New exec field added: pub interrupted_count: usize |
| `ProcessManagerInner` | added_field | **critical** | New exec field added: pub zombie_count: usize |
| `ProcessManagerInner` | ghost_field_added | **high** | Ghost field added to exec struct: pub ghost_ready: PidSet |
| `ProcessManagerInner` | ghost_field_added | **high** | Ghost field added to exec struct: pub ghost_suspended: PidSet |
| `ProcessManagerInner` | ghost_field_added | **high** | Ghost field added to exec struct: pub ghost_interrupted: PidSet |
| `ProcessManagerInner` | ghost_field_added | **high** | Ghost field added to exec struct: pub ghost_zombies: PidSet |
| `ProcessManagerInner` | visibility_changed | **medium** | Field 'interrupt_capable' changed from private to pub (Verus constraint). |
| `ProcessManagerInner` | visibility_changed | **medium** | Field 'next_pid' changed from private to pub (Verus constraint). |
| `ProcessManagerInner` | type_changed | **high** | Field 'next_pid' type changed: 'ProcessIdentifier' → 'i32'. |
| `ProcessManagerInner` | visibility_changed | **medium** | Field 'number_buffered_messages' changed from private to pub (Verus constraint). |

### Function Differences

#### 🟠 `create_thread` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `try_add_thread` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `check_alarm` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `sleep` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `wakeup` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `try_wakeup` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `exit` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `exit_thread` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `terminate` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `interrupt_reason` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `harvest_zombies` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `get_pid` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `get_tid` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `has_capability` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `vmcopy_from_user` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `vmcopy_to_user` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `mmap` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `munmap` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `mctrl` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `mmio_alloc` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `mmio_free` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `attach_pmio` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `detach_pmio` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `read_pmio` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `write_pmio` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `add_event` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `remove_event` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `number_buffered_messages` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `try_borrow` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `try_borrow_mut` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟡 `empty` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `pid_insert` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `pid_remove` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `find_index` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `pid_clear` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `absorb` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `get_running_pid` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `has_ready` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `has_zombies` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `is_interrupt_capable` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `get_buffered_message_count` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `sleep_running` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `sleep_thread_running` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `exit_running` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `exit_thread_running` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `exit_thread_to_suspended` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `exit_thread_to_zombie` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `wakeup_to_ready` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `resume_all_interrupted` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `terminate_ready` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `terminate_ready_stays_ready` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `terminate_suspended` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `harvest_zombie` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `post_message_not_found` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `recv_message` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `capctl_error_noop` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `alarm_interrupt` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `full_schedule` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `wakeup_running_noop` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `wakeup_ready_noop` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `wakeup_not_found` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `wakeup_suspended_failed_noop` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `take_interrupt_reason` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `create_thread_in_ready` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `create_thread_from_suspended` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `sleep_dispatch` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `exit_dispatch` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `exit_thread_dispatch` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `check_alarm_wrapper` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `harvest_zombies_wrapper` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `wakeup_dispatch` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `create_thread_dispatch` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `inner_create_thread` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `inner_create_thread_error` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `inner_try_add_thread` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `inner_wakeup` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `inner_try_wakeup` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `inner_try_wakeup_noop` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_try_borrow` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_try_borrow_mut` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_get_pid` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_get_tid` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_has_capability` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_terminate_ready` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_terminate_error` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_vmcopy_from_user` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_vmcopy_to_user` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_mmap` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_munmap` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_mctrl` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_mmio_alloc` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_mmio_free` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_attach_pmio` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_detach_pmio` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_read_pmio` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_write_pmio` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_add_event` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_remove_event` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_post_message` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_post_message_not_found` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `outer_number_buffered_messages` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟠 `new` — signature_changed (high)

Parameters differ: source='(
        interrupt_capable: bool,
        kernel: ReadyThread,
        root: Vmem,
        tm: ThreadManager,
    )' vs verus='(interrupt_capable: bool)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn new(
        interrupt_capable: bool,
        kernel: ReadyThread,
        root: Vmem,
        tm: ThreadManager,
    ) -> Self {
        let kernel: RunnableProcess = RunnableProcess::new(ProcessIdentifier::KERNEL, kernel, root);

        let (kernel, reason, _, _user_tda): (
            RunningProcess,
            Option<InterruptReason>,
            *mut ContextInformation,
            Option<VirtualAddress>,
        ) = kernel.run();
        debug_assert!(reason.is_none(), "kernel process should not be interrupted");

        Self {
            interrupt_capable,
            interrupt_reason: None,
            next_pid: ProcessIdentifier::from(1),
            ready: LinkedList::new(),
            suspended: LinkedList::new(),
            interrupted: LinkedList::new(),
            zombies: LinkedList::new(),
            running: Some(kernel),
            tm,
            number_buffered_messages: 0,
        }
    }
```

**Verus:**
```rust
pub fn new(interrupt_capable: bool) -> (result: Self)
        ensures
            result.wf(),
            result.spec_running_pid() == 0,
            result.ready_count == 0,
            result.suspended_count == 0,
            result.interrupted_count == 0,
            result.zombie_count == 0,
            result.next_pid == 1i32,
            result.interrupt_capable == interrupt_capable,
            result.number_buffered_messages == 0,
    {
        ProcessManagerInner {
            running_pid: 0i32,
            ready_count: 0usize,
            suspended_count: 0usize,
            interrupted_count: 0usize,
            zombie_count: 0usize,
            next_pid: 1i32,
            interrupt_capable,
            number_buffered_messages: 0usize,
            ghost_ready: PidSet::empty(),
            ghost_suspended: PidSet::empty(),
            ghost_interrupted: PidSet::empty(),
            ghost_zombies: PidSet::empty(),
        }
    }
```
</details>

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,24 +1,16 @@
 {
-        let kernel: RunnableProcess = RunnableProcess::new(ProcessIdentifier::KERNEL, kernel, root);
-
-        let (kernel, reason, _, _user_tda): (
-            RunningProcess,
-            Option<InterruptReason>,
-            *mut ContextInformation,
-            Option<VirtualAddress>,
-        ) = kernel.run();
-        debug_assert!(reason.is_none(), "kernel process should not be interrupted");
-
-        Self {
+        ProcessManagerInner {
+            running_pid: 0i32,
+            ready_count: 0usize,
+            suspended_count: 0usize,
+            interrupted_count: 0usize,
+            zombie_count: 0usize,
+            next_pid: 1i32,
             interrupt_capable,
-            interrupt_reason: None,
-            next_pid: ProcessIdentifier::from(1),
-            ready: LinkedList::new(),
-            suspended: LinkedList::new(),
-            interrupted: LinkedList::new(),
-            zombies: LinkedList::new(),
-            running: Some(kernel),
-            tm,
-            number_buffered_messages: 0,
+            number_buffered_messages: 0usize,
+            ghost_ready: PidSet::empty(),
+            ghost_suspended: PidSet::empty(),
+            ghost_interrupted: PidSet::empty(),
+            ghost_zombies: PidSet::empty(),
         }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        let kernel: RunnableProcess = RunnableProcess::new(ProcessIdentifier::KERNEL, kernel, root);

        let (kernel, reason, _, _user_tda): (
            RunningProcess,
            Option<InterruptReason>,
            *mut ContextInformation,
            Option<VirtualAddress>,
        ) = kernel.run();
        debug_assert!(reason.is_none(), "kernel process should not be interrupted");

        Self {
            interrupt_capable,
            interrupt_reason: None,
            next_pid: ProcessIdentifier::from(1),
            ready: LinkedList::new(),
            suspended: LinkedList::new(),
            interrupted: LinkedList::new(),
            zombies: LinkedList::new(),
            running: Some(kernel),
            tm,
            number_buffered_messages: 0,
        }
    }
```

**Verus:**
```rust
{
        ProcessManagerInner {
            running_pid: 0i32,
            ready_count: 0usize,
            suspended_count: 0usize,
            interrupted_count: 0usize,
            zombie_count: 0usize,
            next_pid: 1i32,
            interrupt_capable,
            number_buffered_messages: 0usize,
            ghost_ready: PidSet::empty(),
            ghost_suspended: PidSet::empty(),
            ghost_interrupted: PidSet::empty(),
            ghost_zombies: PidSet::empty(),
        }
    }
```
</details>

#### 🟠 `forge_user_context` — signature_changed (high)

Parameters differ: source='(
        mm: &mut VirtMemoryManager,
        vmem: &mut Vmem,
        args: &ThreadCreateArgs,
        enable_interrupts: bool,
    )' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn forge_user_context(
        mm: &mut VirtMemoryManager,
        vmem: &mut Vmem,
        args: &ThreadCreateArgs,
        enable_interrupts: bool,
    ) -> Result<(KernelStack, ContextInformation), Error> {
        trace!("args={args:?}, enable_interrupts={enable_interrupts:?}",);

        unsafe extern "C" {
            pub fn __leave_kernel_to_user_mode();
        }

        // Assert pre-conditions (these should have been checked by the caller).
        debug_assert!(Vmem::is_user_region(args.user_stack_base, args.user_stack_size));
        debug_assert!(Vmem::is_user_addr(args.user_fn));

        let kernel_func: VirtualAddress =
            VirtualAddress::from_raw_value(__leave_kernel_to_user_mode as usize);

        // Alloc kernel pages for the kernel stack. If we fail beyond this point, `kernel_stack`
        // gets dropped as soon as we exit this scope and underlying pages are released.
        let kernel_stack: KernelStack = KernelStack::new(mm)?;

        let cr3: u32 = vmem.pgdir().physical_address()?.into_raw_value() as u32;
        let esp: u32 = unsafe {
            hal::arch::forge_user_stack(
                kernel_stack.top().into_raw_value() as *mut u8,
                args.user_stack_base.into_raw_value() + args.user_stack_size,
                args.user_fn.into_raw_value(),
                args.user_fn_arg0,
                args.user_fn_arg1,
                kernel_func.into_raw_value(),
                enable_interrupts,
            )
        } as u32;
        let esp0: u32 = kernel_stack.top().into_raw_value() as u32;

        trace!("cr3={:#x}, esp={:#x}, ebp={:#x}", cr3, esp, esp0);
        let context: ContextInformation = ContextInformation::new(cr3, esp, esp0);

        Ok((kernel_stack, context))
    }
```

**Verus:**
```rust
pub fn forge_user_context(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
        // Context setup: no queue-level change.
    }
```
</details>

#### 🟠 `forge_user_context` — return_type_changed (high)

Return type differs: source='Result<(KernelStack, ContextInformation), Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn forge_user_context(
        mm: &mut VirtMemoryManager,
        vmem: &mut Vmem,
        args: &ThreadCreateArgs,
        enable_interrupts: bool,
    ) -> Result<(KernelStack, ContextInformation), Error> {
        trace!("args={args:?}, enable_interrupts={enable_interrupts:?}",);

        unsafe extern "C" {
            pub fn __leave_kernel_to_user_mode();
        }

        // Assert pre-conditions (these should have been checked by the caller).
        debug_assert!(Vmem::is_user_region(args.user_stack_base, args.user_stack_size));
        debug_assert!(Vmem::is_user_addr(args.user_fn));

        let kernel_func: VirtualAddress =
            VirtualAddress::from_raw_value(__leave_kernel_to_user_mode as usize);

        // Alloc kernel pages for the kernel stack. If we fail beyond this point, `kernel_stack`
        // gets dropped as soon as we exit this scope and underlying pages are released.
        let kernel_stack: KernelStack = KernelStack::new(mm)?;

        let cr3: u32 = vmem.pgdir().physical_address()?.into_raw_value() as u32;
        let esp: u32 = unsafe {
            hal::arch::forge_user_stack(
                kernel_stack.top().into_raw_value() as *mut u8,
                args.user_stack_base.into_raw_value() + args.user_stack_size,
                args.user_fn.into_raw_value(),
                args.user_fn_arg0,
                args.user_fn_arg1,
                kernel_func.into_raw_value(),
                enable_interrupts,
            )
        } as u32;
        let esp0: u32 = kernel_stack.top().into_raw_value() as u32;

        trace!("cr3={:#x}, esp={:#x}, ebp={:#x}", cr3, esp, esp0);
        let context: ContextInformation = ContextInformation::new(cr3, esp, esp0);

        Ok((kernel_stack, context))
    }
```

**Verus:**
```rust
pub fn forge_user_context(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
        // Context setup: no queue-level change.
    }
```
</details>

#### 🔴 `forge_user_context` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,37 +1,3 @@
 {
-        trace!("args={args:?}, enable_interrupts={enable_interrupts:?}",);
-
-        unsafe extern "C" {
-            pub fn __leave_kernel_to_user_mode();
-        }
-
-        // Assert pre-conditions (these should have been checked by the caller).
-        debug_assert!(Vmem::is_user_region(args.user_stack_base, args.user_stack_size));
-        debug_assert!(Vmem::is_user_addr(args.user_fn));
-
-        let kernel_func: VirtualAddress =
-            VirtualAddress::from_raw_value(__leave_kernel_to_user_mode as usize);
-
-        // Alloc kernel pages for the kernel stack. If we fail beyond this point, `kernel_stack`
-        // gets dropped as soon as we exit this scope and underlying pages are released.
-        let kernel_stack: KernelStack = KernelStack::new(mm)?;
-
-        let cr3: u32 = vmem.pgdir().physical_address()?.into_raw_value() as u32;
-        let esp: u32 = unsafe {
-            hal::arch::forge_user_stack(
-                kernel_stack.top().into_raw_value() as *mut u8,
-                args.user_stack_base.into_raw_value() + args.user_stack_size,
-                args.user_fn.into_raw_value(),
-                args.user_fn_arg0,
-                args.user_fn_arg1,
-                kernel_func.into_raw_value(),
-                enable_interrupts,
-            )
-        } as u32;
-        let esp0: u32 = kernel_stack.top().into_raw_value() as u32;
-
-        trace!("cr3={:#x}, esp={:#x}, ebp={:#x}", cr3, esp, esp0);
-        let context: ContextInformation = ContextInformation::new(cr3, esp, esp0);
-
-        Ok((kernel_stack, context))
+        // Context setup: no queue-level change.
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        trace!("args={args:?}, enable_interrupts={enable_interrupts:?}",);

        unsafe extern "C" {
            pub fn __leave_kernel_to_user_mode();
        }

        // Assert pre-conditions (these should have been checked by the caller).
        debug_assert!(Vmem::is_user_region(args.user_stack_base, args.user_stack_size));
        debug_assert!(Vmem::is_user_addr(args.user_fn));

        let kernel_func: VirtualAddress =
            VirtualAddress::from_raw_value(__leave_kernel_to_user_mode as usize);

        // Alloc kernel pages for the kernel stack. If we fail beyond this point, `kernel_stack`
        // gets dropped as soon as we exit this scope and underlying pages are released.
        let kernel_stack: KernelStack = KernelStack::new(mm)?;

        let cr3: u32 = vmem.pgdir().physical_address()?.into_raw_value() as u32;
        let esp: u32 = unsafe {
            hal::arch::forge_user_stack(
                kernel_stack.top().into_raw_value() as *mut u8,
                args.user_stack_base.into_raw_value() + args.user_stack_size,
                args.user_fn.into_raw_value(),
                args.user_fn_arg0,
                args.user_fn_arg1,
                kernel_func.into_raw_value(),
                enable_interrupts,
            )
        } as u32;
        let esp0: u32 = kernel_stack.top().into_raw_value() as u32;

        trace!("cr3={:#x}, esp={:#x}, ebp={:#x}", cr3, esp, esp0);
        let context: ContextInformation = ContextInformation::new(cr3, esp, esp0);

        Ok((kernel_stack, context))
    }
```

**Verus:**
```rust
{
        // Context setup: no queue-level change.
    }
```
</details>

#### 🟠 `set_thread_data_area` — signature_changed (high)

Parameters differ: source='(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        user_tda: Option<VirtualAddress>,
    )' vs verus='(&self, pid: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn set_thread_data_area(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        user_tda: Option<VirtualAddress>,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?
            .set_thread_data_area(pid, tid, user_tda)
    }
```

**Verus:**
```rust
pub fn set_thread_data_area(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
    {
        // Thread metadata update: no queue-level change (T3).
    }
```
</details>

#### 🟠 `set_thread_data_area` — return_type_changed (high)

Return type differs: source='Result<(), Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn set_thread_data_area(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        user_tda: Option<VirtualAddress>,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?
            .set_thread_data_area(pid, tid, user_tda)
    }
```

**Verus:**
```rust
pub fn set_thread_data_area(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
    {
        // Thread metadata update: no queue-level change (T3).
    }
```
</details>

#### 🔴 `set_thread_data_area` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,4 +1,3 @@
 {
-        self.try_borrow_mut()?
-            .set_thread_data_area(pid, tid, user_tda)
+        // Thread metadata update: no queue-level change (T3).
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.try_borrow_mut()?
            .set_thread_data_area(pid, tid, user_tda)
    }
```

**Verus:**
```rust
{
        // Thread metadata update: no queue-level change (T3).
    }
```
</details>

#### 🟠 `get_thread_data_area` — signature_changed (high)

Parameters differ: source='(
        &self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    )' vs verus='(&self, pid: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn get_thread_data_area(
        &self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    ) -> Result<Option<VirtualAddress>, Error> {
        self.try_borrow()?.get_thread_data_area(pid, tid)
    }
```

**Verus:**
```rust
pub fn get_thread_data_area(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `get_thread_data_area` — return_type_changed (high)

Return type differs: source='Result<Option<VirtualAddress>, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn get_thread_data_area(
        &self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    ) -> Result<Option<VirtualAddress>, Error> {
        self.try_borrow()?.get_thread_data_area(pid, tid)
    }
```

**Verus:**
```rust
pub fn get_thread_data_area(&self, pid: i32)
        requires
            self.wf(),
            self.ghost_suspended@.contains(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `get_thread_data_area` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,2 @@
 {
-        self.try_borrow()?.get_thread_data_area(pid, tid)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.try_borrow()?.get_thread_data_area(pid, tid)
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `create_process` — signature_changed (high)

Parameters differ: source='(
        &mut self,
        mm: &mut VirtMemoryManager,
        elf: &Elf32Fhdr,
        args: &str,
        env: &str,
    )' vs verus='(&mut self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn create_process(
        &mut self,
        mm: &mut VirtMemoryManager,
        elf: &Elf32Fhdr,
        args: &str,
        env: &str,
    ) -> Result<ProcessIdentifier, Error> {
        self.try_borrow_mut()?.create_process(mm, elf, args, env)
    }
```

**Verus:**
```rust
pub fn create_process(&mut self) -> (result: i32)
        requires
            old(self).wf(),
            old(self).spec_can_create_process(),
        ensures
            self.wf(),
            result as int == old(self).next_pid as int,
            self.spec_running_pid() == old(self).spec_running_pid(),
            self.next_pid as int == old(self).next_pid as int + 1,
            self.ready_count == old(self).ready_count + 1,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(old(self).next_pid as int),
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.spec_process_exists(result as int),
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let pid: i32 = self.next_pid;

        proof {
            self.lemma_next_pid_is_fresh();
            // PID is fresh: not in any ghost set, so insert adds exactly 1.
            assert(!self.ghost_ready@.contains(pid as int));
            assert(self.ghost_ready@.insert(pid as int).len()
                == self.ghost_ready@.len() + 1);
            // The new PID won't violate disjointness since it's not in any set.
            assert(!self.ghost_suspended@.contains(pid as int));
            assert(!self.ghost_interrupted@.contains(pid as int));
            assert(!self.ghost_zombies@.contains(pid as int));
            // Running PID not in new ready set (it wasn't before, and it's != pid
            // because running_pid < next_pid = pid, so running_pid != pid).
            assert(self.running_pid as int != pid as int);
            assert(!self.ghost_ready@.insert(pid as int).contains(self.running_pid as int));
            // All PIDs in the new ready set are < pid + 1.
            assert(forall |p: int| self.ghost_ready@.insert(pid as int).contains(p)

```
</details>

#### 🟠 `create_process` — return_type_changed (high)

Return type differs: source='Result<ProcessIdentifier, Error>' vs verus='i32'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn create_process(
        &mut self,
        mm: &mut VirtMemoryManager,
        elf: &Elf32Fhdr,
        args: &str,
        env: &str,
    ) -> Result<ProcessIdentifier, Error> {
        self.try_borrow_mut()?.create_process(mm, elf, args, env)
    }
```

**Verus:**
```rust
pub fn create_process(&mut self) -> (result: i32)
        requires
            old(self).wf(),
            old(self).spec_can_create_process(),
        ensures
            self.wf(),
            result as int == old(self).next_pid as int,
            self.spec_running_pid() == old(self).spec_running_pid(),
            self.next_pid as int == old(self).next_pid as int + 1,
            self.ready_count == old(self).ready_count + 1,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(old(self).next_pid as int),
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.spec_process_exists(result as int),
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let pid: i32 = self.next_pid;

        proof {
            self.lemma_next_pid_is_fresh();
            // PID is fresh: not in any ghost set, so insert adds exactly 1.
            assert(!self.ghost_ready@.contains(pid as int));
            assert(self.ghost_ready@.insert(pid as int).len()
                == self.ghost_ready@.len() + 1);
            // The new PID won't violate disjointness since it's not in any set.
            assert(!self.ghost_suspended@.contains(pid as int));
            assert(!self.ghost_interrupted@.contains(pid as int));
            assert(!self.ghost_zombies@.contains(pid as int));
            // Running PID not in new ready set (it wasn't before, and it's != pid
            // because running_pid < next_pid = pid, so running_pid != pid).
            assert(self.running_pid as int != pid as int);
            assert(!self.ghost_ready@.insert(pid as int).contains(self.running_pid as int));
            // All PIDs in the new ready set are < pid + 1.
            assert(forall |p: int| self.ghost_ready@.insert(pid as int).contains(p)

```
</details>

#### 🔴 `create_process` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,7 @@
 {
-        self.try_borrow_mut()?.create_process(mm, elf, args, env)
+        let pid: i32 = self.next_pid;
+        self.ghost_ready.pid_insert(pid as u64);
+        self.ready_count = self.ready_count + 1;
+        self.next_pid = pid + 1;
+        pid
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.try_borrow_mut()?.create_process(mm, elf, args, env)
    }
```

**Verus:**
```rust
{
        let pid: i32 = self.next_pid;

        proof {
            self.lemma_next_pid_is_fresh();
            // PID is fresh: not in any ghost set, so insert adds exactly 1.
            assert(!self.ghost_ready@.contains(pid as int));
            assert(self.ghost_ready@.insert(pid as int).len()
                == self.ghost_ready@.len() + 1);
            // The new PID won't violate disjointness since it's not in any set.
            assert(!self.ghost_suspended@.contains(pid as int));
            assert(!self.ghost_interrupted@.contains(pid as int));
            assert(!self.ghost_zombies@.contains(pid as int));
            // Running PID not in new ready set (it wasn't before, and it's != pid
            // because running_pid < next_pid = pid, so running_pid != pid).
            assert(self.running_pid as int != pid as int);
            assert(!self.ghost_ready@.insert(pid as int).contains(self.running_pid as int));
            // All PIDs in the new ready set are < pid + 1.
            assert(forall |p: int| self.ghost_ready@.insert(pid as int).contains(p)
                ==> 0 <= p && p < (pid + 1) as int);
            // PID bounds for other sets still hold with new next_pid.
            assert(forall |p: int| self.ghost_suspended@.contains(p)
                ==> p < (pid + 1) as int);
            assert(forall |p: int| self.ghost_interrupted@.contains(p)
                ==> p < (pid + 1) as int);
            assert(forall |p: int| self.ghost_zombies@.contains(p)
                ==> p < (pid + 1) as int);
            // Running PID is still < new next_pid.
            assert((self.running_pid as int) < (pid + 1) as int);
            // Counts bounded: old total + 1 ≤ old next_pid + 1 = new next_pid.
            assert((self.ready_count + 1) as int + (self.suspended_count as int)
                + (self.interrupted_count as int) + (self.zombie_count as int) + 1
                <= (pid + 1) as int);
        }

        self.ghost_ready.pid_insert(pid as u64)
```
</details>

#### 🟠 `schedule` — signature_changed (high)

Parameters differ: source='(
        &mut self,
    )' vs verus='(&mut self, chosen_next: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn schedule(
        &mut self,
    ) -> (
        ProcessIdentifier,
        ThreadIdentifier,
        *mut ContextInformation,
        *mut ContextInformation,
        Option<VirtualAddress>,
    ) {
        // Reschedule running process.
        let previous_process: RunningProcess = self.take_running();

        let (previous_process, previous_context) = previous_process.schedule();
        self.ready.push_back(previous_process);

        self.check_alarm();

        // Process all interrupted processes.
        while let Some(interrupted_process) = self.interrupted.pop_front() {
            let ready_process: RunnableProcess = interrupted_process.resume();
            self.ready.push_back(ready_process);
        }

        // Select next process to run.
        let next_process: RunnableProcess = self.take_earliest_ready();

        let (next_process, reason, next_context, user_tda): (
            RunningProcess,
            Option<InterruptReason>,
            *mut ContextInformation,
            Option<VirtualAddress>,
        ) = next_process.run();

        let next_pid: ProcessIdentifier = next_process.state().pid();
        let next_tid: ThreadIdentifier = next_process.get_tid();
        self.interrupt_reason = reason;
        self.running = Some(next_process);
        (next_pid, next_tid, previous_context, next_context, user_tda)
    }
```

**Verus:**
```rust
pub fn schedule(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).spec_ready_with_running().contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(
                old(self).running_pid as int
            ).remove(chosen_next as int),
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
        }

        self.ghost_ready.pid_insert(old_running as u64);
        self.ghost_ready.pid_remove(chosen_next as u64);
        self.running_pid = chosen_next;
    }
```
</details>

#### 🟠 `schedule` — return_type_changed (high)

Return type differs: source='(
        ProcessIdentifier,
        ThreadIdentifier,
        *mut ContextInformation,
        *mut ContextInformation,
        Option<VirtualAddress>,
    )' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn schedule(
        &mut self,
    ) -> (
        ProcessIdentifier,
        ThreadIdentifier,
        *mut ContextInformation,
        *mut ContextInformation,
        Option<VirtualAddress>,
    ) {
        // Reschedule running process.
        let previous_process: RunningProcess = self.take_running();

        let (previous_process, previous_context) = previous_process.schedule();
        self.ready.push_back(previous_process);

        self.check_alarm();

        // Process all interrupted processes.
        while let Some(interrupted_process) = self.interrupted.pop_front() {
            let ready_process: RunnableProcess = interrupted_process.resume();
            self.ready.push_back(ready_process);
        }

        // Select next process to run.
        let next_process: RunnableProcess = self.take_earliest_ready();

        let (next_process, reason, next_context, user_tda): (
            RunningProcess,
            Option<InterruptReason>,
            *mut ContextInformation,
            Option<VirtualAddress>,
        ) = next_process.run();

        let next_pid: ProcessIdentifier = next_process.state().pid();
        let next_tid: ThreadIdentifier = next_process.get_tid();
        self.interrupt_reason = reason;
        self.running = Some(next_process);
        (next_pid, next_tid, previous_context, next_context, user_tda)
    }
```

**Verus:**
```rust
pub fn schedule(&mut self, chosen_next: i32)
        requires
            old(self).wf(),
            old(self).spec_ready_with_running().contains(chosen_next as int),
            chosen_next >= 0i32,
            chosen_next < old(self).next_pid,
        ensures
            self.wf(),
            self.running_pid == chosen_next,
            self.ghost_ready@ =~= old(self).ghost_ready@.insert(
                old(self).running_pid as int
            ).remove(chosen_next as int),
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.number_buffered_messages == old(self).number_buffered_messages,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        let old_running: i32 = self.running_pid;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
        }

        self.ghost_ready.pid_insert(old_running as u64);
        self.ghost_ready.pid_remove(chosen_next as u64);
        self.running_pid = chosen_next;
    }
```
</details>

#### 🔴 `schedule` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,31 +1,6 @@
 {
-        // Reschedule running process.
-        let previous_process: RunningProcess = self.take_running();
-
-        let (previous_process, previous_context) = previous_process.schedule();
-        self.ready.push_back(previous_process);
-
-        self.check_alarm();
-
-        // Process all interrupted processes.
-        while let Some(interrupted_process) = self.interrupted.pop_front() {
-            let ready_process: RunnableProcess = interrupted_process.resume();
-            self.ready.push_back(ready_process);
-        }
-
-        // Select next process to run.
-        let next_process: RunnableProcess = self.take_earliest_ready();
-
-        let (next_process, reason, next_context, user_tda): (
-            RunningProcess,
-            Option<InterruptReason>,
-            *mut ContextInformation,
-            Option<VirtualAddress>,
-        ) = next_process.run();
-
-        let next_pid: ProcessIdentifier = next_process.state().pid();
-        let next_tid: ThreadIdentifier = next_process.get_tid();
-        self.interrupt_reason = reason;
-        self.running = Some(next_process);
-        (next_pid, next_tid, previous_context, next_context, user_tda)
+        let old_running: i32 = self.running_pid;
+        self.ghost_ready.pid_insert(old_running as u64);
+        self.ghost_ready.pid_remove(chosen_next as u64);
+        self.running_pid = chosen_next;
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        // Reschedule running process.
        let previous_process: RunningProcess = self.take_running();

        let (previous_process, previous_context) = previous_process.schedule();
        self.ready.push_back(previous_process);

        self.check_alarm();

        // Process all interrupted processes.
        while let Some(interrupted_process) = self.interrupted.pop_front() {
            let ready_process: RunnableProcess = interrupted_process.resume();
            self.ready.push_back(ready_process);
        }

        // Select next process to run.
        let next_process: RunnableProcess = self.take_earliest_ready();

        let (next_process, reason, next_context, user_tda): (
            RunningProcess,
            Option<InterruptReason>,
            *mut ContextInformation,
            Option<VirtualAddress>,
        ) = next_process.run();

        let next_pid: ProcessIdentifier = next_process.state().pid();
        let next_tid: ThreadIdentifier = next_process.get_tid();
        self.interrupt_reason = reason;
        self.running = Some(next_process);
        (next_pid, next_tid, previous_context, next_context, user_tda)
    }
```

**Verus:**
```rust
{
        let old_running: i32 = self.running_pid;

        proof {
            let old_ready: Set<int> = old(self).ghost_ready@;
            Self::lemma_schedule_ready_len(old_ready, old_running as int, chosen_next as int);
            self.lemma_kernel_alive_after_schedule(chosen_next as int);
        }

        self.ghost_ready.pid_insert(old_running as u64);
        self.ghost_ready.pid_remove(chosen_next as u64);
        self.running_pid = chosen_next;
    }
```
</details>

#### 🟠 `capctl` — signature_changed (high)

Parameters differ: source='(
        &mut self,
        pid: ProcessIdentifier,
        capability: Capability,
        value: bool,
    )' vs verus='(&self, pid: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn capctl(
        &mut self,
        pid: ProcessIdentifier,
        capability: Capability,
        value: bool,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?.capctl(pid, capability, value)
    }
```

**Verus:**
```rust
pub fn capctl(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `capctl` — return_type_changed (high)

Return type differs: source='Result<(), Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn capctl(
        &mut self,
        pid: ProcessIdentifier,
        capability: Capability,
        value: bool,
    ) -> Result<(), Error> {
        self.try_borrow_mut()?.capctl(pid, capability, value)
    }
```

**Verus:**
```rust
pub fn capctl(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `capctl` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,2 @@
 {
-        self.try_borrow_mut()?.capctl(pid, capability, value)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.try_borrow_mut()?.capctl(pid, capability, value)
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `handle_fpu_exception` — signature_changed (high)

Parameters differ: source='(&mut self)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn handle_fpu_exception(&mut self) -> Result<(), Error> {
        self.try_borrow_mut()?.handle_fpu_exception()
    }
```

**Verus:**
```rust
pub fn handle_fpu_exception(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `handle_fpu_exception` — return_type_changed (high)

Return type differs: source='Result<(), Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn handle_fpu_exception(&mut self) -> Result<(), Error> {
        self.try_borrow_mut()?.handle_fpu_exception()
    }
```

**Verus:**
```rust
pub fn handle_fpu_exception(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `handle_fpu_exception` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,2 @@
 {
-        self.try_borrow_mut()?.handle_fpu_exception()
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.try_borrow_mut()?.handle_fpu_exception()
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `try_join_thread` — signature_changed (high)

Parameters differ: source='(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    )' vs verus='(&self, pid: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn try_join_thread(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    ) -> Result<ZombieThread, Result<Condvar, Error>> {
        match self.find_process_mut(pid).map_err(Err)? {
            ProcessRefMut::Running(process) => process.try_join_thread(tid),
            ProcessRefMut::Runnable(_) => {
                let reason: &str = "process is runnable";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Sleeping(_) => {
                let reason: &str = "process is sleeping";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Interrupted(_) => {
                let reason: &str = "process is interrupted";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Zombie(_) => {
                let reason: &str = "process is a zombie";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
        }
    }
```

**Verus:**
```rust
pub fn try_join_thread(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
        // Thread join: no queue-level change (T3).
    }
```
</details>

#### 🟠 `try_join_thread` — return_type_changed (high)

Return type differs: source='Result<ZombieThread, Result<Condvar, Error>>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn try_join_thread(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
    ) -> Result<ZombieThread, Result<Condvar, Error>> {
        match self.find_process_mut(pid).map_err(Err)? {
            ProcessRefMut::Running(process) => process.try_join_thread(tid),
            ProcessRefMut::Runnable(_) => {
                let reason: &str = "process is runnable";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Sleeping(_) => {
                let reason: &str = "process is sleeping";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Interrupted(_) => {
                let reason: &str = "process is interrupted";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Zombie(_) => {
                let reason: &str = "process is a zombie";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
        }
    }
```

**Verus:**
```rust
pub fn try_join_thread(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
        // Thread join: no queue-level change (T3).
    }
```
</details>

#### 🔴 `try_join_thread` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,25 +1,3 @@
 {
-        match self.find_process_mut(pid).map_err(Err)? {
-            ProcessRefMut::Running(process) => process.try_join_thread(tid),
-            ProcessRefMut::Runnable(_) => {
-                let reason: &str = "process is runnable";
-                error!("{reason} (pid={pid:?}, tid={tid:?})");
-                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
-            },
-            ProcessRefMut::Sleeping(_) => {
-                let reason: &str = "process is sleeping";
-                error!("{reason} (pid={pid:?}, tid={tid:?})");
-                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
-            },
-            ProcessRefMut::Interrupted(_) => {
-                let reason: &str = "process is interrupted";
-                error!("{reason} (pid={pid:?}, tid={tid:?})");
-                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
-            },
-            ProcessRefMut::Zombie(_) => {
-                let reason: &str = "process is a zombie";
-                error!("{reason} (pid={pid:?}, tid={tid:?})");
-                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
-            },
-        }
+        // Thread join: no queue-level change (T3).
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        match self.find_process_mut(pid).map_err(Err)? {
            ProcessRefMut::Running(process) => process.try_join_thread(tid),
            ProcessRefMut::Runnable(_) => {
                let reason: &str = "process is runnable";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Sleeping(_) => {
                let reason: &str = "process is sleeping";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Interrupted(_) => {
                let reason: &str = "process is interrupted";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
            ProcessRefMut::Zombie(_) => {
                let reason: &str = "process is a zombie";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                Err(Err(Error::new(ErrorCode::OperationNotPermitted, reason)))
            },
        }
    }
```

**Verus:**
```rust
{
        // Thread join: no queue-level change (T3).
    }
```
</details>

#### 🟠 `get_mutex` — signature_changed (high)

Parameters differ: source='(&mut self, mutex_addr: MutexAddress)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
        self.get_running_mut().state_mut().get_mutex(mutex_addr)
    }
```

**Verus:**
```rust
pub fn get_mutex(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `get_mutex` — return_type_changed (high)

Return type differs: source='Result<Mutex, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn get_mutex(&mut self, mutex_addr: MutexAddress) -> Result<Mutex, Error> {
        self.get_running_mut().state_mut().get_mutex(mutex_addr)
    }
```

**Verus:**
```rust
pub fn get_mutex(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `get_mutex` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,2 @@
 {
-        self.get_running_mut().state_mut().get_mutex(mutex_addr)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.get_running_mut().state_mut().get_mutex(mutex_addr)
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `get_cond` — signature_changed (high)

Parameters differ: source='(&mut self, cond_addr: ConditionAddress)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn get_cond(&mut self, cond_addr: ConditionAddress) -> Result<Condvar, Error> {
        self.get_running_mut().state_mut().get_cond(cond_addr)
    }
```

**Verus:**
```rust
pub fn get_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `get_cond` — return_type_changed (high)

Return type differs: source='Result<Condvar, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn get_cond(&mut self, cond_addr: ConditionAddress) -> Result<Condvar, Error> {
        self.get_running_mut().state_mut().get_cond(cond_addr)
    }
```

**Verus:**
```rust
pub fn get_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `get_cond` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,2 @@
 {
-        self.get_running_mut().state_mut().get_cond(cond_addr)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.get_running_mut().state_mut().get_cond(cond_addr)
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `put_cond` — signature_changed (high)

Parameters differ: source='(&mut self, cond_addr: ConditionAddress)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn put_cond(&mut self, cond_addr: ConditionAddress) -> Result<(), Error> {
        self.get_running_mut().state_mut().put_cond(cond_addr)
    }
```

**Verus:**
```rust
pub fn put_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `put_cond` — return_type_changed (high)

Return type differs: source='Result<(), Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn put_cond(&mut self, cond_addr: ConditionAddress) -> Result<(), Error> {
        self.get_running_mut().state_mut().put_cond(cond_addr)
    }
```

**Verus:**
```rust
pub fn put_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `put_cond` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,2 @@
 {
-        self.get_running_mut().state_mut().put_cond(cond_addr)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.get_running_mut().state_mut().put_cond(cond_addr)
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `put_mutex_guard` — signature_changed (high)

Parameters differ: source='(&mut self, mutex_addr: MutexAddress, guard: MutexGuard)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn put_mutex_guard(&mut self, mutex_addr: MutexAddress, guard: MutexGuard) {
        self.get_running_mut()
            .running_mut()
            .put_mutex_guard(mutex_addr, guard);
    }
```

**Verus:**
```rust
pub fn put_mutex_guard(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `put_mutex_guard` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,5 +1,2 @@
 {
-        self.get_running_mut()
-            .running_mut()
-            .put_mutex_guard(mutex_addr, guard);
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.get_running_mut()
            .running_mut()
            .put_mutex_guard(mutex_addr, guard);
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `take_mutex_guard` — signature_changed (high)

Parameters differ: source='(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        mutex_addr: MutexAddress,
    )' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn take_mutex_guard(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        mutex_addr: MutexAddress,
    ) -> Result<MutexGuard, Error> {
        let mutex_guard: MutexGuard = match self
            .get_running_mut()
            .running_mut()
            .take_mutex_guard(mutex_addr)
        {
            Some(mutex_guard) => mutex_guard,
            None => {
                let reason: &str = "thread does not own mutex";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                return Err(Error::new(ErrorCode::OperationNotPermitted, reason));
            },
        };

        self.get_running_mut().state_mut().put_mutex(mutex_addr)?;

        Ok(mutex_guard)
    }
```

**Verus:**
```rust
pub fn take_mutex_guard(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `take_mutex_guard` — return_type_changed (high)

Return type differs: source='Result<MutexGuard, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn take_mutex_guard(
        &mut self,
        pid: ProcessIdentifier,
        tid: ThreadIdentifier,
        mutex_addr: MutexAddress,
    ) -> Result<MutexGuard, Error> {
        let mutex_guard: MutexGuard = match self
            .get_running_mut()
            .running_mut()
            .take_mutex_guard(mutex_addr)
        {
            Some(mutex_guard) => mutex_guard,
            None => {
                let reason: &str = "thread does not own mutex";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                return Err(Error::new(ErrorCode::OperationNotPermitted, reason));
            },
        };

        self.get_running_mut().state_mut().put_mutex(mutex_addr)?;

        Ok(mutex_guard)
    }
```

**Verus:**
```rust
pub fn take_mutex_guard(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `take_mutex_guard` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,18 +1,2 @@
 {
-        let mutex_guard: MutexGuard = match self
-            .get_running_mut()
-            .running_mut()
-            .take_mutex_guard(mutex_addr)
-        {
-            Some(mutex_guard) => mutex_guard,
-            None => {
-                let reason: &str = "thread does not own mutex";
-                error!("{reason} (pid={pid:?}, tid={tid:?})");
-                return Err(Error::new(ErrorCode::OperationNotPermitted, reason));
-            },
-        };
-
-        self.get_running_mut().state_mut().put_mutex(mutex_addr)?;
-
-        Ok(mutex_guard)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        let mutex_guard: MutexGuard = match self
            .get_running_mut()
            .running_mut()
            .take_mutex_guard(mutex_addr)
        {
            Some(mutex_guard) => mutex_guard,
            None => {
                let reason: &str = "thread does not own mutex";
                error!("{reason} (pid={pid:?}, tid={tid:?})");
                return Err(Error::new(ErrorCode::OperationNotPermitted, reason));
            },
        };

        self.get_running_mut().state_mut().put_mutex(mutex_addr)?;

        Ok(mutex_guard)
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `take_earliest_ready` — signature_changed (high)

Parameters differ: source='(&mut self)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn take_earliest_ready(&mut self) -> RunnableProcess {
        // SAFETY: As the kernel process is always runnable, the following statement will never panic.
        let mut selected: (usize, SystemTime) = (
            0,
            self.ready
                .front()
                .expect("there should always be a process ready to run")
                .earliest_admission_time(),
        );

        // Select process with the earliest admission time.
        for (i, process) in self.ready.iter().enumerate() {
            let process_admission_time: SystemTime = process.earliest_admission_time();
            if process_admission_time < selected.1 {
                selected = (i, process_admission_time);
            }
        }

        // Remove the selected process from the list of ready processes.
        self.ready.remove(selected.0)
    }
```

**Verus:**
```rust
pub fn take_earliest_ready(&self)
        requires
            self.wf(),
            self.spec_has_ready(),
        ensures
            self.wf(),
    {
        // Selection is abstracted by T1: chosen_next parameter in schedule.
    }
```
</details>

#### 🟠 `take_earliest_ready` — return_type_changed (high)

Return type differs: source='RunnableProcess' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn take_earliest_ready(&mut self) -> RunnableProcess {
        // SAFETY: As the kernel process is always runnable, the following statement will never panic.
        let mut selected: (usize, SystemTime) = (
            0,
            self.ready
                .front()
                .expect("there should always be a process ready to run")
                .earliest_admission_time(),
        );

        // Select process with the earliest admission time.
        for (i, process) in self.ready.iter().enumerate() {
            let process_admission_time: SystemTime = process.earliest_admission_time();
            if process_admission_time < selected.1 {
                selected = (i, process_admission_time);
            }
        }

        // Remove the selected process from the list of ready processes.
        self.ready.remove(selected.0)
    }
```

**Verus:**
```rust
pub fn take_earliest_ready(&self)
        requires
            self.wf(),
            self.spec_has_ready(),
        ensures
            self.wf(),
    {
        // Selection is abstracted by T1: chosen_next parameter in schedule.
    }
```
</details>

#### 🔴 `take_earliest_ready` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,21 +1,3 @@
 {
-        // SAFETY: As the kernel process is always runnable, the following statement will never panic.
-        let mut selected: (usize, SystemTime) = (
-            0,
-            self.ready
-                .front()
-                .expect("there should always be a process ready to run")
-                .earliest_admission_time(),
-        );
-
-        // Select process with the earliest admission time.
-        for (i, process) in self.ready.iter().enumerate() {
-            let process_admission_time: SystemTime = process.earliest_admission_time();
-            if process_admission_time < selected.1 {
-                selected = (i, process_admission_time);
-            }
-        }
-
-        // Remove the selected process from the list of ready processes.
-        self.ready.remove(selected.0)
+        // Selection is abstracted by T1: chosen_next parameter in schedule.
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        // SAFETY: As the kernel process is always runnable, the following statement will never panic.
        let mut selected: (usize, SystemTime) = (
            0,
            self.ready
                .front()
                .expect("there should always be a process ready to run")
                .earliest_admission_time(),
        );

        // Select process with the earliest admission time.
        for (i, process) in self.ready.iter().enumerate() {
            let process_admission_time: SystemTime = process.earliest_admission_time();
            if process_admission_time < selected.1 {
                selected = (i, process_admission_time);
            }
        }

        // Remove the selected process from the list of ready processes.
        self.ready.remove(selected.0)
    }
```

**Verus:**
```rust
{
        // Selection is abstracted by T1: chosen_next parameter in schedule.
    }
```
</details>

#### 🟠 `take_running` — signature_changed (high)

Parameters differ: source='(&mut self)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn take_running(&mut self) -> RunningProcess {
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.take().expect("the kernel should be running")
    }
```

**Verus:**
```rust
pub fn take_running(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
        // Internal helper subsumed by schedule/sleep/exit verified transitions.
    }
```
</details>

#### 🟠 `take_running` — return_type_changed (high)

Return type differs: source='RunningProcess' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn take_running(&mut self) -> RunningProcess {
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.take().expect("the kernel should be running")
    }
```

**Verus:**
```rust
pub fn take_running(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
        // Internal helper subsumed by schedule/sleep/exit verified transitions.
    }
```
</details>

#### 🔴 `take_running` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,4 +1,3 @@
 {
-        // NOTE: it is safe to call unwrap because there is always a process running.
-        self.running.take().expect("the kernel should be running")
+        // Internal helper subsumed by schedule/sleep/exit verified transitions.
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.take().expect("the kernel should be running")
    }
```

**Verus:**
```rust
{
        // Internal helper subsumed by schedule/sleep/exit verified transitions.
    }
```
</details>

#### 🟠 `get_running` — return_type_changed (high)

Return type differs: source='&RunningProcess' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn get_running(&self) -> &RunningProcess {
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.as_ref().expect("the kernel should be running")
    }
```

**Verus:**
```rust
pub fn get_running(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `get_running` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,4 +1,2 @@
 {
-        // NOTE: it is safe to call unwrap because there is always a process running.
-        self.running.as_ref().expect("the kernel should be running")
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.as_ref().expect("the kernel should be running")
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `get_running_mut` — signature_changed (high)

Parameters differ: source='(&mut self)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn get_running_mut(&mut self) -> &mut RunningProcess {
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.as_mut().expect("the kernel should be running")
    }
```

**Verus:**
```rust
pub fn get_running_mut(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `get_running_mut` — return_type_changed (high)

Return type differs: source='&mut RunningProcess' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn get_running_mut(&mut self) -> &mut RunningProcess {
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.as_mut().expect("the kernel should be running")
    }
```

**Verus:**
```rust
pub fn get_running_mut(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `get_running_mut` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,4 +1,2 @@
 {
-        // NOTE: it is safe to call unwrap because there is always a process running.
-        self.running.as_mut().expect("the kernel should be running")
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        // NOTE: it is safe to call unwrap because there is always a process running.
        self.running.as_mut().expect("the kernel should be running")
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `find_process` — signature_changed (high)

Parameters differ: source='(&self, pid: ProcessIdentifier)' vs verus='(&self, pid: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_process(&self, pid: ProcessIdentifier) -> Result<ProcessRef<'_>, Error> {
        if self.get_running().state().pid() == pid {
            Ok(ProcessRef::Running(self.get_running()))
        } else if let Some(process) = self.ready.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Runnable(process))
        } else if let Some(process) = self.suspended.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Interrupted(process))
        } else if let Some(process) = self.zombies.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
```

**Verus:**
```rust
pub fn find_process(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `find_process` — return_type_changed (high)

Return type differs: source='Result<ProcessRef<'_>, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_process(&self, pid: ProcessIdentifier) -> Result<ProcessRef<'_>, Error> {
        if self.get_running().state().pid() == pid {
            Ok(ProcessRef::Running(self.get_running()))
        } else if let Some(process) = self.ready.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Runnable(process))
        } else if let Some(process) = self.suspended.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Interrupted(process))
        } else if let Some(process) = self.zombies.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
```

**Verus:**
```rust
pub fn find_process(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `find_process` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,17 +1,2 @@
 {
-        if self.get_running().state().pid() == pid {
-            Ok(ProcessRef::Running(self.get_running()))
-        } else if let Some(process) = self.ready.iter().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRef::Runnable(process))
-        } else if let Some(process) = self.suspended.iter().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRef::Sleeping(process))
-        } else if let Some(process) = self.interrupted.iter().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRef::Interrupted(process))
-        } else if let Some(process) = self.zombies.iter().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRef::Zombie(process))
-        } else {
-            let reason: &str = "process not found";
-            error!("{reason} (pid={pid:?})");
-            Err(Error::new(ErrorCode::NoSuchProcess, reason))
-        }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        if self.get_running().state().pid() == pid {
            Ok(ProcessRef::Running(self.get_running()))
        } else if let Some(process) = self.ready.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Runnable(process))
        } else if let Some(process) = self.suspended.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Interrupted(process))
        } else if let Some(process) = self.zombies.iter().find(|p| p.state().pid() == pid) {
            Ok(ProcessRef::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `find_process_mut` — signature_changed (high)

Parameters differ: source='(&mut self, pid: ProcessIdentifier)' vs verus='(&self, pid: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_process_mut(&mut self, pid: ProcessIdentifier) -> Result<ProcessRefMut<'_>, Error> {
        if self.get_running_mut().state().pid() == pid {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self.suspended.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self.zombies.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
```

**Verus:**
```rust
pub fn find_process_mut(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `find_process_mut` — return_type_changed (high)

Return type differs: source='Result<ProcessRefMut<'_>, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_process_mut(&mut self, pid: ProcessIdentifier) -> Result<ProcessRefMut<'_>, Error> {
        if self.get_running_mut().state().pid() == pid {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self.suspended.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self.zombies.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
```

**Verus:**
```rust
pub fn find_process_mut(&self, pid: i32)
        requires
            self.wf(),
            self.spec_process_exists(pid as int),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `find_process_mut` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,17 +1,2 @@
 {
-        if self.get_running_mut().state().pid() == pid {
-            Ok(ProcessRefMut::Running(self.get_running_mut()))
-        } else if let Some(process) = self.ready.iter_mut().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRefMut::Runnable(process))
-        } else if let Some(process) = self.suspended.iter_mut().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRefMut::Sleeping(process))
-        } else if let Some(process) = self.interrupted.iter_mut().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRefMut::Interrupted(process))
-        } else if let Some(process) = self.zombies.iter_mut().find(|p| p.state().pid() == pid) {
-            Ok(ProcessRefMut::Zombie(process))
-        } else {
-            let reason: &str = "process not found";
-            error!("{reason} (pid={pid:?})");
-            Err(Error::new(ErrorCode::NoSuchProcess, reason))
-        }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        if self.get_running_mut().state().pid() == pid {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self.suspended.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self.interrupted.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self.zombies.iter_mut().find(|p| p.state().pid() == pid) {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "process not found";
            error!("{reason} (pid={pid:?})");
            Err(Error::new(ErrorCode::NoSuchProcess, reason))
        }
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `find_process_by_tid` — signature_changed (high)

Parameters differ: source='(&mut self, tid: ThreadIdentifier)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_process_by_tid(&mut self, tid: ThreadIdentifier) -> Result<ProcessRefMut<'_>, Error> {
        if self.get_running_mut().find_thread(tid).is_some() {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.find_thread(tid).is_some()) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self
            .suspended
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self
            .interrupted
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self
            .zombies
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "thread not found";
            error!("{reason} (tid={tid:?})");
            Err(Error::new(ErrorCode::NoSuchEntry, reason))
        }
    }
```

**Verus:**
```rust
pub fn find_process_by_tid(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `find_process_by_tid` — return_type_changed (high)

Return type differs: source='Result<ProcessRefMut<'_>, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_process_by_tid(&mut self, tid: ThreadIdentifier) -> Result<ProcessRefMut<'_>, Error> {
        if self.get_running_mut().find_thread(tid).is_some() {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.find_thread(tid).is_some()) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self
            .suspended
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self
            .interrupted
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self
            .zombies
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "thread not found";
            error!("{reason} (tid={tid:?})");
            Err(Error::new(ErrorCode::NoSuchEntry, reason))
        }
    }
```

**Verus:**
```rust
pub fn find_process_by_tid(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `find_process_by_tid` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,29 +1,2 @@
 {
-        if self.get_running_mut().find_thread(tid).is_some() {
-            Ok(ProcessRefMut::Running(self.get_running_mut()))
-        } else if let Some(process) = self.ready.iter_mut().find(|p| p.find_thread(tid).is_some()) {
-            Ok(ProcessRefMut::Runnable(process))
-        } else if let Some(process) = self
-            .suspended
-            .iter_mut()
-            .find(|p| p.find_thread(tid).is_some())
-        {
-            Ok(ProcessRefMut::Sleeping(process))
-        } else if let Some(process) = self
-            .interrupted
-            .iter_mut()
-            .find(|p| p.find_thread(tid).is_some())
-        {
-            Ok(ProcessRefMut::Interrupted(process))
-        } else if let Some(process) = self
-            .zombies
-            .iter_mut()
-            .find(|p| p.find_thread(tid).is_some())
-        {
-            Ok(ProcessRefMut::Zombie(process))
-        } else {
-            let reason: &str = "thread not found";
-            error!("{reason} (tid={tid:?})");
-            Err(Error::new(ErrorCode::NoSuchEntry, reason))
-        }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        if self.get_running_mut().find_thread(tid).is_some() {
            Ok(ProcessRefMut::Running(self.get_running_mut()))
        } else if let Some(process) = self.ready.iter_mut().find(|p| p.find_thread(tid).is_some()) {
            Ok(ProcessRefMut::Runnable(process))
        } else if let Some(process) = self
            .suspended
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Sleeping(process))
        } else if let Some(process) = self
            .interrupted
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Interrupted(process))
        } else if let Some(process) = self
            .zombies
            .iter_mut()
            .find(|p| p.find_thread(tid).is_some())
        {
            Ok(ProcessRefMut::Zombie(process))
        } else {
            let reason: &str = "thread not found";
            error!("{reason} (tid={tid:?})");
            Err(Error::new(ErrorCode::NoSuchEntry, reason))
        }
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `find_thread_mut` — signature_changed (high)

Parameters differ: source='(&mut self, tid: ThreadIdentifier)' vs verus='(&self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_thread_mut(&mut self, tid: ThreadIdentifier) -> Result<ThreadRefMut<'_>, Error> {
        // Search thread in the running process.
        if let Some(thread) = self.running.as_mut() {
            if let Some(thread) = thread.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of ready processes.
        for process in self.ready.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of sleeping processes.
        for process in self.suspended.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of interrupted processes.
        for process in self.interrupted.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of zombie processes.
        for process in self.zombies.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        let reason: &str = "thread not found";
        error!("{reason} (tid={tid:?})");
        Err(Error::new(ErrorCode::NoSuchEntry, reason))
    }
```

**Verus:**
```rust
pub fn find_thread_mut(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🟠 `find_thread_mut` — return_type_changed (high)

Return type differs: source='Result<ThreadRefMut<'_>, Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn find_thread_mut(&mut self, tid: ThreadIdentifier) -> Result<ThreadRefMut<'_>, Error> {
        // Search thread in the running process.
        if let Some(thread) = self.running.as_mut() {
            if let Some(thread) = thread.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of ready processes.
        for process in self.ready.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of sleeping processes.
        for process in self.suspended.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of interrupted processes.
        for process in self.interrupted.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of zombie processes.
        for process in self.zombies.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        let reason: &str = "thread not found";
        error!("{reason} (tid={tid:?})");
        Err(Error::new(ErrorCode::NoSuchEntry, reason))
    }
```

**Verus:**
```rust
pub fn find_thread_mut(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
```
</details>

#### 🔴 `find_thread_mut` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,40 +1,2 @@
 {
-        // Search thread in the running process.
-        if let Some(thread) = self.running.as_mut() {
-            if let Some(thread) = thread.find_thread_mut(tid) {
-                return Ok(thread);
-            }
-        }
-
-        // Search thread in the list of ready processes.
-        for process in self.ready.iter_mut() {
-            if let Some(thread) = process.find_thread_mut(tid) {
-                return Ok(thread);
-            }
-        }
-
-        // Search thread in the list of sleeping processes.
-        for process in self.suspended.iter_mut() {
-            if let Some(thread) = process.find_thread_mut(tid) {
-                return Ok(thread);
-            }
-        }
-
-        // Search thread in the list of interrupted processes.
-        for process in self.interrupted.iter_mut() {
-            if let Some(thread) = process.find_thread_mut(tid) {
-                return Ok(thread);
-            }
-        }
-
-        // Search thread in the list of zombie processes.
-        for process in self.zombies.iter_mut() {
-            if let Some(thread) = process.find_thread_mut(tid) {
-                return Ok(thread);
-            }
-        }
-
-        let reason: &str = "thread not found";
-        error!("{reason} (tid={tid:?})");
-        Err(Error::new(ErrorCode::NoSuchEntry, reason))
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        // Search thread in the running process.
        if let Some(thread) = self.running.as_mut() {
            if let Some(thread) = thread.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of ready processes.
        for process in self.ready.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of sleeping processes.
        for process in self.suspended.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of interrupted processes.
        for process in self.interrupted.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        // Search thread in the list of zombie processes.
        for process in self.zombies.iter_mut() {
            if let Some(thread) = process.find_thread_mut(tid) {
                return Ok(thread);
            }
        }

        let reason: &str = "thread not found";
        error!("{reason} (tid={tid:?})");
        Err(Error::new(ErrorCode::NoSuchEntry, reason))
    }
```

**Verus:**
```rust
{
    }
```
</details>

#### 🟠 `post_message` — signature_changed (high)

Parameters differ: source='(
        &mut self,
        receiver: MessageReceiver,
        message: Message,
    )' vs verus='(&mut self, receiver_pid: i32)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn post_message(
        &mut self,
        receiver: MessageReceiver,
        message: Message,
    ) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = match receiver.as_id() {
            Ok(pid) => pm.find_process_mut(pid)?,
            Err(tid) => pm.find_process_by_tid(tid)?,
        };
        process.state_mut().post_message(message);
        pm.number_buffered_messages += 1;
        Ok(())
    }
```

**Verus:**
```rust
pub fn post_message(&mut self, receiver_pid: i32)
        requires
            old(self).wf(),
            old(self).spec_process_exists(receiver_pid as int),
            old(self).number_buffered_messages < usize::MAX - 1,
        ensures
            self.wf(),
            self.number_buffered_messages == old(self).number_buffered_messages + 1,
            self.running_pid == old(self).running_pid,
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.number_buffered_messages = self.number_buffered_messages + 1;
    }
```
</details>

#### 🟠 `post_message` — return_type_changed (high)

Return type differs: source='Result<(), Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn post_message(
        &mut self,
        receiver: MessageReceiver,
        message: Message,
    ) -> Result<(), Error> {
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = match receiver.as_id() {
            Ok(pid) => pm.find_process_mut(pid)?,
            Err(tid) => pm.find_process_by_tid(tid)?,
        };
        process.state_mut().post_message(message);
        pm.number_buffered_messages += 1;
        Ok(())
    }
```

**Verus:**
```rust
pub fn post_message(&mut self, receiver_pid: i32)
        requires
            old(self).wf(),
            old(self).spec_process_exists(receiver_pid as int),
            old(self).number_buffered_messages < usize::MAX - 1,
        ensures
            self.wf(),
            self.number_buffered_messages == old(self).number_buffered_messages + 1,
            self.running_pid == old(self).running_pid,
            self.ready_count == old(self).ready_count,
            self.suspended_count == old(self).suspended_count,
            self.interrupted_count == old(self).interrupted_count,
            self.zombie_count == old(self).zombie_count,
            self.next_pid == old(self).next_pid,
            self.interrupt_capable == old(self).interrupt_capable,
    {
        self.number_buffered_messages = self.number_buffered_messages + 1;
    }
```
</details>

#### 🔴 `post_message` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,10 +1,3 @@
 {
-        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
-        let mut process: ProcessRefMut = match receiver.as_id() {
-            Ok(pid) => pm.find_process_mut(pid)?,
-            Err(tid) => pm.find_process_by_tid(tid)?,
-        };
-        process.state_mut().post_message(message);
-        pm.number_buffered_messages += 1;
-        Ok(())
+        self.number_buffered_messages = self.number_buffered_messages + 1;
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        let mut pm: RefMut<ProcessManagerInner> = self.try_borrow_mut()?;
        let mut process: ProcessRefMut = match receiver.as_id() {
            Ok(pid) => pm.find_process_mut(pid)?,
            Err(tid) => pm.find_process_by_tid(tid)?,
        };
        process.state_mut().post_message(message);
        pm.number_buffered_messages += 1;
        Ok(())
    }
```

**Verus:**
```rust
{
        self.number_buffered_messages = self.number_buffered_messages + 1;
    }
```
</details>

## Module: thread_manager

- **Source:** `src/kernel/src/pm/thread/mod.rs`
- **Verus:** `verus/split/kernel/pm/thread/thread_manager.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 4 critical, 4 high, 1 medium. Structs: 2 diffs. Functions: 7 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `ReadyThread` | added_struct | **high** | New struct added in verus that doesn't exist in source. |
| `ThreadManager` | visibility_changed | **medium** | Field 'next_id' changed from private to pub (Verus constraint). |

### Function Differences

#### 🟠 `thread_state_mut` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🔴 `thread_state` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,9 +1,9 @@
 {
         match self {
-            ThreadRef::Ready(thread) => thread.thread_state(),
-            ThreadRef::Running(thread) => thread.thread_state(),
-            ThreadRef::Sleeping(thread) => thread.thread_state(),
-            ThreadRef::Interrupted(thread) => thread.thread_state(),
-            ThreadRef::Zombie(thread) => thread.thread_state(),
+            ThreadRefMutModel::Ready(state) => state,
+            ThreadRefMutModel::Running(state) => state,
+            ThreadRefMutModel::Sleeping(state) => state,
+            ThreadRefMutModel::Interrupted(state) => state,
+            ThreadRefMutModel::Zombie(state) => state,
         }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        match self {
            ThreadRef::Ready(thread) => thread.thread_state(),
            ThreadRef::Running(thread) => thread.thread_state(),
            ThreadRef::Sleeping(thread) => thread.thread_state(),
            ThreadRef::Interrupted(thread) => thread.thread_state(),
            ThreadRef::Zombie(thread) => thread.thread_state(),
        }
    }
```

**Verus:**
```rust
{
        match self {
            ThreadRefMutModel::Ready(state) => state,
            ThreadRefMutModel::Running(state) => state,
            ThreadRefMutModel::Sleeping(state) => state,
            ThreadRefMutModel::Interrupted(state) => state,
            ThreadRefMutModel::Zombie(state) => state,
        }
    }
```
</details>

#### 🟠 `new` — return_type_changed (high)

Return type differs: source='(ReadyThread, Self)' vs verus='(ReadyThread, ThreadManager)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
fn new() -> (ReadyThread, Self) {
        let kernel: ReadyThread = {
            ReadyThread::new(
                From::<i32>::from(0),
                None,
                None,
                None,
                ContextInformation::default(),
                // SAFETY: calls to FpuState::new are synchronized.
                unsafe { FpuState::new() },
            )
        };
        (
            kernel,
            Self {
                next_id: From::<i32>::from(1),
            },
        )
    }
```

**Verus:**
```rust
fn new() -> (result: (ReadyThread, ThreadManager))
        ensures
            result.0.spec_id() == 0,
            result.0.wf(),
            result.0.spec_drop_safe(),
            !result.0.spec_is_interrupted(),
            result.0.spec_locked_mutex_count() == 0,
            result.1.spec_next_id() == 1,
            result.1.wf(),
    {
        let kernel: ReadyThread = ReadyThread::new(
            ThreadIdentifier::from_i32(0),
            None,
            None,
            None,
        );
        let manager: ThreadManager = ThreadManager {
            next_id: ThreadIdentifier::from_i32(1),
        };
        (kernel, manager)
    }
```
</details>

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,19 +1,12 @@
 {
-        let kernel: ReadyThread = {
-            ReadyThread::new(
-                From::<i32>::from(0),
-                None,
-                None,
-                None,
-                ContextInformation::default(),
-                // SAFETY: calls to FpuState::new are synchronized.
-                unsafe { FpuState::new() },
-            )
+        let kernel: ReadyThread = ReadyThread::new(
+            ThreadIdentifier::from_i32(0),
+            None,
+            None,
+            None,
+        );
+        let manager: ThreadManager = ThreadManager {
+            next_id: ThreadIdentifier::from_i32(1),
         };
-        (
-            kernel,
-            Self {
-                next_id: From::<i32>::from(1),
-            },
-        )
+        (kernel, manager)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        let kernel: ReadyThread = {
            ReadyThread::new(
                From::<i32>::from(0),
                None,
                None,
                None,
                ContextInformation::default(),
                // SAFETY: calls to FpuState::new are synchronized.
                unsafe { FpuState::new() },
            )
        };
        (
            kernel,
            Self {
                next_id: From::<i32>::from(1),
            },
        )
    }
```

**Verus:**
```rust
{
        let kernel: ReadyThread = ReadyThread::new(
            ThreadIdentifier::from_i32(0),
            None,
            None,
            None,
        );
        let manager: ThreadManager = ThreadManager {
            next_id: ThreadIdentifier::from_i32(1),
        };
        (kernel, manager)
    }
```
</details>

#### 🟠 `create_thread` — signature_changed (high)

Parameters differ: source='(
        &mut self,
        kernel_stack: Option<KernelStack>,
        user_stack: Option<UserStack>,
        user_tda: Option<VirtualAddress>,
        context: ContextInformation,
    )' vs verus='(
        &mut self,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    )'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn create_thread(
        &mut self,
        kernel_stack: Option<KernelStack>,
        user_stack: Option<UserStack>,
        user_tda: Option<VirtualAddress>,
        context: ContextInformation,
    ) -> ReadyThread {
        let id: ThreadIdentifier = self.next_id;
        self.next_id = ThreadIdentifier::from(<i32>::from(self.next_id) + 1);

        ReadyThread::new(
            id,
            kernel_stack,
            user_stack,
            user_tda,
            context,
            // SAFETY: calls to FpuState::new are synchronized.
            unsafe { FpuState::new() },
        )
    }
```

**Verus:**
```rust
pub fn create_thread(
        &mut self,
        kernel_stack: Option<int>,
        user_stack: Option<int>,
        user_tda: Option<int>,
    ) -> (result: ReadyThread)
        requires
            old(self).wf(),
            old(self).next_id.value < i32::MAX,
        ensures
            result.spec_id() == old(self).spec_next_id(),
            result.spec_kernel_stack() == kernel_stack,
            result.spec_user_stack() == user_stack,
            result.spec_user_tda() == user_tda,
            result.wf(),
            result.spec_drop_safe(),
            !result.spec_is_interrupted(),
            result.spec_locked_mutex_count() == 0,
            self.spec_next_id() == old(self).spec_next_id() + 1,
            self.wf(),
    {
        let id: ThreadIdentifier = self.next_id;
        self.next_id = ThreadIdentifier::from_i32(self.next_id.into_i32() + 1);
        ReadyThread::new(id, kernel_stack, user_stack, user_tda)
    }
```
</details>

#### 🔴 `create_thread` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,14 +1,5 @@
 {
         let id: ThreadIdentifier = self.next_id;
-        self.next_id = ThreadIdentifier::from(<i32>::from(self.next_id) + 1);
-
-        ReadyThread::new(
-            id,
-            kernel_stack,
-            user_stack,
-            user_tda,
-            context,
-            // SAFETY: calls to FpuState::new are synchronized.
-            unsafe { FpuState::new() },
-        )
+        self.next_id = ThreadIdentifier::from_i32(self.next_id.into_i32() + 1);
+        ReadyThread::new(id, kernel_stack, user_stack, user_tda)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        let id: ThreadIdentifier = self.next_id;
        self.next_id = ThreadIdentifier::from(<i32>::from(self.next_id) + 1);

        ReadyThread::new(
            id,
            kernel_stack,
            user_stack,
            user_tda,
            context,
            // SAFETY: calls to FpuState::new are synchronized.
            unsafe { FpuState::new() },
        )
    }
```

**Verus:**
```rust
{
        let id: ThreadIdentifier = self.next_id;
        self.next_id = ThreadIdentifier::from_i32(self.next_id.into_i32() + 1);
        ReadyThread::new(id, kernel_stack, user_stack, user_tda)
    }
```
</details>

#### 🔴 `init` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,5 +1,3 @@
 {
-    // TODO: check for double initialization.
-
     ThreadManager::new()
 }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
    // TODO: check for double initialization.

    ThreadManager::new()
}
```

**Verus:**
```rust
{
    ThreadManager::new()
}
```
</details>

## Module: mutex

- **Source:** `src/kernel/src/pm/sync/mutex.rs`
- **Verus:** `verus/split/kernel/pm/sync/mutex.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 8 critical, 9 high, 2 medium. Structs: 5 diffs. Functions: 14 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `MutexInner` | removed_struct | **critical** | Struct exists in source but missing in verus. |
| `MutexGuard` | removed_struct | **critical** | Struct exists in source but missing in verus. |
| `Mutex` | added_field | **critical** | New exec field added: pub locked: bool |
| `Mutex` | added_field | **critical** | New exec field added: pub id: usize |
| `Mutex` | added_field | **critical** | New exec field added: pub token_issued: bool |

### Function Differences

#### 🟠 `unlock_unchecked` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `reference_count` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `fmt` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `drop` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟡 `unlock` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `is_locked` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟠 `new` — signature_changed (high)

Parameters differ: source='()' vs verus='(id: usize)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn new() -> Self {
        Self(Arc::new(MutexInner {
            locked: AtomicBool::new(false),
            sleeping: Condvar::new(),
        }))
    }
```

**Verus:**
```rust
pub fn new(id: usize) -> (result: Self)
        ensures
            !result.locked,
            result.spec_is_unlocked(),
            result@ == Mutex::spec_new_view(id as nat),
            result@.id == id as nat,
            result.wf(),
    {
        Mutex { locked: false, id: id, token_issued: false }
    }
```
</details>

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,6 +1,3 @@
 {
-        Self(Arc::new(MutexInner {
-            locked: AtomicBool::new(false),
-            sleeping: Condvar::new(),
-        }))
+        Mutex { locked: false, id: id, token_issued: false }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        Self(Arc::new(MutexInner {
            locked: AtomicBool::new(false),
            sleeping: Condvar::new(),
        }))
    }
```

**Verus:**
```rust
{
        Mutex { locked: false, id: id, token_issued: false }
    }
```
</details>

#### 🟠 `try_lock` — signature_changed (high)

Parameters differ: source='(&self)' vs verus='(&mut self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn try_lock(&self) -> Result<MutexGuard, ()> {
        if self
            .0
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            Err(())
        } else {
            Ok(MutexGuard {
                mutex: self.0.clone(),
            })
        }
    }
```

**Verus:**
```rust
pub fn try_lock(&mut self) -> (result: (bool, Tracked<Option<MutexToken>>))
        requires
            old(self).wf(),
        ensures
            result.0 == !old(self).locked,
            // Unconditional: mutex is always held after try_lock (success: acquired;
            // failure: was already locked, state unchanged).
            self.locked,
            self@.id == old(self)@.id,
            !result.0 ==> self@ == old(self)@,
            result.0 ==> self@.token_issued,
            result.0 ==> result.1@.is_some(),
            result.0 ==> result.1@.unwrap().view == self@,
            !result.0 ==> result.1@.is_none(),
            !result.0 ==> self@.token_issued == old(self)@.token_issued,
            self.wf(),
    {
        if !self.locked {
            self.locked = true;
            self.token_issued = true;
            let tracked token: MutexToken = MutexToken { view: self@ };
            (true, Tracked(Some(token)))
        } else {
            (false, Tracked(None))
        }
    }
```
</details>

#### 🟠 `try_lock` — return_type_changed (high)

Return type differs: source='Result<MutexGuard, ()>' vs verus='(bool, Tracked<Option<MutexToken>>)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn try_lock(&self) -> Result<MutexGuard, ()> {
        if self
            .0
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            Err(())
        } else {
            Ok(MutexGuard {
                mutex: self.0.clone(),
            })
        }
    }
```

**Verus:**
```rust
pub fn try_lock(&mut self) -> (result: (bool, Tracked<Option<MutexToken>>))
        requires
            old(self).wf(),
        ensures
            result.0 == !old(self).locked,
            // Unconditional: mutex is always held after try_lock (success: acquired;
            // failure: was already locked, state unchanged).
            self.locked,
            self@.id == old(self)@.id,
            !result.0 ==> self@ == old(self)@,
            result.0 ==> self@.token_issued,
            result.0 ==> result.1@.is_some(),
            result.0 ==> result.1@.unwrap().view == self@,
            !result.0 ==> result.1@.is_none(),
            !result.0 ==> self@.token_issued == old(self)@.token_issued,
            self.wf(),
    {
        if !self.locked {
            self.locked = true;
            self.token_issued = true;
            let tracked token: MutexToken = MutexToken { view: self@ };
            (true, Tracked(Some(token)))
        } else {
            (false, Tracked(None))
        }
    }
```
</details>

#### 🔴 `try_lock` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,14 +1,10 @@
 {
-        if self
-            .0
-            .locked
-            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
-            .is_err()
-        {
-            Err(())
+        if !self.locked {
+            self.locked = true;
+            self.token_issued = true;
+            let tracked token: MutexToken = MutexToken { view: self@ };
+            (true, Tracked(Some(token)))
         } else {
-            Ok(MutexGuard {
-                mutex: self.0.clone(),
-            })
+            (false, Tracked(None))
         }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        if self
            .0
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            Err(())
        } else {
            Ok(MutexGuard {
                mutex: self.0.clone(),
            })
        }
    }
```

**Verus:**
```rust
{
        if !self.locked {
            self.locked = true;
            self.token_issued = true;
            let tracked token: MutexToken = MutexToken { view: self@ };
            (true, Tracked(Some(token)))
        } else {
            (false, Tracked(None))
        }
    }
```
</details>

#### 🟠 `lock` — signature_changed (high)

Parameters differ: source='(&self, timeout: Option<SystemTime>)' vs verus='(&mut self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub unsafe fn lock(&self, timeout: Option<SystemTime>) -> Result<MutexGuard, SleepError> {
        loop {
            // Attempt to acquire the mutex.
            match self.try_lock() {
                // Success.
                Ok(guard) => break Ok(guard),
                // Failed to acquire the mutex.
                Err(()) => {
                    self.0.sleeping.wait(timeout)?;
                },
            }
        }
    }
```

**Verus:**
```rust
pub fn lock(&mut self) -> (token: Tracked<MutexToken>)
        requires
            old(self).spec_is_unlocked(),
            old(self).wf(),
            !old(self).token_issued(),
        ensures
            self.locked,
            self.spec_is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.wf(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: MutexToken = opt_token.tracked_unwrap();
        Tracked(token)
    }
```
</details>

#### 🟠 `lock` — return_type_changed (high)

Return type differs: source='Result<MutexGuard, SleepError>' vs verus='Tracked<MutexToken>'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub unsafe fn lock(&self, timeout: Option<SystemTime>) -> Result<MutexGuard, SleepError> {
        loop {
            // Attempt to acquire the mutex.
            match self.try_lock() {
                // Success.
                Ok(guard) => break Ok(guard),
                // Failed to acquire the mutex.
                Err(()) => {
                    self.0.sleeping.wait(timeout)?;
                },
            }
        }
    }
```

**Verus:**
```rust
pub fn lock(&mut self) -> (token: Tracked<MutexToken>)
        requires
            old(self).spec_is_unlocked(),
            old(self).wf(),
            !old(self).token_issued(),
        ensures
            self.locked,
            self.spec_is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.wf(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: MutexToken = opt_token.tracked_unwrap();
        Tracked(token)
    }
```
</details>

#### 🔴 `lock` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,13 +1,5 @@
 {
-        loop {
-            // Attempt to acquire the mutex.
-            match self.try_lock() {
-                // Success.
-                Ok(guard) => break Ok(guard),
-                // Failed to acquire the mutex.
-                Err(()) => {
-                    self.0.sleeping.wait(timeout)?;
-                },
-            }
-        }
+        let (_success, Tracked(opt_token)) = self.try_lock();
+        let tracked token: MutexToken = opt_token.tracked_unwrap();
+        Tracked(token)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        loop {
            // Attempt to acquire the mutex.
            match self.try_lock() {
                // Success.
                Ok(guard) => break Ok(guard),
                // Failed to acquire the mutex.
                Err(()) => {
                    self.0.sleeping.wait(timeout)?;
                },
            }
        }
    }
```

**Verus:**
```rust
{
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: MutexToken = opt_token.tracked_unwrap();
        Tracked(token)
    }
```
</details>

## Module: semaphore

- **Source:** `src/kernel/src/pm/sync/semaphore.rs`
- **Verus:** `verus/split/kernel/pm/sync/semaphore.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 3 critical, 6 high, 5 medium. Structs: 2 diffs. Functions: 12 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `Semaphore` | visibility_changed | **medium** | Field 'value' changed from private to pub (Verus constraint). |
| `Semaphore` | type_changed | **high** | Field 'value' type changed: 'AtomicUsize' → 'usize'. |

### Function Differences

#### 🟠 `down` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟡 `down_available` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `down_or_block` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `get_value` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `is_available` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,6 +1,3 @@
 {
-        Self {
-            value: AtomicUsize::new(value),
-            sleeping: Condvar::new(),
-        }
+        Semaphore { value: value }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        Self {
            value: AtomicUsize::new(value),
            sleeping: Condvar::new(),
        }
    }
```

**Verus:**
```rust
{
        Semaphore { value: value }
    }
```
</details>

#### 🟠 `try_down` — signature_changed (high)

Parameters differ: source='(&self)' vs verus='(&mut self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn try_down(&self) -> Result<(), Error> {
        if self
            .value
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                if value == 0 {
                    None
                } else {
                    Some(value - 1)
                }
            })
            .is_ok()
        {
            return Ok(());
        }

        Err(Error::new(ErrorCode::TryAgain, "semaphore is busy"))
    }
```

**Verus:**
```rust
pub fn try_down(&mut self) -> (result: bool)
        requires
            old(self).wf(),
        ensures
            result == old(self).spec_is_available(),
            result ==> self.value == old(self).value - 1,
            result ==> self@.value == old(self)@.value - 1,
            !result ==> self@ == old(self)@,
            !result ==> self.spec_is_exhausted(),
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        if self.value > 0 {
            self.value = self.value - 1;
            true
        } else {
            false
        }
    }
```
</details>

#### 🟠 `try_down` — return_type_changed (high)

Return type differs: source='Result<(), Error>' vs verus='bool'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn try_down(&self) -> Result<(), Error> {
        if self
            .value
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                if value == 0 {
                    None
                } else {
                    Some(value - 1)
                }
            })
            .is_ok()
        {
            return Ok(());
        }

        Err(Error::new(ErrorCode::TryAgain, "semaphore is busy"))
    }
```

**Verus:**
```rust
pub fn try_down(&mut self) -> (result: bool)
        requires
            old(self).wf(),
        ensures
            result == old(self).spec_is_available(),
            result ==> self.value == old(self).value - 1,
            result ==> self@.value == old(self)@.value - 1,
            !result ==> self@ == old(self)@,
            !result ==> self.spec_is_exhausted(),
            self@.waiters == old(self)@.waiters,
            self.wf(),
    {
        if self.value > 0 {
            self.value = self.value - 1;
            true
        } else {
            false
        }
    }
```
</details>

#### 🔴 `try_down` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,17 +1,8 @@
 {
-        if self
-            .value
-            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
-                if value == 0 {
-                    None
-                } else {
-                    Some(value - 1)
-                }
-            })
-            .is_ok()
-        {
-            return Ok(());
+        if self.value > 0 {
+            self.value = self.value - 1;
+            true
+        } else {
+            false
         }
-
-        Err(Error::new(ErrorCode::TryAgain, "semaphore is busy"))
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        if self
            .value
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                if value == 0 {
                    None
                } else {
                    Some(value - 1)
                }
            })
            .is_ok()
        {
            return Ok(());
        }

        Err(Error::new(ErrorCode::TryAgain, "semaphore is busy"))
    }
```

**Verus:**
```rust
{
        if self.value > 0 {
            self.value = self.value - 1;
            true
        } else {
            false
        }
    }
```
</details>

#### 🟠 `up` — signature_changed (high)

Parameters differ: source='(&self)' vs verus='(&mut self, ctx: Ghost<CallerContext>)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub unsafe fn up(&self) -> Result<(), Error> {
        self.value.fetch_add(1, Ordering::SeqCst);
        self.sleeping.notify_first().map(|_awakened| ())
    }
```

**Verus:**
```rust
pub fn up(&mut self, ctx: Ghost<CallerContext>)
        requires
            old(self).wf(),
            old(self).value < usize::MAX,
            ctx@.safe_for_up(),
        ensures
            self.value == old(self).value + 1,
            self@.value == old(self)@.value + 1,
            self@.waiters == old(self)@.waiters,
            self.spec_is_available(),
            self.wf(),
    {
        self.value = self.value + 1;
    }
```
</details>

#### 🟠 `up` — return_type_changed (high)

Return type differs: source='Result<(), Error>' vs verus='None'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub unsafe fn up(&self) -> Result<(), Error> {
        self.value.fetch_add(1, Ordering::SeqCst);
        self.sleeping.notify_first().map(|_awakened| ())
    }
```

**Verus:**
```rust
pub fn up(&mut self, ctx: Ghost<CallerContext>)
        requires
            old(self).wf(),
            old(self).value < usize::MAX,
            ctx@.safe_for_up(),
        ensures
            self.value == old(self).value + 1,
            self@.value == old(self)@.value + 1,
            self@.waiters == old(self)@.waiters,
            self.spec_is_available(),
            self.wf(),
    {
        self.value = self.value + 1;
    }
```
</details>

#### 🔴 `up` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,4 +1,3 @@
 {
-        self.value.fetch_add(1, Ordering::SeqCst);
-        self.sleeping.notify_first().map(|_awakened| ())
+        self.value = self.value + 1;
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.value.fetch_add(1, Ordering::SeqCst);
        self.sleeping.notify_first().map(|_awakened| ())
    }
```

**Verus:**
```rust
{
        self.value = self.value + 1;
    }
```
</details>

## Module: spinlock

- **Source:** `src/kernel/src/pm/sync/spinlock.rs`
- **Verus:** `verus/split/kernel/pm/sync/spinlock.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 6 critical, 4 high, 3 medium. Structs: 4 diffs. Functions: 9 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `SpinlockGuard` | removed_struct | **critical** | Struct exists in source but missing in verus. |
| `Spinlock` | added_field | **critical** | New exec field added: pub locked: bool |
| `Spinlock` | added_field | **critical** | New exec field added: pub id: usize |
| `Spinlock` | added_field | **critical** | New exec field added: pub token_issued: bool |

### Function Differences

#### 🟠 `drop` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟡 `try_lock` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `unlock` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `is_locked` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟠 `new` — signature_changed (high)

Parameters differ: source='()' vs verus='(id: usize)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }
```

**Verus:**
```rust
pub fn new(id: usize) -> (result: Self)
        ensures
            !result.locked,
            result.spec_is_unlocked(),
            result@ == Spinlock::spec_new_view(id as nat),
            result@.id == id as nat,
            result.wf(),
    {
        Spinlock { locked: false, id: id, token_issued: false }
    }
```
</details>

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,3 @@
 {
-        Self(AtomicBool::new(false))
+        Spinlock { locked: false, id: id, token_issued: false }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        Self(AtomicBool::new(false))
    }
```

**Verus:**
```rust
{
        Spinlock { locked: false, id: id, token_issued: false }
    }
```
</details>

#### 🟠 `lock` — signature_changed (high)

Parameters differ: source='(&self)' vs verus='(&mut self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn lock(&self) -> SpinlockGuard {
        loop {
            match self
                .0
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(false) => break,
                _ => ::arch::cpu::pause(),
            }
        }

        SpinlockGuard(self)
    }
```

**Verus:**
```rust
pub fn lock(&mut self) -> (token: Tracked<LockToken>)
        requires
            old(self).spec_is_unlocked(),
            old(self).wf(),
            !old(self).token_issued(),
        ensures
            self.locked,
            self.spec_is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.wf(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: LockToken = opt_token.tracked_unwrap();
        Tracked(token)
    }
```
</details>

#### 🟠 `lock` — return_type_changed (high)

Return type differs: source='SpinlockGuard' vs verus='Tracked<LockToken>'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn lock(&self) -> SpinlockGuard {
        loop {
            match self
                .0
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(false) => break,
                _ => ::arch::cpu::pause(),
            }
        }

        SpinlockGuard(self)
    }
```

**Verus:**
```rust
pub fn lock(&mut self) -> (token: Tracked<LockToken>)
        requires
            old(self).spec_is_unlocked(),
            old(self).wf(),
            !old(self).token_issued(),
        ensures
            self.locked,
            self.spec_is_locked(),
            self@.id == old(self)@.id,
            self@.token_issued,
            token@.view == self@,
            self.wf(),
    {
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: LockToken = opt_token.tracked_unwrap();
        Tracked(token)
    }
```
</details>

#### 🔴 `lock` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,13 +1,5 @@
 {
-        loop {
-            match self
-                .0
-                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
-            {
-                Ok(false) => break,
-                _ => ::arch::cpu::pause(),
-            }
-        }
-
-        SpinlockGuard(self)
+        let (_success, Tracked(opt_token)) = self.try_lock();
+        let tracked token: LockToken = opt_token.tracked_unwrap();
+        Tracked(token)
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        loop {
            match self
                .0
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(false) => break,
                _ => ::arch::cpu::pause(),
            }
        }

        SpinlockGuard(self)
    }
```

**Verus:**
```rust
{
        let (_success, Tracked(opt_token)) = self.try_lock();
        let tracked token: LockToken = opt_token.tracked_unwrap();
        Tracked(token)
    }
```
</details>

## Module: condvar

- **Source:** `src/kernel/src/pm/sync/condvar.rs`
- **Verus:** `verus/split/kernel/pm/sync/condvar.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 4 critical, 8 high, 12 medium. Structs: 3 diffs. Functions: 21 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `CondvarInner` | removed_struct | **critical** | Struct exists in source but missing in verus. |
| `Condvar` | added_field | **critical** | New exec field added: pub len: usize |
| `Condvar` | added_field | **critical** | New exec field added: pub sleeping: Vec<(i32, i32)> |

### Function Differences

#### 🟠 `reference_count` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `notify_first` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `notify_process` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `notify_thread` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `notify_all` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `wait` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `fmt` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟠 `drop` — missing_in_verus (high)

Function exists in source but not in verus exec code.

#### 🟡 `enqueue` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `try_enqueue` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `dequeue_first` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `remove_at` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `remove_entry` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `remove_by_pid` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `remove_by_tid` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `try_remove_by_pid` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `try_remove_by_tid` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `clear` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `is_empty` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `get_len` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,7 +1,3 @@
 {
-        Self {
-            inner: Arc::new(CondvarInner {
-                sleeping: RefCell::new(LinkedList::new()),
-            }),
-        }
+        Condvar { len: 0, sleeping: Vec::new() }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        Self {
            inner: Arc::new(CondvarInner {
                sleeping: RefCell::new(LinkedList::new()),
            }),
        }
    }
```

**Verus:**
```rust
{
        Condvar { len: 0, sleeping: Vec::new() }
    }
```
</details>

## Module: fence

- **Source:** `src/kernel/src/pm/sync/fence.rs`
- **Verus:** `verus/split/kernel/pm/sync/fence.rs`
- **Tree Hash Match:** ❌ No
- **Summary:** Tree hash MISMATCH. Issues: 3 critical, 2 high, 5 medium. Structs: 3 diffs. Functions: 7 diffs.

### Struct Differences

| Struct | Type | Severity | Detail |
|--------|------|----------|--------|
| `Fence` | visibility_changed | **medium** | Field 'count' changed from private to pub (Verus constraint). |
| `Fence` | type_changed | **high** | Field 'count' type changed: 'AtomicUsize' → 'usize'. |
| `Fence` | visibility_changed | **medium** | Field 'total' changed from private to pub (Verus constraint). |

### Function Differences

#### 🟡 `is_satisfied` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `get_count` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🟡 `get_total` — added_in_verus (medium)

New exec function in verus (not in source). Modifiers: []

#### 🔴 `new` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,6 +1,3 @@
 {
-        Self {
-            count: AtomicUsize::new(0),
-            total,
-        }
+        Fence { count: 0, total }
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        Self {
            count: AtomicUsize::new(0),
            total,
        }
    }
```

**Verus:**
```rust
{
        Fence { count: 0, total }
    }
```
</details>

#### 🔴 `wait` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,5 +1,4 @@
 {
-        while self.count.load(Ordering::Acquire) < self.total {
-            ::arch::cpu::pause();
-        }
+        // In the sequential model, the precondition guarantees satisfaction,
+        // so the spin loop body is never entered.
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        while self.count.load(Ordering::Acquire) < self.total {
            ::arch::cpu::pause();
        }
    }
```

**Verus:**
```rust
{
        // In the sequential model, the precondition guarantees satisfaction,
        // so the spin loop body is never entered.
    }
```
</details>

#### 🟠 `signal` — signature_changed (high)

Parameters differ: source='(&self)' vs verus='(&mut self)'

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
pub fn signal(&self) {
        self.count.fetch_add(1, Ordering::Release);
    }
```

**Verus:**
```rust
pub fn signal(&mut self)
        requires
            old(self).wf(),
            old(self).spec_is_waiting(),
        ensures
            self.count == old(self).count + 1,
            self.total == old(self).total,
            self.spec_count() == old(self).spec_count() + 1,
            self.spec_total() == old(self).spec_total(),
            self.wf(),
            old(self).spec_remaining() > 0 ==> self.spec_remaining() == old(self).spec_remaining() - 1,
            self.spec_remaining() == 0 ==> self.spec_is_satisfied(),
    {
        self.count = self.count + 1;
    }
```
</details>

#### 🔴 `signal` — body_changed_semantic (critical)

Body differs semantically. Diff:
--- source
+++ verus
@@ -1,3 +1,3 @@
 {
-        self.count.fetch_add(1, Ordering::Release);
+        self.count = self.count + 1;
     }

<details>
<summary>Source vs Verus code</summary>

**Source:**
```rust
{
        self.count.fetch_add(1, Ordering::Release);
    }
```

**Verus:**
```rust
{
        self.count = self.count + 1;
    }
```
</details>

---
*Generated by `scripts/verus_exec_diff.py` using tree-sitter-verus.*