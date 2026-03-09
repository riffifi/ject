# Ject

> **A modern, expressive scripting language built in Rust**

Ject combines the simplicity of Python with elegant syntax inspired by Ruby and Crystal. Built for readability, performance, and developer happiness.

```ject
# Hello, World!
print "Hello, World!"

# Functions
fn greet(name)
    print "Hello, " + name + "!"
end

greet("World")

# Arrays with beautiful slicing
let numbers = [1, 2, 3, 4, 5]
print numbers[from:1 to:4]    # [2, 3, 4]
print numbers[::2]            # [1, 3, 5]

# Ranges
for i in 1..6 do
    print "Count: " + i
end
```

---

## Quick Start

### Installation

```bash
git clone <repository-url>
cd ject
cargo build --release
./target/release/ject
```

### Your First Program

Create `hello.ject`:

```ject
print "Hello, World!"

let name = "Ject"
print "Welcome to " + name
```

Run it:

```bash
./target/release/ject hello.ject
```

---

## Why Ject?

###  Clean Syntax

```ject
# Native elseif (not awkward 'elif')
if score >= 90
    print "A"
elseif score >= 80
    print "B"
else
    print "C"
end

# Explicit blocks - no indentation fragility
if condition
    if nested
        do_something()
    end
end
```

### 🚀 Rich Standard Library

```ject
# No 'import math' needed
print sqrt(16)      # 4
print sin(PI / 2)   # 1
print abs(-42)      # 42
```

###  Three Slicing Syntaxes (Better Than Python!)

```ject
let arr = [0, 1, 2, 3, 4, 5]

# Named parameters (most readable)
arr[from:1 to:4]        # [1, 2, 3]
arr[from:0 to:5 step:2] # [0, 2, 4]

# Range syntax (concise)
arr[1..4]               # [1, 2, 3]
arr[0..5:2]             # [0, 2, 4]

# Python-style (familiar)
arr[1:4]                # [1, 2, 3]
arr[::2]                # [0, 2, 4]
```

### 🧩 Beautiful Lambdas

```ject
let square = fn(x) -> x * x
print square(5)  # 25

let numbers = [1, 2, 3, 4, 5]
let doubled = map(numbers, fn(n) -> n * 2)
```

---

## Features

- - Dynamic typing with clean syntax
- - First-class functions and closures
- - Lambda expressions
- - **Three slicing syntaxes** for arrays and strings
- - **Unique arrays** `{| |}` with auto-deduplication
- - Range expressions (`1..6`, `1..10:2`)
- - Module system with selective imports
- - Struct definitions
- - Try/catch error handling
- - String interpolation
- - Negative indexing
- - Built-in standard library
- - REPL with history

---

## Documentation

 **Full documentation:** See [DOCUMENTATION.md](DOCUMENTATION.md)

The documentation includes:
- Complete language reference
- Standard library guide
- Best practices
- Comparison with other languages
- Quick reference card

---

## Language Overview

### Variables

```ject
let name = "Alice"  # Declare
name = "Bob"        # Reassign
```

### Functions

```ject
fn add(a, b)
    return a + b
end

# Default parameters
fn greet(name, greeting = "Hello")
    print greeting + ", " + name
end
```

### Control Flow

```ject
if condition
    # ...
elseif other
    # ...
else
    # ...
end

while condition do
    # ...
end

for item in array do
    # ...
end
```

### Collections

```ject
let arr = [1, 2, 3]
let dict = {name: "Alice", age: 30}
let tags = {|"rust", "python", "ject"|}  # Unique array (auto-deduplicates!)

# Access
print arr[0]        # 1
print arr[-1]       # 3 (last)
print dict["name"]  # "Alice"
print tags          # {|"rust", "python", "ject"|}
```

---

## Roadmap

- [x] Core language features
- [x] Rich standard library
- [x] Range syntax with steps
- [x] Enhanced REPL with history
- [x] Lambda functions
- [x] Module system
- [x] Struct system
- [x] **Advanced slicing** (named, range, Python-style)
- [ ] Package manager
- [ ] VSCode extension
- [ ] Advanced error handling
- [ ] Performance optimizations

---

## Inspiration

Ject draws from:
- **Crystal** - Elegant syntax
- **Ruby** - Expressiveness
- **Python** - User-friendly design
- **Rust** - Safety and performance

---

## License

MIT License

---

**Made with ❤️ in Rust**
