# Rust Assembly & Compiler Inspection Guide

A comprehensive guide to understanding what Rust actually compiles to and how to inspect the generated code.

## Table of Contents

1. [Why Inspect Assembly?](#why-inspect-assembly)
2. [Quick Start Tools](#quick-start-tools)
3. [Assembly Basics](#assembly-basics)
4. [Cargo ASM - Local Assembly Inspection](#cargo-asm---local-assembly-inspection)
5. [Online Tools](#online-tools)
6. [Intermediate Representations](#intermediate-representations)
7. [Code Size Analysis](#code-size-analysis)
8. [Macro Expansion](#macro-expansion)
9. [Performance Profiling](#performance-profiling)
10. [Debugging Tools](#debugging-tools)
11. [Practical Examples](#practical-examples)
12. [Common Patterns in Assembly](#common-patterns-in-assembly)

---

## Why Inspect Assembly?

Understanding what Rust compiles to helps you:

- ✅ **Verify zero-cost abstractions** - See that iterators compile to raw loops
- ✅ **Understand ownership** - See that moves are just pointer copies
- ✅ **Debug performance** - Find hot paths and optimization opportunities
- ✅ **Learn optimization** - See how different code patterns affect output
- ✅ **Validate assumptions** - Confirm what the compiler is actually doing

**Key insight:** Rust's abstractions (ownership, borrowing, traits) compile away at runtime!

---

## Quick Start Tools

### Essential Tools to Install

```bash
# Assembly viewer (HIGHLY RECOMMENDED)
cargo install cargo-show-asm

# Code bloat analyzer
cargo install cargo-bloat

# Macro expander
cargo install cargo-expand

# LLVM IR analyzer
cargo install cargo-llvm-lines

# Flamegraph profiler (Linux/macOS)
cargo install cargo-flamegraph
```

### 30-Second Quick Start

```bash
# Build with optimizations
cd your-project
cargo build --release

# View assembly for a function
cargo asm --release your_crate::function_name --intel

# See what's taking up space
cargo bloat --release

# Expand macros
cargo expand
```

---

## Assembly Basics

### x86-64 Register Cheat Sheet

```assembly
; General Purpose Registers (64-bit)
rax, rbx, rcx, rdx    - General purpose (rax often for return values)
rsi, rdi              - Source/Destination (often used for args)
r8 - r15              - Additional general purpose

; Function Calling Convention (System V AMD64 - Linux/macOS)
; First 6 arguments in registers:
rdi    - 1st argument
rsi    - 2nd argument
rdx    - 3rd argument
rcx    - 4th argument
r8     - 5th argument
r9     - 6th argument
; Additional arguments on stack

rax    - Return value

; Special Registers
rsp    - Stack pointer
rbp    - Base pointer (frame pointer)
rip    - Instruction pointer
```

### Common Instructions

```assembly
; Intel Syntax (destination first)
mov rax, 5          ; Move 5 into rax
add rax, rbx        ; Add rbx to rax
sub rax, 10         ; Subtract 10 from rax
mul rbx             ; Multiply rax by rbx
imul rax, rbx       ; Signed multiply

lea rax, [rbp-8]    ; Load effective address (like &variable)
mov rax, [rbp-8]    ; Load value from memory

push rax            ; Push rax onto stack
pop rbx             ; Pop stack into rbx

cmp rax, rbx        ; Compare (sets flags)
je .label           ; Jump if equal
jne .label          ; Jump if not equal
jl .label           ; Jump if less
jg .label           ; Jump if greater

call function       ; Call function
ret                 ; Return from function

; AT&T Syntax (source first) - less common but seen in some tools
movq $5, %rax       ; Same as: mov rax, 5
addq %rbx, %rax     ; Same as: add rax, rbx
```

---

## Cargo ASM - Local Assembly Inspection

### Installation

```bash
cargo install cargo-show-asm
```

### Basic Usage

```bash
# List all available functions
cargo asm --release --list

# View specific function (Intel syntax - easier to read)
cargo asm --release --intel your_crate::module::function

# View with source code interleaved
cargo asm --release --intel --source your_crate::function

# Search for function by pattern
cargo asm --release --list | grep fibonacci

# Output to file
cargo asm --release --intel your_crate::function > output.asm
```

### Examples

```bash
# View the fibonacci function from our example
cargo asm --release --intel test_crate::fibonacci::fibonacci_iterative

# Compare multiple implementations
cargo asm --release --intel test_crate::fibonacci::fibonacci > fib_recursive.asm
cargo asm --release --intel test_crate::fibonacci::fibonacci_iterative > fib_iterative.asm
diff fib_recursive.asm fib_iterative.asm
```

### Understanding Output

```assembly
example::add:
        push    rbx                  ; Save rbx (callee-saved)
        mov     rbx, rdi             ; Move first arg to rbx
        add     rbx, rsi             ; Add second arg to rbx
        mov     rax, rbx             ; Move result to rax (return value)
        pop     rbx                  ; Restore rbx
        ret                          ; Return
```

---

## Online Tools

### 1. Rust Playground - Quick Experiments

**URL:** https://play.rust-lang.org/

**Features:**

- Write Rust code in browser
- Click **"..." → Tools → "ASM"** to see assembly
- No installation needed
- Share code via URL
- Multiple output formats (ASM, LLVM IR, MIR)

**Use cases:**

- Quick experiments
- Sharing code examples
- Testing compiler output
- Learning without local setup

### 2. Compiler Explorer (Godbolt) - Multi-Language Comparison

**URL:** https://rust.godbolt.org/

**Features:**

- **Interactive:** Click code to highlight corresponding assembly
- **Compare:** Side-by-side Rust, C, C++, and more
- **Optimization levels:** Toggle -O0, -O1, -O2, -O3
- **Multiple compilers:** Try different Rust versions
- **Color coding:** See exactly which code generates which assembly
- **Libraries:** Import crates to test

**How to use:**

1. Paste your Rust code on left
2. Assembly appears on right
3. Click any line to see mapping
4. Change optimization level in compiler options
5. Add filters (demangle, intel syntax, etc.)

**Pro tips:**

```rust
// Mark functions as pub to prevent them being optimized away
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Use #[inline(never)] to prevent inlining
#[inline(never)]
pub fn no_inline(x: i32) -> i32 {
    x * 2
}
```

### Example Session on Godbolt

**Input:**

```rust
pub fn simple_add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Output (with -O3):**

```assembly
simple_add:
        lea     eax, [rdi + rsi]    ; Single instruction!
        ret
```

---

## Intermediate Representations

### MIR (Mid-level Intermediate Representation)

**What is MIR?**

- Rust's internal representation after type checking
- Shows control flow clearly
- Includes borrow checker annotations
- Before LLVM optimization

**Generate MIR:**

```bash
# Method 1: Via rustc flags
cd test-crate
RUSTFLAGS="--emit mir" cargo build --release

# Output in: target/release/deps/*.mir
ls target/release/deps/*.mir
cat target/release/deps/test_crate-*.mir | less
```

**Method 2: More control**

```bash
cargo rustc --release -- --emit mir

# For specific function
cargo rustc --release -- --emit mir -Z dump-mir=all
```

**Example MIR output:**

```rust
fn fibonacci(_1: u64) -> u64 {
    let mut _0: u64;
    let mut _2: bool;

    bb0: {
        _2 = Le(_1, const 1_u64);
        switchInt(move _2) -> [false: bb1, otherwise: bb2];
    }

    bb1: {
        // Recursive case
    }

    bb2: {
        _0 = _1;
        return;
    }
}
```

### LLVM IR (LLVM Intermediate Representation)

**What is LLVM IR?**

- Platform-independent assembly-like code
- Input to LLVM optimizer
- More verbose than assembly
- Shows optimization opportunities

**Generate LLVM IR:**

```bash
# Generate .ll files
cargo rustc --release -- --emit llvm-ir

# Output in: target/release/deps/*.ll
cat target/release/deps/test_crate-*.ll | less

# Or view specific functions
cargo rustc --release -- --emit llvm-ir
grep -A 20 "define.*fibonacci" target/release/deps/*.ll
```

**Example LLVM IR:**

```llvm
define i64 @fibonacci(i64 %n) {
start:
  %0 = icmp ule i64 %n, 1
  br i1 %0, label %bb1, label %bb2

bb1:
  ret i64 %n

bb2:
  %1 = sub i64 %n, 1
  %2 = call i64 @fibonacci(i64 %1)
  %3 = sub i64 %n, 2
  %4 = call i64 @fibonacci(i64 %3)
  %5 = add i64 %2, %4
  ret i64 %5
}
```

---

## Code Size Analysis

### cargo-bloat - What's Taking Space?

**Installation:**

```bash
cargo install cargo-bloat
```

**Basic usage:**

```bash
cd test-crate

# Analyze entire binary
cargo bloat --release

# Show functions sorted by size
cargo bloat --release -n 20

# Filter by crate
cargo bloat --release --filter test_crate

# Show per-function breakdown
cargo bloat --release --fn
```

**Example output:**

```
 File  .text     Size Crate Name
 0.5%   5.1%   1.2KiB std   <std::io::Write::write_fmt>
 0.4%   4.2%   1.0KiB std   <String as Display>::fmt
 0.3%   3.1%     756B test  test_crate::main
 0.2%   2.5%     612B std   core::fmt::write
```

**Interpretation:**

- **File %** - Percentage of entire binary
- **.text %** - Percentage of code section
- **Size** - Actual size in bytes/KB
- **Crate** - Which crate/dependency
- **Name** - Function name (demangled)

### cargo-llvm-lines - Code Generation Analysis

**Installation:**

```bash
cargo install cargo-llvm-lines
```

**Usage:**

```bash
cargo llvm-lines --release
```

**What it shows:**

- How many lines of LLVM IR each function generates
- Which functions are monomorphized most (generics)
- Code bloat from excessive instantiation

**Example output:**

```
  Lines        Copies        Function name
  -----        ------        -------------
  15000 (10%)      42        <Vec<T> as Clone>::clone
  12000 (8%)       38        <HashMap<K,V> as Debug>::fmt
  8000  (5%)       25        core::fmt::write
```

**What to look for:**

- High copy counts indicate aggressive generic instantiation
- Large line counts show complex codegen
- Opportunities for code sharing with `dyn Trait`

---

## Macro Expansion

### cargo-expand - See What Macros Generate

**Installation:**

```bash
cargo install cargo-expand

# Requires nightly for full functionality
rustup install nightly
```

**Usage:**

```bash
# Expand entire crate
cargo expand

# Expand specific module
cargo expand fibonacci

# Expand specific function
cargo expand fibonacci::fibonacci

# Use nightly
cargo +nightly expand
```

**Example: What println! expands to**

**Your code:**

```rust
println!("Hello {}", name);
```

**Expands to:**

```rust
{
    ::std::io::_print(
        ::std::fmt::Arguments::new_v1(
            &["Hello ", "\n"],
            &[::std::fmt::ArgumentV1::new_display(&name)]
        )
    );
}
```

**Example: Derive macros**

**Your code:**

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
```

**Expands to:**

```rust
struct Point {
    x: i32,
    y: i32,
}

impl ::core::fmt::Debug for Point {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "Point",
            "x",
            &&self.x,
            "y",
            &&self.y,
        )
    }
}

impl ::core::clone::Clone for Point {
    fn clone(&self) -> Self {
        Point {
            x: ::core::clone::Clone::clone(&self.x),
            y: ::core::clone::Clone::clone(&self.y),
        }
    }
}

// ... PartialEq impl ...
```

---

## Performance Profiling

### cargo-flamegraph - Visual Performance Analysis

**Installation (Linux/macOS):**

```bash
cargo install flamegraph
```

**Usage:**

```bash
cd test-crate

# Generate flamegraph
cargo flamegraph --release

# Opens flamegraph.svg in browser
# Click on boxes to zoom into function calls
```

**What is a flamegraph?**

- Visual representation of where CPU time is spent
- Width = time spent
- Height = call stack depth
- Interactive - click to zoom

**Reading flamegraphs:**

- Wide boxes = hot functions (optimization targets)
- Tall stacks = deep call chains
- Color = nothing (just for contrast)

### perf - Linux Performance Analysis

**Installation:**

```bash
# Usually pre-installed on Linux
sudo apt-get install linux-perf  # Ubuntu/Debian
```

**Usage:**

```bash
# Build with debug symbols
cargo build --release

# Record performance
perf record --call-graph=dwarf ./target/release/test-crate

# View report
perf report

# Annotate source with performance data
perf annotate
```

**Example output:**

```
  45.23%  test-crate  [.] fibonacci_iterative
  32.11%  test-crate  [.] main
  12.05%  test-crate  [.] __libc_start_main
```

### Instruments (macOS)

**Built-in profiler for macOS:**

1. Build release binary
2. Open Instruments.app
3. Choose "Time Profiler" or "Allocations"
4. Select your binary
5. Run and analyze

---

## Debugging Tools

### LLDB/GDB - Step-by-Step Execution

**Build with debug symbols:**

```bash
cargo build --debug  # or just: cargo build
```

**Using LLDB (macOS/Linux):**

```bash
lldb target/debug/test-crate

# Inside lldb:
(lldb) b main                    # Set breakpoint at main
(lldb) b fibonacci.rs:52         # Breakpoint at line 52
(lldb) r                         # Run
(lldb) n                         # Next line
(lldb) s                         # Step into function
(lldb) c                         # Continue
(lldb) p variable_name           # Print variable
(lldb) fr v                      # Frame variables (all locals)
(lldb) disassemble               # Show assembly for current function
(lldb) register read             # Show register values
(lldb) bt                        # Backtrace (call stack)
```

**Using GDB (Linux):**

```bash
gdb target/debug/test-crate

# Commands similar to lldb
(gdb) break main
(gdb) run
(gdb) next
(gdb) step
(gdb) print variable
(gdb) disassemble
(gdb) info registers
(gdb) backtrace
```

### rust-lldb/rust-gdb - Rust-aware Debuggers

**Prettier printing of Rust types:**

```bash
# Use rust-lldb instead of lldb
rust-lldb target/debug/test-crate

# Or rust-gdb instead of gdb
rust-gdb target/debug/test-crate
```

**Features:**

- Pretty-prints Rust types (Vec, String, etc.)
- Understands Option, Result
- Shows actual values instead of internal representation

### VS Code Debugging

**Install extension:** CodeLLDB

**Launch configuration (.vscode/launch.json):**

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug test-crate",
      "cargo": {
        "args": ["build", "--bin=test-crate"]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

---

## Practical Examples

### Example 1: Verify Move is Cheap

**Code:**

```rust
pub fn take_string(s: String) -> String {
    s  // Just return the input
}
```

**View assembly:**

```bash
cargo asm --release --intel your_crate::take_string
```

**Result:**

```assembly
take_string:
        mov     rax, rdi    ; Just copy pointer to return register
        mov     rdx, rsi    ; Copy length
        mov     rcx, r8     ; Copy capacity
        ret                 ; Done - no memcpy!
```

**Conclusion:** Moving a String is 3 register moves - incredibly cheap!

### Example 2: Iterator Optimization

**Code:**

```rust
pub fn sum_squares(n: u32) -> u32 {
    (0..n).map(|x| x * x).sum()
}

pub fn sum_squares_manual(n: u32) -> u32 {
    let mut sum = 0;
    for i in 0..n {
        sum += i * i;
    }
    sum
}
```

**Compare on Godbolt:**

- Both compile to identical assembly with -O3!
- Iterator version is zero-cost abstraction

### Example 3: Option is Free

**Code:**

```rust
pub fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}
```

**Assembly shows:**

- No heap allocation
- Just clever register use
- Match statements compile to simple jumps

### Example 4: Comparing Fibonacci Implementations

```bash
cd test-crate

# View each implementation
cargo asm --release --intel test_crate::fibonacci::fibonacci > fib_memo.asm
cargo asm --release --intel test_crate::fibonacci::fibonacci_iterative > fib_iter.asm
cargo asm --release --intel test_crate::fibonacci::fibonacci_functional > fib_func.asm

# Compare file sizes
wc -l *.asm

# Iterative and functional should be similar and small
# Memoized will be larger (includes lock/unlock code)
```

---

## Common Patterns in Assembly

### Pattern 1: Function Prologue/Epilogue

**Prologue (setup):**

```assembly
push    rbp              ; Save old base pointer
mov     rbp, rsp         ; Set new base pointer
sub     rsp, 32          ; Allocate 32 bytes on stack
```

**Epilogue (cleanup):**

```assembly
add     rsp, 32          ; Deallocate stack space
pop     rbp              ; Restore base pointer
ret                      ; Return
```

### Pattern 2: Tail Call Optimization

**Before:**

```rust
fn factorial(n: u32) -> u32 {
    if n <= 1 { 1 } else { n * factorial(n - 1) }
}
```

**After optimization (tail recursion):**

```assembly
factorial:
.loop:
        cmp     edi, 1
        jbe     .done
        ; Transform recursion to loop!
        dec     edi
        jmp     .loop
.done:
        mov     eax, 1
        ret
```

### Pattern 3: Inlining

**Small functions are often inlined:**

```rust
#[inline]  // Hint to inline
fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn caller() -> i32 {
    add(5, 3)
}
```

**Assembly shows no call - just:**

```assembly
caller:
        mov     eax, 8      ; Computed at compile time!
        ret
```

### Pattern 4: Loop Unrolling

**Code:**

```rust
pub fn sum_array(arr: &[i32; 4]) -> i32 {
    arr.iter().sum()
}
```

**Optimized assembly may unroll:**

```assembly
sum_array:
        mov     eax, [rdi]       ; arr[0]
        add     eax, [rdi + 4]   ; arr[1]
        add     eax, [rdi + 8]   ; arr[2]
        add     eax, [rdi + 12]  ; arr[3]
        ret
```

---

## Quick Reference Commands

```bash
# ============================================================================
# ASSEMBLY OUTPUT
# ============================================================================

# Local (requires cargo-show-asm)
cargo asm --release --intel FUNCTION

# Online
# Rust Playground: https://play.rust-lang.org/ → Tools → ASM
# Godbolt: https://rust.godbolt.org/

# ============================================================================
# CODE SIZE ANALYSIS
# ============================================================================

# Binary size breakdown
cargo bloat --release

# LLVM IR line counts
cargo llvm-lines --release

# ============================================================================
# INTERMEDIATE REPRESENTATIONS
# ============================================================================

# MIR (Rust's IR)
RUSTFLAGS="--emit mir" cargo build --release
cat target/release/deps/*.mir

# LLVM IR
cargo rustc --release -- --emit llvm-ir
cat target/release/deps/*.ll

# ============================================================================
# MACRO EXPANSION
# ============================================================================

cargo expand
cargo +nightly expand MODULE

# ============================================================================
# PROFILING
# ============================================================================

# Flamegraph (visual)
cargo flamegraph --release

# Linux perf
perf record --call-graph=dwarf ./target/release/BINARY
perf report

# ============================================================================
# DEBUGGING
# ============================================================================

# LLDB
lldb target/debug/BINARY

# GDB
gdb target/debug/BINARY

# Rust-aware versions
rust-lldb target/debug/BINARY
rust-gdb target/debug/BINARY

# ============================================================================
# STANDARD UNIX TOOLS
# ============================================================================

# Disassemble binary
objdump -d target/release/BINARY

# List symbols
nm target/release/BINARY

# Demangled symbols
nm target/release/BINARY | rustfilt

# Binary info
file target/release/BINARY
size target/release/BINARY
```

---

## Learning Path

### Week 1: Get Familiar

1. Install `cargo-show-asm`
2. Use Rust Playground to see simple function assembly
3. Compare optimized vs unoptimized builds

### Week 2: Understand Patterns

1. Write simple functions (add, multiply, conditionals)
2. View assembly for each
3. Match assembly to source code

### Week 3: Verify Abstractions

1. Test iterator vs manual loops
2. Compare Option vs manual null checks
3. Verify moves are cheap

### Week 4: Optimize

1. Profile your code
2. Identify hot paths
3. View assembly of slow functions
4. Optimize and compare

---

## Pro Tips

1. **Always use `--release` for realistic output** - debug builds have no optimization
2. **Use `--intel` syntax** - easier to read than AT&T
3. **Start with small functions** - don't try to understand entire programs
4. **Compare with C** - use Godbolt to see Rust vs C for same algorithm
5. **Use `#[inline(never)]`** - prevents inlining when testing
6. **Mark functions `pub`** - prevents dead code elimination
7. **Use Godbolt for learning** - interactive highlighting is invaluable
8. **Don't micro-optimize** - profile first, optimize second

---

## Common Questions

**Q: Why is my function missing from assembly output?**
A: It was probably optimized away or inlined. Mark it `pub` and `#[inline(never)]`.

**Q: Why does release assembly look so different from my code?**
A: LLVM performs aggressive optimization. Use `-C opt-level=0` for less optimization.

**Q: How do I see what a specific line compiles to?**
A: Use Godbolt and click on the line - it highlights corresponding assembly.

**Q: Is assembly the same on all platforms?**
A: No! x86-64 (Intel/AMD), ARM, and others have different instruction sets.

**Q: Should I write assembly to optimize Rust?**
A: Almost never! Rust + LLVM are smarter than manual assembly in 99% of cases.

---

## Additional Resources

- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **Rust Compiler Development Guide**: https://rustc-dev-guide.rust-lang.org/
- **LLVM Documentation**: https://llvm.org/docs/
- **x86-64 Reference**: https://www.felixcloutier.com/x86/
- **Agner Fog's Optimization Manuals**: https://www.agner.org/optimize/

---

## Summary

**The toolkit:**

- 🔍 **cargo-show-asm** - View assembly locally
- 🌐 **Godbolt** - Interactive online exploration
- 📊 **cargo-bloat** - Find code bloat
- 🔬 **cargo-expand** - Understand macros
- 🔥 **flamegraph** - Profile performance
- 🐛 **lldb/gdb** - Debug step-by-step

**Key insights from assembly:**

- Ownership = zero runtime cost
- Moves = pointer copies
- Iterators = optimized to raw loops
- Option = no allocation
- Trait objects = vtable dispatch
- Generics = monomorphization (code duplication)

**Remember:** Understanding assembly helps you build better mental models of Rust's performance characteristics, but you should profile before optimizing!

---

**Happy disassembling! 🦀⚙️**
