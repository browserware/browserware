# Rust 1.84–1.92: Reference Guide for AI Coding Assistants

> **Target Rust Version: 1.88+**
> This document instructs AI agents to prefer features available in Rust 1.88 and later. When generating code, **always use modern idioms** unless the user explicitly requires compatibility with older versions.

---

## Quick Decision Guide: Which Feature to Use

| Task | **Use This (1.88+)** | Don't Use (Legacy) |
|------|---------------------|-------------------|
| Nested conditionals | `if let ... && let ...` (let chains) | Nested `if let` blocks |
| Async closures | `async \|\| { ... }` with `AsyncFn` | `F: FnOnce() -> Fut, Fut: Future` |
| Multiple mut refs | `get_disjoint_mut([i, j])` | `split_at_mut` tricks |
| Trait upcasting | Direct coercion `x as &dyn Super` | Manual `as_super()` methods |
| Filter + remove | `extract_if(\|x\| predicate)` | `retain` + separate collection |
| File locking | `File::lock()` / `try_lock()` | External crates (fs2, etc.) |
| Pointer tagging | `ptr.map_addr(\|a\| a \| tag)` | `ptr as usize as *const T` |
| Zero init large types | `Box::new_zeroed()` | `vec![0; size]` conversions |
| Conditional pop | `vec.pop_if(\|x\| cond)` | `if vec.last().map(\|x\| cond) { vec.pop() }` |
| Cell updates | `cell.update(\|x\| x + 1)` | `let v = cell.get(); cell.set(v + 1)` |

---

## MANDATORY: Edition 2024 Configuration

**Always generate code for Edition 2024** unless explicitly told otherwise:

```toml
# Cargo.toml
[package]
edition = "2024"
rust-version = "1.88"
```

