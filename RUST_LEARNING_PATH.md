# Rust Learning Path for Python, TypeScript & Java Developers

Welcome! This guide is designed to get you productive in Rust quickly by building on your experience with Python, TypeScript, and Java.

## Table of Contents
1. [Quick Overview](#quick-overview)
2. [The Big Difference: Ownership](#the-big-difference-ownership)
3. [Syntax Crash Course](#syntax-crash-course)
4. [Type System](#type-system)
5. [Error Handling](#error-handling)
6. [Collections & Iterators](#collections--iterators)
7. [Structs, Enums & Pattern Matching](#structs-enums--pattern-matching)
8. [Traits (Like Interfaces++)](#traits-like-interfaces)
9. [Modules & Crates](#modules--crates)
10. [Concurrency](#concurrency)
11. [Common Patterns](#common-patterns)
12. [Next Steps](#next-steps)

---

## Quick Overview

**Rust in one sentence:** A systems programming language with zero-cost abstractions, memory safety without garbage collection, and fearless concurrency.

**What you already know:**
- **From Java:** Strong static typing, explicit types, compiled language, interfaces (traits)
- **From TypeScript:** Type inference, generics, structural typing concepts, modern tooling
- **From Python:** Powerful iterators, great ergonomics, strong package ecosystem

**What's different:**
- No garbage collector (ownership system manages memory)
- Immutable by default (like TypeScript `const`, but enforced)
- Explicit error handling (no exceptions)
- No null (uses `Option<T>`)
- Compiler is your strict teacher (if it compiles, it usually works)

---

## The Big Difference: Ownership

This is THE fundamental concept in Rust. Everything else makes sense once you understand this.

### The Three Rules

1. **Each value has exactly one owner**
2. **When the owner goes out of scope, the value is dropped (freed)**
3. **Values can be borrowed (referenced) but not owned by multiple places**

### Coming from Python/JavaScript

```python
# Python - reference counting
x = [1, 2, 3]
y = x  # Both point to same list
y.append(4)  # x also has 4 now
```

```rust
// Rust - ownership transfers (move)
let x = vec![1, 2, 3];
let y = x;  // x is MOVED to y, x is now invalid
// println!("{:?}", x);  // ERROR! x no longer exists
println!("{:?}", y);  // OK
```

### Borrowing (References)

Like pointers in Java, but compile-time checked:

```rust
let x = vec![1, 2, 3];
let y = &x;  // Borrow (immutable reference)
// Both x and y are valid here
println!("{:?}", x);
println!("{:?}", y);
```

**Rules:**
- Many immutable references (`&T`) **OR** one mutable reference (`&mut T`)
- Never both at the same time
- References must always be valid (no dangling pointers)

```rust
let mut x = vec![1, 2, 3];
let y = &mut x;  // Mutable borrow
y.push(4);
// println!("{:?}", x);  // ERROR! Can't use x while mutably borrowed
println!("{:?}", y);  // OK
```

### When to Use What

| Pattern | Use Case | Example |
|---------|----------|---------|
| Move (ownership transfer) | Consuming operations | `thread::spawn(move \|\| { ... })` |
| Immutable borrow (`&T`) | Read-only access | `fn print(data: &Vec<i32>)` |
| Mutable borrow (`&mut T`) | Modify without owning | `fn add_one(x: &mut i32)` |
| Clone | Need independent copy | `let y = x.clone();` |

**Python/Java devs:** Think of this as making memory management bugs impossible at compile time.

---

## Syntax Crash Course

### Variables & Mutability

```rust
// Immutable by default (like TypeScript const)
let x = 5;
// x = 6;  // ERROR!

// Must explicitly opt-in to mutability
let mut y = 5;
y = 6;  // OK

// Type inference (like TypeScript/Python)
let name = "Alice";  // inferred as &str

// Explicit types (like TypeScript/Java)
let age: i32 = 30;

// Shadowing (redeclare same name)
let x = 5;
let x = x + 1;  // New variable, shadows previous
let x = "now a string";  // Can even change type
```

### Functions

```rust
// Python/TS devs: Note the return type syntax
fn add(a: i32, b: i32) -> i32 {
    a + b  // No semicolon = return expression
}

// Explicit return
fn subtract(a: i32, b: i32) -> i32 {
    return a - b;  // With semicolon
}

// No return value (like void in Java/TS)
fn print_sum(a: i32, b: i32) {
    println!("{}", a + b);
}

// Unit type () is like void
fn explicit_unit() -> () {
    println!("Returns unit");
}
```

### Control Flow

```rust
// If expressions (like ternary, but better)
let number = if condition { 5 } else { 6 };

// Pattern matching (like switch on steroids)
match value {
    1 => println!("one"),
    2 | 3 => println!("two or three"),
    4..=10 => println!("four through ten"),
    _ => println!("something else"),  // default case
}

// Loops
loop {  // infinite loop
    break;
}

while condition {
    // ...
}

for item in collection {  // Like Python's for-in
    // ...
}

for i in 0..10 {  // Range (exclusive end)
    // ...
}

for i in 0..=10 {  // Inclusive range
    // ...
}
```

---

## Type System

### Primitives

```rust
// Integers (signed and unsigned)
let a: i8 = -128;      // 8-bit signed
let b: u8 = 255;       // 8-bit unsigned
let c: i32 = -1000;    // 32-bit (default int type)
let d: usize = 100;    // Architecture-dependent (like size_t in C)

// Floats
let e: f32 = 3.14;
let f: f64 = 3.14159;  // Default float type

// Boolean
let t: bool = true;

// Char (4 bytes, Unicode scalar)
let c: char = '😀';

// Tuples (like Python)
let tuple: (i32, f64, &str) = (500, 6.4, "hello");
let (x, y, z) = tuple;  // Destructuring
let first = tuple.0;     // Index access
```

### Strings

**Two types** (this trips everyone up):

```rust
// &str - string slice (borrowed, immutable)
// Like Python str or Java String
let s1: &str = "hello";  // String literal

// String - owned, mutable, heap-allocated
// Like Python's bytearray or Java StringBuilder
let mut s2: String = String::from("hello");
s2.push_str(", world");

// Convert between them
let s3: &str = &s2;           // String -> &str
let s4: String = s3.to_string();  // &str -> String
```

**When to use:**
- `&str` for function parameters (more flexible)
- `String` when you need ownership or mutation

### Arrays & Vectors

```rust
// Array - fixed size, stack allocated (like Java array)
let arr: [i32; 3] = [1, 2, 3];
let first = arr[0];

// Vector - dynamic size, heap allocated (like Python list or Java ArrayList)
let mut vec: Vec<i32> = vec![1, 2, 3];  // vec! macro
vec.push(4);
let first = vec[0];
// vec.get(0)  // Returns Option<&T> (safer)
```

---

## Error Handling

**No exceptions!** Rust uses types for errors.

### Option<T> - Like TypeScript's T | undefined

```rust
// Instead of null/undefined/None
fn find_user(id: i32) -> Option<User> {
    if id == 1 {
        Some(User { name: "Alice" })
    } else {
        None
    }
}

// Using Option
match find_user(1) {
    Some(user) => println!("Found {}", user.name),
    None => println!("Not found"),
}

// Or use if let (syntactic sugar)
if let Some(user) = find_user(1) {
    println!("Found {}", user.name);
}

// Unwrap (panics if None - use sparingly)
let user = find_user(1).unwrap();

// Unwrap with default
let user = find_user(999).unwrap_or(default_user);

// Question mark operator (early return)
fn get_user_name(id: i32) -> Option<String> {
    let user = find_user(id)?;  // Returns None if None
    Some(user.name)
}
```

### Result<T, E> - Like Either monad or checked exceptions

```rust
use std::fs::File;
use std::io::{self, Read};

// Function that can fail
fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;  // ? propagates error
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)  // Wrap success in Ok
}

// Using Result
match read_file("data.txt") {
    Ok(contents) => println!("{}", contents),
    Err(e) => eprintln!("Error: {}", e),
}

// Or with ?
fn wrapper() -> Result<(), io::Error> {
    let contents = read_file("data.txt")?;
    println!("{}", contents);
    Ok(())
}
```

**Python devs:** Like using return values instead of try/except.
**Java devs:** Like checked exceptions, but as return types.

---

## Collections & Iterators

### Common Collections

```rust
use std::collections::{HashMap, HashSet};

// HashMap (like Python dict or TS Map)
let mut map = HashMap::new();
map.insert("key", "value");
let val = map.get("key");  // Returns Option<&V>

// HashSet (like Python set or TS Set)
let mut set = HashSet::new();
set.insert(1);
set.contains(&1);  // true
```

### Iterators (Like Python iterators++)

```rust
let vec = vec![1, 2, 3, 4, 5];

// Map (like Python map or TS .map())
let doubled: Vec<i32> = vec.iter().map(|x| x * 2).collect();

// Filter (like Python filter or TS .filter())
let evens: Vec<&i32> = vec.iter().filter(|x| *x % 2 == 0).collect();

// Fold (like Python reduce or TS .reduce())
let sum: i32 = vec.iter().fold(0, |acc, x| acc + x);

// Chain operations (lazy evaluation)
let result: Vec<i32> = vec.iter()
    .filter(|x| *x % 2 == 0)
    .map(|x| x * 2)
    .collect();

// For comprehension style
let squares: Vec<i32> = (0..10).map(|x| x * x).collect();
```

**Iterators are zero-cost abstractions** - as fast as hand-written loops!

---

## Structs, Enums & Pattern Matching

### Structs (Like classes without methods... for now)

```rust
// Regular struct (like Java class or TS interface)
struct User {
    name: String,
    email: String,
    age: u32,
}

// Creating instances
let user = User {
    name: String::from("Alice"),
    email: String::from("alice@example.com"),
    age: 30,
};

// Accessing fields
println!("{}", user.name);

// Struct update syntax (like object spread)
let user2 = User {
    email: String::from("alice2@example.com"),
    ..user  // Copy other fields
};

// Tuple struct
struct Point(i32, i32);
let p = Point(10, 20);

// Unit struct (no fields)
struct Marker;
```

### Implementations (Methods)

```rust
impl User {
    // Associated function (like static method)
    fn new(name: String, email: String, age: u32) -> User {
        User { name, email, age }
    }
    
    // Method (takes &self)
    fn greet(&self) {
        println!("Hello, I'm {}", self.name);
    }
    
    // Mutable method
    fn have_birthday(&mut self) {
        self.age += 1;
    }
    
    // Consuming method (takes ownership)
    fn into_name(self) -> String {
        self.name
    }
}

// Usage
let mut user = User::new(
    String::from("Alice"),
    String::from("alice@example.com"),
    30
);
user.greet();
user.have_birthday();
```

### Enums (Like TypeScript unions on steroids)

```rust
// Simple enum (like Java enum)
enum Status {
    Active,
    Inactive,
    Pending,
}

// Enum with data (like TS discriminated unions)
enum Message {
    Quit,
    Move { x: i32, y: i32 },  // Struct-like
    Write(String),             // Tuple-like
    ChangeColor(i32, i32, i32),
}

// Pattern matching
let msg = Message::Write(String::from("hello"));
match msg {
    Message::Quit => println!("Quit"),
    Message::Move { x, y } => println!("Move to {}, {}", x, y),
    Message::Write(text) => println!("Text: {}", text),
    Message::ChangeColor(r, g, b) => println!("RGB: {}, {}, {}", r, g, b),
}

// Methods on enums
impl Message {
    fn call(&self) {
        // ...
    }
}
```

**Option and Result are just enums!**

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

---

## Traits (Like Interfaces++)

Traits define shared behavior. Think: Java interfaces + TypeScript interfaces + mixins.

### Defining Traits

```rust
trait Summary {
    fn summarize(&self) -> String;
    
    // Default implementation
    fn announce(&self) -> String {
        format!("Summary: {}", self.summarize())
    }
}

struct Article {
    title: String,
    content: String,
}

// Implementing trait
impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.title, &self.content[..50])
    }
}

let article = Article {
    title: String::from("Rust is great"),
    content: String::from("Learn Rust today..."),
};

println!("{}", article.summarize());
```

### Trait Bounds (Generic Constraints)

```rust
// Function that accepts anything implementing Summary
fn notify(item: &impl Summary) {
    println!("{}", item.summarize());
}

// More explicit syntax (like Java <T extends Summary>)
fn notify_v2<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

// Multiple trait bounds
fn notify_v3<T: Summary + Display>(item: &T) {
    // ...
}

// Where clause (cleaner for complex bounds)
fn notify_v4<T>(item: &T)
where
    T: Summary + Display,
{
    // ...
}
```

### Common Traits

```rust
// Debug - for {:?} formatting
#[derive(Debug)]
struct Point { x: i32, y: i32 }

// Clone - explicit cloning
#[derive(Clone)]
struct Data { /* ... */ }

// Copy - implicit copying (for simple types)
#[derive(Copy, Clone)]
struct SmallData { x: i32 }

// PartialEq, Eq - equality comparison
#[derive(PartialEq, Eq)]
struct User { /* ... */ }

// Derive multiple traits
#[derive(Debug, Clone, PartialEq)]
struct Person {
    name: String,
    age: u32,
}
```

**Python devs:** Like `__str__`, `__eq__`, etc., but explicit.
**Java devs:** Like interfaces + Object methods (equals, toString).

---

## Modules & Crates

### Crates (Packages)

```bash
# Create new binary project
cargo new my_project
cd my_project

# Create new library
cargo new --lib my_lib

# Add dependency
cargo add serde  # Or edit Cargo.toml
```

**Cargo.toml** (like package.json or requirements.txt):

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

### Modules

```rust
// In src/lib.rs or src/main.rs
mod network {
    pub fn connect() {
        // ...
    }
    
    fn internal() {  // Private by default
        // ...
    }
}

// Use the module
fn main() {
    network::connect();
}

// Import into scope
use network::connect;

fn main() {
    connect();
}
```

**File structure:**

```
src/
├── main.rs
├── lib.rs
└── network/
    ├── mod.rs      // Module root
    ├── client.rs
    └── server.rs
```

```rust
// In src/network/mod.rs
pub mod client;
pub mod server;

// In src/main.rs
mod network;
use network::client;
```

**Like:**
- Python: `import` statements and `__init__.py`
- TypeScript: `import/export` and file modules
- Java: packages and `import`

---

## Concurrency

Rust's killer feature: **fearless concurrency** - compiler prevents data races!

### Threads

```rust
use std::thread;
use std::time::Duration;

fn main() {
    // Spawn thread
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("Thread: {}", i);
            thread::sleep(Duration::from_millis(1));
        }
    });
    
    // Main thread work
    for i in 1..5 {
        println!("Main: {}", i);
        thread::sleep(Duration::from_millis(1));
    }
    
    // Wait for thread
    handle.join().unwrap();
}

// Moving data into thread
let v = vec![1, 2, 3];
let handle = thread::spawn(move || {
    println!("{:?}", v);  // v is moved into closure
});
```

### Message Passing (Channels)

```rust
use std::sync::mpsc;  // Multiple producer, single consumer
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        tx.send(String::from("hello")).unwrap();
    });
    
    let received = rx.recv().unwrap();
    println!("Got: {}", received);
}
```

### Shared State (Arc + Mutex)

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    // Arc = Atomic Reference Counted (thread-safe Rc)
    // Mutex = Mutual exclusion lock
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Result: {}", *counter.lock().unwrap());
}
```

### Async/Await (Like JavaScript/Python)

```rust
// Using tokio runtime
use tokio;

#[tokio::main]
async fn main() {
    let result = fetch_data().await;
    println!("{}", result);
}

async fn fetch_data() -> String {
    // Async work
    String::from("data")
}

// Concurrent tasks
use tokio::join;

async fn do_stuff() {
    let (result1, result2) = join!(
        async_task_1(),
        async_task_2(),
    );
}
```

---

## Common Patterns

### Builder Pattern

```rust
struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

impl User {
    fn builder() -> UserBuilder {
        UserBuilder::default()
    }
}

#[derive(Default)]
struct UserBuilder {
    name: Option<String>,
    email: Option<String>,
    age: Option<u32>,
}

impl UserBuilder {
    fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
    
    fn build(self) -> Result<User, String> {
        Ok(User {
            name: self.name.ok_or("Name required")?,
            email: self.email.ok_or("Email required")?,
            age: self.age,
        })
    }
}

// Usage
let user = User::builder()
    .name(String::from("Alice"))
    .email(String::from("alice@example.com"))
    .age(30)
    .build()?;
```

### Newtype Pattern (Type Safety)

```rust
// Wrap primitive types for type safety
struct UserId(i32);
struct ProductId(i32);

fn get_user(id: UserId) { /* ... */ }
fn get_product(id: ProductId) { /* ... */ }

let user_id = UserId(1);
let product_id = ProductId(1);

get_user(user_id);  // OK
// get_user(product_id);  // Compile error!
```

### RAII Pattern (Resource Acquisition Is Initialization)

```rust
// File is automatically closed when dropped
{
    let file = File::open("data.txt")?;
    // Use file...
}  // file is automatically closed here

// Same with locks
{
    let data = mutex.lock().unwrap();
    // Use data...
}  // lock is automatically released
```

### Iterator Adapters (Functional Style)

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Chain multiple operations
let result: i32 = numbers
    .iter()
    .filter(|&&x| x % 2 == 0)
    .map(|&x| x * x)
    .sum();

// Custom iterator
struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count < 6 {
            Some(self.count)
        } else {
            None
        }
    }
}
```

### Error Propagation Pattern

```rust
use std::error::Error;

// Box<dyn Error> accepts any error type
fn process() -> Result<(), Box<dyn Error>> {
    let data = read_file("data.txt")?;
    let parsed = parse_data(&data)?;
    save_result(parsed)?;
    Ok(())
}
```

---

## Next Steps

### 1. Practice Projects

Start with these to build muscle memory:

1. **CLI Tool** - Parse args, read files, format output
   - Try: `cargo add clap` for argument parsing
   
2. **Web API** - Build a REST API
   - Try: `axum` or `actix-web` frameworks
   
3. **Data Processing** - Read CSV, transform, write output
   - Try: `csv` and `serde` crates

4. **Concurrent Tool** - Web scraper or parallel file processor
   - Try: `tokio` for async or `rayon` for parallel iterators

### 2. Essential Crates to Know

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }  # Serialization
tokio = { version = "1", features = ["full"] }       # Async runtime
clap = { version = "4", features = ["derive"] }      # CLI parsing
anyhow = "1"                                          # Error handling
thiserror = "1"                                       # Custom errors
reqwest = "0.11"                                      # HTTP client
sqlx = "0.7"                                          # SQL toolkit
```

### 3. Key Resources

- **The Rust Book**: https://doc.rust-lang.org/book/ (comprehensive)
- **Rust by Example**: https://doc.rust-lang.org/rust-by-example/ (hands-on)
- **Rustlings**: Small exercises to learn Rust syntax
- **Exercism Rust Track**: Practice problems with mentoring

### 4. Debugging Tips

```rust
// Print debugging
println!("{:?}", variable);     // Debug print
println!("{:#?}", variable);    // Pretty debug print
dbg!(variable);                 // Debug macro (shows location)

// Common compiler errors
// - "value borrowed here after move" → You moved a value, use & instead
// - "cannot borrow as mutable" → Multiple borrows, restructure code
// - "lifetime may not live long enough" → References outlive data
```

### 5. Mental Model Shifts

**From Python:**
- Think about ownership (not reference counting)
- No implicit conversions
- Explicit error handling (no try/except)

**From TypeScript:**
- Stricter type system
- No implicit nulls
- Trait bounds instead of structural typing

**From Java:**
- No null (use Option)
- No exceptions (use Result)
- Stack vs heap matters
- Traits are more powerful than interfaces

### 6. The Rust Learning Curve

```
Productivity
    ^
    |        ..... <- You'll get here faster than you think
    |      ..
    |    ..
    |  .. 
    |..____________> Time
     ^
     First few weeks (steep but rewarding)
```

**Don't get discouraged by the compiler!** It's strict, but once you understand what it wants, you'll write more correct code on the first try.

---

## Quick Reference Cheat Sheet

```rust
// Variables
let x = 5;              // Immutable
let mut y = 5;          // Mutable
const MAX: i32 = 100;   // Constant

// Functions
fn add(a: i32, b: i32) -> i32 { a + b }

// Ownership
let s1 = String::from("hello");
let s2 = s1;            // Move
let s3 = s2.clone();    // Clone
let s4 = &s3;           // Borrow

// Control flow
if x > 5 { } else { }
match x { 1 => {}, _ => {} }
for i in 0..10 { }

// Types
i32, u32, f64, bool, char
String, &str
Vec<T>, [T; N]
Option<T>, Result<T, E>

// Traits
impl TraitName for TypeName { }
fn foo<T: Trait>(x: T) { }

// Macros (end with !)
println!(), vec![], format!()

// Common methods
.unwrap(), .expect()
.map(), .filter(), .collect()
.iter(), .into_iter(), .iter_mut()
```

---

## You're Ready!

Start with a small project. Fight the compiler. Learn from it. Before you know it, you'll be writing safe, fast, concurrent code without thinking twice.

**Welcome to Rust!** 🦀
