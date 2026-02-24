# Exec Consistency Fix: raw_array

## Summary
- Mismatches fixed: 0
- Missing functions added: 0
- Documented equivalences: 9

## Changes
| Function | Action | Justification |
|----------|--------|---------------|
| `new_managed` [diff](new_managed.diff) [source](new_managed_source.rs) [verus](new_managed_verus.rs) | EQUIVALENT | Identical exec logic in both files. Source (lines 79-105) and verus (lines 36-53) both: validate `len == 0 \|\| len >= i32::MAX`, allocate via `Layout::array::<T>(len)`, create `NonNull`, zero-initialize with `ptr::write_bytes`, and return `RawArrayStorage::Managed { ptr, len }`. Only difference is removal of comments in verus. |
| `new_unmanaged` [diff](new_unmanaged.diff) [source](new_unmanaged_source.rs) [verus](new_unmanaged_verus.rs) | EQUIVALENT | Identical exec logic. Source (lines 130-151) and verus (lines 55-68) both: validate length, check wrapping with `ptr.wrapping_add(len) < ptr`, create `NonNull`, zero-initialize with `ptr::write_bytes`, and return `RawArrayStorage::Unmanaged { ptr, len }`. Only difference is removal of comments. |
| `new` [diff](new.diff) [source](new_source.rs) [verus](new_verus.rs) | EQUIVALENT | Identical exec logic: `Ok(RawArray { storage: RawArrayStorage::new_managed(len)? })`. Only difference is added Verus requires/ensures annotations. |
| `from_raw_parts` [diff](from_raw_parts.diff) [source](from_raw_parts_source.rs) [verus](from_raw_parts_verus.rs) | EQUIVALENT | Identical exec logic: `Ok(RawArray { storage: RawArrayStorage::new_unmanaged(ptr, len)? })`. Only difference is added Verus requires/ensures annotations. |
| `get` [diff](get.diff) [source](get_source.rs) [verus](get_verus.rs) | EQUIVALENT | False-positive mismatch. The AST tool incorrectly paired source's `RawArrayStorage::get(&self) -> &[T]` (lines 182-191) with verus's `RawArray::get(&self, index) -> &T` (lines 219-226), which are different functions. Source's `RawArrayStorage::get()` exists identically in verus at lines 81-90 with the same `match` on `Managed`/`Unmanaged` calling `slice::from_raw_parts`. The verus `RawArray::get(index)` is an extra accessor (see EXTRA_IN_VERUS below). |
| `deref` [diff](deref.diff) [source](deref_source.rs) [verus](deref_verus.rs) | EQUIVALENT | Identical exec logic: both return `self.storage.get()`. Verus adds `ensures result@ == self@` annotation. |
| `deref_mut` [source](deref_mut_source.rs) | DOCUMENTED | `DerefMut` trait implementation is missing in verus. This is a Verus limitation: Verus does not support mutable `Deref`/`DerefMut` trait impls within `verus!` blocks. The verus version provides an explicit `set(index, value)` method instead, which replaces the `deref_mut` [source](deref_mut_source.rs)-based element assignment pattern `array[i] = value` with `array.set(i, value)`. |
| `drop` [diff](drop.diff) [source](drop_source.rs) [verus](drop_verus.rs) | EQUIVALENT | Semantically identical. Source implements `Drop for RawArray<T>` (lines 275-288) matching on `self.storage`; verus implements `Drop for RawArrayStorage<T>` (lines 100-110) matching on `self`. Moving Drop to `RawArrayStorage` is equivalent because when `RawArray` is dropped, its `storage` field is dropped, triggering the same dealloc. Source uses `match Layout::array::<T>(*len) { Ok(layout) => layout, Err(_) => return }` followed by `dealloc`; verus uses `if let Ok(layout) = Layout::array::<T>(*len) { dealloc... }` — algebraically identical control flow. |
| `get_mut` | EQUIVALENT | Already marked MATCH by AST tool. Identical exec logic in both files. |

## Extra Functions in Verus (EXTRA_IN_VERUS)
| Function | Action | Justification |
|----------|--------|---------------|
| `set` [verus](set_verus.rs) | EXTRA_JUSTIFIED | Explicit setter `self.storage.get_mut()[index] = value` needed because Verus cannot use `DerefMut` trait. Replaces `array[i] = value` pattern that source achieves via `DerefMut`. |
| `len` [verus](len_verus.rs) | EXTRA_JUSTIFIED | Explicit length accessor returning `self.storage.storage_len()`. Needed for verification because Verus cannot call `.len()` on a `Deref` target slice in spec context. Source achieves this via `Deref` to `&[T]` then `.len()`. |
| `storage_len` [verus](storage_len_verus.rs) | EXTRA_JUSTIFIED | Helper on `RawArrayStorage` returning the length field. Supports `RawArray::len()`. Trivial getter with `match self { Managed { len, .. } => *len, Unmanaged { len, .. } => *len }`. |
| `from_raw_addr` [verus](from_raw_addr_verus.rs) | EXTRA_JUSTIFIED | Convenience constructor that casts a `usize` address to `*mut T` and delegates to `from_raw_parts` [diff](from_raw_parts.diff) [source](from_raw_parts_source.rs) [verus](from_raw_parts_verus.rs). Needed for verification scenarios where callers work with raw addresses rather than typed pointers (e.g., memory-mapped I/O regions). |

## Extra Structs in Verus (EXTRA_IN_VERUS)
| Struct | Action | Justification |
|--------|--------|---------------|
| `ExRawArrayStorage` | EXTRA_JUSTIFIED | Verus `#[verifier::external_type_specification]` wrapper required to make the non-verus `RawArrayStorage` type usable inside `verus!` blocks. This is a standard Verus pattern for external types. |

## Verification: PASS