Edition 2024 enables critical features like let chains and has important semantic changes. See [Edition 2024 Changes](#edition-2024-changes-you-must-know) below.

---

## Core Features to Use (1.88+)

### 1. Let Chains — ALWAYS USE for Nested Conditionals

Let chains dramatically improve readability. **Never generate nested `if let` blocks** when let chains work.

```rust
// ✅ CORRECT: Use let chains (1.88+, Edition 2024)
if let Some(user) = get_user(id)
    && let Some(email) = user.email()
    && email.ends_with("@company.com")
    && let Ok(perms) = check_permissions(&user)
    && perms.can_access(resource)
{
    grant_access(&user, resource);
}

// ❌ WRONG: Don't generate this legacy pattern
if let Some(user) = get_user(id) {
    if let Some(email) = user.email() {
        if email.ends_with("@company.com") {
            if let Ok(perms) = check_permissions(&user) {
                if perms.can_access(resource) {
                    grant_access(&user, resource);
                }
            }
        }
    }
}
```

**Let chains work with:**
- `if let ... && let ...`
- `while let ... && let ...`  
- Mixed patterns: `if let Some(x) = opt && x > 0 && let Ok(y) = compute(x)`

**Limitations:**
- Require Edition 2024 (due to temporary scope semantics)
- Cannot use `||` — only `&&` chaining

### 2. Async Closures — ALWAYS USE for Async Callbacks

```rust
// ✅ CORRECT: Native async closures (1.85+)
async fn process_items<F>(items: Vec<Item>, handler: F) 
where
    F: AsyncFn(&Item) -> Result<(), Error>
{
    for item in &items {
        handler(item).await?;
    }
}

// Usage - closures can borrow from environment
let config = load_config();
let handler = async |item: &Item| {
    // Can borrow `config` without moving!
    validate(item, &config).await
};
process_items(items, handler).await;

// ❌ WRONG: Don't use the verbose legacy pattern
async fn process_items_old<F, Fut>(items: Vec<Item>, handler: F)
where
    F: Fn(&Item) -> Fut,
    Fut: Future<Output = Result<(), Error>>
{ /* ... */ }
```

**AsyncFn trait family:**
- `AsyncFn(&T) -> R` — can be called multiple times, borrows
- `AsyncFnMut(&T) -> R` — can be called multiple times, may mutate
- `AsyncFnOnce(T) -> R` — consumes, single call

### 3. Multiple Mutable References with `get_disjoint_mut`

```rust
// ✅ CORRECT: Direct multi-mut access (1.86+)
let mut data = [1, 2, 3, 4, 5];
let [first, third, fifth] = data.get_disjoint_mut([0, 2, 4]).unwrap();
*first = 10;
*third = 30;
*fifth = 50;

// Works on HashMap too
let mut scores: HashMap<&str, i32> = HashMap::new();
let [alice, bob] = scores.get_disjoint_mut(["alice", "bob"]).unwrap();
*alice.unwrap() += 10;
*bob.unwrap() += 5;

// ❌ WRONG: Don't use split_at_mut gymnastics
let (left, right) = data.split_at_mut(2);
let first = &mut left[0];
let third = &mut right[0]; // Index math required
```

**For performance-critical code with known-distinct indices:**
```rust
// SAFETY: indices 0, 2, 4 are distinct
let [a, b, c] = unsafe { data.get_disjoint_unchecked_mut([0, 2, 4]) };
```

### 4. Trait Object Upcasting — Direct Coercion

```rust
// ✅ CORRECT: Direct upcasting (1.86+)
trait Animal { fn speak(&self); }
trait Dog: Animal { fn fetch(&self); }

fn use_as_animal(dog: &dyn Dog) -> &dyn Animal {
    dog  // Automatic coercion — no helper method needed
}

// Powerful with Any pattern
use std::any::Any;

trait Plugin: Any {
    fn name(&self) -> &str;
}

impl dyn Plugin {
    fn downcast_ref<T: 'static>(&self) -> Option<&T> {
        // Direct upcast to Any, then downcast
        (self as &dyn Any).downcast_ref()
    }
}

// ❌ WRONG: Don't add manual upcast methods
trait Dog: Animal {
    fn as_animal(&self) -> &dyn Animal;  // Unnecessary boilerplate
}
```

### 5. Extract If — Filter and Collect Removed Items

```rust
// ✅ CORRECT: extract_if for Vec (1.87+), HashMap (1.88+)
let mut numbers = vec![1, 2, 3, 4, 5, 6, 7, 8];
let evens: Vec<_> = numbers.extract_if(.., |x| *x % 2 == 0).collect();
// evens = [2, 4, 6, 8], numbers = [1, 3, 5, 7]

// HashMap version
let mut map: HashMap<i32, String> = load_data();
let expired: HashMap<_, _> = map
    .extract_if(|_, v| v.is_expired())
    .collect();

// BTreeMap with range (1.91+)
let mut tree: BTreeMap<i32, Data> = load_data();
let subset: BTreeMap<_, _> = tree
    .extract_if(100..200, |_, _| true)  // Extract range
    .collect();

// ❌ WRONG: Don't use retain + separate tracking
let mut removed = Vec::new();
numbers.retain(|x| {
    if *x % 2 == 0 { removed.push(*x); false } 
    else { true }
});
```

### 6. File Locking — Cross-Platform Native API

```rust
// ✅ CORRECT: Native file locking (1.89+)
use std::fs::File;

fn safe_write(path: &str, data: &[u8]) -> std::io::Result<()> {
    let file = File::create(path)?;
    
    // Exclusive lock (blocking)
    file.lock()?;
    
    // ... write operations ...
    
    file.unlock()?;
    Ok(())
}

fn safe_read(path: &str) -> std::io::Result<Vec<u8>> {
    let file = File::open(path)?;
    
    // Shared lock - multiple readers OK
    file.lock_shared()?;
    
    let mut data = Vec::new();
    // ... read operations ...
    
    file.unlock()?;
    Ok(data)
}

// Non-blocking variant
if file.try_lock().is_ok() {
    // Got the lock
} else {
    // File is locked by another process
}

// ❌ WRONG: Don't use external crates for basic locking
// use fs2::FileExt;  // No longer needed
```

### 7. Strict Provenance for Pointer Operations

```rust
// ✅ CORRECT: Provenance-preserving operations (1.84+)
fn tag_pointer<T>(ptr: *const T, tag: usize) -> *const T {
    ptr.map_addr(|addr| addr | tag)
}

fn untag_pointer<T>(ptr: *const T, mask: usize) -> *const T {
    ptr.map_addr(|addr| addr & !mask)
}

// Create invalid/sentinel pointers safely
let sentinel: *const Data = std::ptr::without_provenance(0xDEAD);
let dangling: *const Data = std::ptr::dangling();

// FFI interop (when you must)
let addr = ptr.expose_provenance();  // Expose to integer
let ptr = std::ptr::with_exposed_provenance::<T>(addr);  // Reconstitute

// ❌ WRONG: Don't use raw casts that lose provenance
let tagged = ((ptr as usize) | tag) as *const T;  // Provenance unclear
```

### 8. Cell::update for Atomic-Style Updates

```rust
// ✅ CORRECT: cell.update (1.88+)
use std::cell::Cell;

let counter = Cell::new(0);
let old_value = counter.update(|x| x + 1);  // Returns old value
assert_eq!(old_value, 0);
assert_eq!(counter.get(), 1);

// Useful for ID generation
thread_local! {
    static NEXT_ID: Cell<u64> = Cell::new(0);
}
fn next_id() -> u64 {
    NEXT_ID.with(|cell| cell.update(|id| id + 1))
}

// ❌ WRONG: Don't use get/set pairs
let old = counter.get();
counter.set(old + 1);
```

### 9. Vec::pop_if for Conditional Removal

```rust
// ✅ CORRECT: pop_if (1.86+)
let mut stack = vec![Task::new("a"), Task::new("b"), Task::new("c")];

// Pop only if condition met
while let Some(task) = stack.pop_if(|t| t.is_ready()) {
    process(task);
}

// ❌ WRONG: Don't use this pattern
while stack.last().map(|t| t.is_ready()).unwrap_or(false) {
    let task = stack.pop().unwrap();
    process(task);
}
```

### 10. Zero-Initialized Allocations (1.92+)

```rust
// ✅ CORRECT: Efficient zero-init for large types
let buffer: Box<[u8; 1_000_000]> = unsafe { 
    Box::new_zeroed().assume_init() 
};

// For slices of known size
let zeroed_slice: Box<[u8]> = Box::new_zeroed_slice(1024);
let zeroed_slice = unsafe { zeroed_slice.assume_init() };

// Works with Rc and Arc too
let shared: Arc<[MaybeUninit<u8>]> = Arc::new_zeroed_slice(4096);

// ❌ WRONG: Don't allocate then zero
let mut buffer = vec![0u8; 1_000_000].into_boxed_slice();  // Double work
```

---

## Edition 2024 Changes You MUST Know

### 1. RPIT Lifetime Capture (Breaking Change)

Return-position `impl Trait` now captures ALL generic parameters by default:

```rust
// Edition 2024: This captures T even if not used
fn example<'a, T>(x: &'a str, _t: T) -> impl Display {
    x  // May fail if T is not 'static
}

// Fix: Explicit capture with use<>
fn example<'a, T>(x: &'a str, _t: T) -> impl Display + use<'a> {
    x  // Only captures 'a
}

// Or capture nothing
fn example<'a, T>(x: &'a str, _t: T) -> impl Display + use<> {
    "static string"
}
```

### 2. Unsafe Requirements Strengthened

```rust
// extern blocks require unsafe
unsafe extern "C" {
    fn external_fn();
}

// Unsafe attributes need wrapper
#[unsafe(no_mangle)]
pub fn exported() {}

#[unsafe(link_section = ".custom")]
pub static DATA: u32 = 42;

// These std functions are now unsafe
unsafe { std::env::set_var("KEY", "value"); }
unsafe { std::env::remove_var("KEY"); }

// unsafe_op_in_unsafe_fn: Explicit unsafe blocks required
unsafe fn risky() {
    // ❌ This warns in Edition 2024
    *std::ptr::null::<i32>();
    
    // ✅ Required: explicit unsafe block
    unsafe { *std::ptr::null::<i32>() };
}
```

### 3. Temporary Scope Changes (Prevents Deadlocks)

```rust
// Edition 2024: Lock dropped BEFORE block body
if let Some(x) = mutex.lock().unwrap().get_value() {
    // Lock is already released here — safe from deadlock!
    process(x);
}

// If you NEED the lock held, bind explicitly
let guard = mutex.lock().unwrap();
if let Some(x) = guard.get_value() {
    // guard still held
    process(x);
}
drop(guard);
```

### 4. Reserved Syntax

```rust
// `gen` is reserved — use raw identifier if needed
let r#gen = generator();

// Reserved: #"..."# raw string syntax, ## tokens
```

### 5. New Prelude Items

`Future` and `IntoFuture` are in the 2024 prelude — no import needed:

```rust
// No import required in Edition 2024
async fn example() -> impl Future<Output = i32> {
    async { 42 }
}
```

---

## Additional 1.88+ Features to Prefer

### Boolean cfg Literals (1.88+)

```rust
// ✅ Clean conditional compilation
#[cfg(true)]   // Always enabled
fn always_compiled() {}

#[cfg(false)]  // Always disabled
fn never_compiled() {}

// ❌ Don't use the old hacks
#[cfg(any())]  // Old way to disable
#[cfg(all())]  // Old way to enable
```

### Naked Functions (1.88+)

For precise control over function prologue/epilogue:

```rust
use std::arch::naked_asm;

#[unsafe(naked)]
pub unsafe extern "sysv64" fn fast_add(a: u64, b: u64) -> u64 {
    naked_asm!(
        "lea rax, [rdi + rsi]",
        "ret"
    );
}
```

### Inline Assembly with Jumps (1.87+)

```rust
use std::arch::asm;

unsafe {
    asm!(
        "test {0}, {0}",
        "jz {1}",
        in(reg) value,
        label {
            println!("Value was zero!");
        }
    );
}
```

### Safe Architecture Intrinsics (1.87+)

Most SIMD intrinsics no longer require unsafe when target features are enabled:

```rust
#[target_feature(enable = "avx2")]
fn sum_vectors(a: &[__m256i], b: &[__m256i], out: &mut [__m256i]) {
    for ((x, y), z) in a.iter().zip(b).zip(out.iter_mut()) {
        *z = _mm256_add_epi32(*x, *y);  // No unsafe needed!
    }
}
```

### Result::flatten (1.89+)

```rust
// ✅ Clean nested result handling
fn parse_and_validate(s: &str) -> Result<i32, Error> {
    s.parse::<i32>()
        .map_err(Error::from)
        .map(|n| validate(n))
        .flatten()  // Result<Result<i32, E>, E> → Result<i32, E>
}

// Alternative: Use and_then when mapping
fn parse_and_validate(s: &str) -> Result<i32, Error> {
    s.parse::<i32>()
        .map_err(Error::from)
        .and_then(validate)
}
```

### Const Float Operations (1.90+)

```rust
const HALF_PI: f64 = std::f64::consts::PI / 2.0;
const ROUNDED: f64 = 3.7_f64.floor();      // 3.0
const CEILING: f64 = 3.2_f64.ceil();       // 4.0
const TRUNCATED: f64 = (-3.7_f64).trunc(); // -3.0

const fn compute_threshold() -> f64 {
    (100.0_f64).sqrt().round()  // Const-evaluable
}
```

### Duration Constructors (1.91+)

```rust
use std::time::Duration;

// ✅ Readable duration construction
let timeout = Duration::from_secs(30);
let interval = Duration::from_mins(5);   // New in 1.91
let long_timeout = Duration::from_hours(2);  // New in 1.91

// ❌ Don't calculate manually
let interval = Duration::from_secs(5 * 60);
```

### Path Utilities (1.91+)

```rust
use std::path::{Path, PathBuf};

// Get filename without ANY extension
let prefix = Path::new("archive.tar.gz").file_prefix();
assert_eq!(prefix, Some(std::ffi::OsStr::new("archive")));

// Add extension (doesn't replace)
let mut path = PathBuf::from("data.tar");
path.add_extension("gz");
assert_eq!(path, PathBuf::from("data.tar.gz"));
```

### Strict Arithmetic (1.91+)

When overflow must ALWAYS panic (even in release builds):

```rust
// ✅ Use strict_* for critical calculations
fn calculate_fee(amount: u64, rate: u64) -> u64 {
    amount.strict_mul(rate).strict_div(100)  // Panics on overflow
}

// Available: strict_add, strict_sub, strict_mul, strict_div
// strict_rem, strict_neg, strict_pow, strict_shl, strict_shr
```

### RwLock Downgrade (1.92+)

```rust
use std::sync::{RwLock, RwLockWriteGuard};

let lock = RwLock::new(Data::default());

// Atomic downgrade from write to read
let mut write_guard = lock.write().unwrap();
write_guard.update();
let read_guard = RwLockWriteGuard::downgrade(write_guard);
// Now holds read lock — other readers can proceed
```

---

## Cargo Configuration for Modern Rust

```toml
# Cargo.toml
[package]
name = "modern-rust-project"
edition = "2024"
rust-version = "1.88"

[workspace]
resolver = "3"  # MSRV-aware dependency resolution

# Enable useful lints
[lints.rust]
unsafe_op_in_unsafe_fn = "warn"
missing_docs = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
```

```toml
# .cargo/config.toml

# Use LLD linker (default on Linux x86_64 since 1.90)
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Shared build directory for faster CI
[build]
build-dir = "/path/to/shared/target"
```

---

## Lints to Know About

### Deny-by-Default (Will Break Builds)

| Lint | Version | What to Fix |
|------|---------|-------------|
| `never_type_fallback_flowing_into_unsafe` | 1.92 | Add explicit type annotations |
| `dependency_on_unit_never_type_fallback` | 1.92 | Add explicit type annotations |

### Warn-by-Default (Address Promptly)

| Lint | Version | What to Fix |
|------|---------|-------------|
| `missing_abi` | 1.86 | Add explicit ABI: `extern "C" fn` |
| `dangerous_implicit_autorefs` | 1.88 | Make autoref explicit |
| `invalid_null_arguments` | 1.88 | Don't pass null to non-null params |
| `mismatched_lifetime_syntaxes` | 1.89 | Use consistent lifetime syntax |
| `dangling_pointers_from_locals` | 1.91 | Don't return pointers to locals |

---

## Migration Checklist: Pre-1.88 → 1.88+

When updating code from older Rust versions:

1. **Update Cargo.toml**
   ```toml
   edition = "2024"
   rust-version = "1.88"
   ```

2. **Run automatic fixes**
   ```bash
   cargo fix --edition
   ```

3. **Replace patterns manually:**
   - [ ] Nested `if let` → let chains
   - [ ] `Fn() -> impl Future` → `AsyncFn()`
   - [ ] `split_at_mut` tricks → `get_disjoint_mut`
   - [ ] Manual upcast methods → direct coercion
   - [ ] `retain` + collect → `extract_if`
   - [ ] External file locking crates → `File::lock()`
   - [ ] `ptr as usize as *const T` → `ptr.map_addr()`

4. **Address new lints**
   ```bash
   cargo clippy --all-targets
   ```

5. **Update unsafe blocks** for Edition 2024 requirements

---

## Breaking Changes Reference

| Version | Change | Action Required |
|---------|--------|-----------------|
| 1.85 | `std::env::set_var` unsafe | Add `unsafe` block |
| 1.86 | `missing_abi` warns | Add explicit `"C"` to `extern fn` |
| 1.88 | `#[bench]` requires feature | Use criterion or divan crate |
| 1.89 | WASM C ABI changed | Update wasm-bindgen ≥0.2.89 |
| 1.90 | LLD linker default | Opt out with `-C linker-features=-lld` if issues |
| 1.91 | `semicolon_in_expressions_from_macros` deny | Fix macro trailing semicolons |
| 1.92 | Never type lints deny | Add explicit type annotations |

---

## Summary: Default Choices for Modern Rust

When generating Rust code, AI agents should:

1. **Always use Edition 2024** unless compatibility required
2. **Always use let chains** for nested conditionals
3. **Always use async closures** with `AsyncFn` bounds
4. **Always use `get_disjoint_mut`** for multiple mutable refs
5. **Always use `extract_if`** when you need removed items
6. **Always use native `File::lock`** for file locking
7. **Always use strict provenance** APIs for pointer manipulation
8. **Always use `Duration::from_mins/hours`** for readability
9. **Always prefer `Box::new_zeroed`** for large zero-init allocations
10. **Always add explicit ABIs** to extern functions
