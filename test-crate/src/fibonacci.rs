use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================================
// OPTION 1: Global cache with lazy_static + Mutex (thread-safe)
// ============================================================================
// This is the most common way to have global mutable state in Rust.
// You'll need to add `lazy_static = "1.4"` to Cargo.toml

// Uncomment this to use:
// use lazy_static::lazy_static;
//
// lazy_static! {
//     static ref FIB_CACHE: Mutex<HashMap<u64, u64>> = Mutex::new(HashMap::new());
// }
//
// pub fn fibonacci(n: u64) -> u64 {
//     if n <= 1 {
//         return n;
//     }
//
//     // Try to get from cache
//     {
//         let cache = FIB_CACHE.lock().unwrap();
//         if let Some(&result) = cache.get(&n) {
//             return result;
//         }
//     } // Lock is dropped here
//
//     // Calculate and cache
//     let result = fibonacci(n - 1) + fibonacci(n - 2);
//
//     let mut cache = FIB_CACHE.lock().unwrap();
//     cache.insert(n, result);
//
//     result
// }

// ============================================================================
// OPTION 2: OnceLock (built-in, no external deps, Rust 1.70+)
// ============================================================================
// This is the modern approach without external dependencies

use std::sync::OnceLock;

static FIB_CACHE: OnceLock<Mutex<HashMap<u64, u64>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<u64, u64>> {
    FIB_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    // Try to get from cache
    {
        let cache = get_cache().lock().unwrap();
        if let Some(&result) = cache.get(&n) {
            return result;
        }
    } // Lock is dropped here

    // Calculate and cache
    let result = fibonacci(n - 1) + fibonacci(n - 2);

    let mut cache = get_cache().lock().unwrap();
    cache.insert(n, result);

    result
}

// ============================================================================
// OPTION 3: Pass HashMap as parameter (most idiomatic/Rusty way)
// ============================================================================
// This is the preferred approach in Rust - explicit is better than global state

pub fn fibonacci_with_cache(n: u64, cache: &mut HashMap<u64, u64>) -> u64 {
    if n <= 1 {
        return n;
    }

    if let Some(&result) = cache.get(&n) {
        return result;
    }

    let result = fibonacci_with_cache(n - 1, cache) + fibonacci_with_cache(n - 2, cache);
    cache.insert(n, result);

    result
}

// ============================================================================
// OPTION 4: Iterator-based (no memoization, but fast and elegant)
// ============================================================================
// For fibonacci specifically, iterative approach is often better

pub fn fibonacci_iterative(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }

    let mut a = 0;
    let mut b = 1;

    for _ in 2..=n {
        let temp = a + b;
        a = b;
        b = temp;
    }

    b
}

// Or using fold for a more functional style
pub fn fibonacci_functional(n: u64) -> u64 {
    (0..n).fold((0, 1), |(a, b), _| (b, a + b)).0
}

// ============================================================================
// Why global mutable state is tricky in Rust:
// ============================================================================
//
// 1. Static values must be initialized with const expressions
//    HashMap::new() is NOT const, so this doesn't work:
//    static FIB_CACHE: HashMap<i32, i32> = HashMap::new(); // ERROR!
//
// 2. Even with lazy initialization, you need interior mutability:
//    - Mutex<T> for thread-safe mutable access
//    - RefCell<T> for single-threaded (not safe for static)
//
// 3. The Rust philosophy: prefer explicit over implicit
//    Passing cache as parameter makes ownership and mutation clear
//
// 4. For performance-critical code, consider:
//    - Iterative approach (often faster)
//    - thread_local! for per-thread caching
//    - Pre-computed lookup tables
