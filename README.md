# Ject

A scripting language built in Rust. Clean syntax, dynamic typing, first-class functions, and a rich standard library — no imports needed for most things.

```ject
fn greet(name, greeting = "Hello")
    print "$greeting, $name!"
end

greet("World")
greet("Leo", "Hey")

let numbers = [1, 2, 3, 4, 5]
let doubled = map(numbers, fn(n) -> n * 2)
print doubled  # [2, 4, 6, 8, 10]

for i in 1..6 do
    print "Count: $i"
end
```

---

## Installation

```bash
git clone https://github.com/riffifi/ject.git
cd ject
cargo build --release
```

Run a file:

```bash
./target/release/ject hello.ject
```

Start the REPL (with history):

```bash
./target/release/ject
```

---

## Language

### Variables and types

Dynamically typed. `let` declares, plain assignment reassigns.

```ject
let name = "Alice"
let version = 2.0
let active = true
let nothing = nil

name = "Bob"  # reassign, no let needed
```

### Functions

```ject
fn add(a, b)
    return a + b
end

# Default parameters
fn greet(name, greeting = "Hello")
    print "$greeting, $name!"
end

# Lambdas
let square = fn(x) -> x * x
let clamp = fn(v, lo, hi) -> max(lo, min(hi, v))
```

### Control flow

```ject
if score >= 90
    print "A"
elseif score >= 80
    print "B"
else
    print "F"
end

# if as an expression
let label = if score >= 60 then "Pass" else "Fail" end

while i < 10 do
    i = i + 1
end

for item in ["apple", "banana", "cherry"] do
    print item
end

for i in 0..10:2 do   # 0, 2, 4, 6, 8
    print i
end
```

### Arrays and slicing

Three equivalent slicing syntaxes — use whichever reads best:

```ject
let arr = [0, 1, 2, 3, 4, 5]

# Named parameters
arr[from:1 to:4]          # [1, 2, 3]
arr[from:0 to:6 step:2]   # [0, 2, 4]

# Range style
arr[1..4]                 # [1, 2, 3]
arr[0..6:2]               # [0, 2, 4]

# Python-style
arr[1:4]                  # [1, 2, 3]
arr[::2]                  # [0, 2, 4]
arr[-1]                   # 5 (last element)
```

**Unique arrays** — auto-deduplicate on push:

```ject
let tags = {|"rust", "ject", "rust"|}  # {|"rust", "ject"|}
tags = push(tags, "ject")              # no change
tags = push(tags, "go")                # {|"rust", "ject", "go"|}
```

### Strings

```ject
let name = "World"
print "Hello, $name!"
print "2 + 2 = ${2 + 2}"

split("a,b,c", ",")    # ["a", "b", "c"]
join(["a","b"], "-")   # "a-b"
upper("hello")         # "HELLO"
trim("  hi  ")         # "hi"
```

Slicing works on strings too: `"hello"[1:4]` gives `"ell"`.

### Structs

```ject
struct Point { x, y }

let p = new Point { x: 10, y: 20 }
print p.x   # 10
p.x = 99
```

### Dictionaries

```ject
let person = {name: "Alice", age: 30}
person["email"] = "alice@example.com"
print person["name"]
```

### Modules

```ject
import "math" as m
print m.sqrt(16)        # 4.0
print m.fibonacci(10)   # 55
print m.is_prime(17)    # true

import {PI, sin, cos} from "math"
print sin(PI / 2)       # 1.0

import "string" as s
print s.title_case("hello world")  # "Hello World"
```

Write your own:

```ject
# utils.ject
export PI = 3.14159

export fn circle_area(r)
    return PI * r * r
end
```

```ject
import "utils" as u
print u.circle_area(5)
```

### Error handling

```ject
fn divide(a, b)
    if b == 0 then
        throw "Division by zero"
    end
    return a / b
end

try
    print divide(10, 2)   # 5
    print divide(10, 0)   # throws
catch err
    print "Error: " + err
end
```

---

## Standard Library

No imports needed for most things. Math, arrays, strings, I/O — it's all there.

```ject
# Math
sqrt(16)         # 4.0
pow(2, 10)       # 1024
abs(-42)         # 42
floor(3.9)       # 3
round(3.5)       # 4

# Arrays
sum([1,2,3,4])            # 10
sort([3,1,2])             # [1,2,3]
reverse([1,2,3])          # [3,2,1]
filter([1,2,3,4], fn(x) -> x % 2 == 0)  # [2,4]
map([1,2,3], fn(x) -> x * x)            # [1,4,9]

# Strings
len("hello")              # 5
replace("hello", "l", "r")   # "herro"
repeat("ab", 3)           # "ababab"

# I/O
write_file("out.txt", "hello")
let content = read_file("out.txt")
let name = input("Name: ")

# System
let output = exec("ls")
let home = env("HOME")
```

The `math`, `string`, `array`, `io`, `json`, and `gui` modules extend this further. See [DOCS.md](DOCS.md) for the full reference.

---

## What's in the box

- Dynamic typing, clean syntax
- First-class functions and closures
- Three slicing syntaxes for arrays and strings
- Unique arrays `{| |}` with auto-deduplication
- Range expressions (`1..6`, `0..10:2`)
- String interpolation (`"Hello, $name!"`, `"${expr}"`)
- Module system with selective imports and exports
- Structs and dictionaries
- `try`/`catch`/`throw` error handling
- Built-in math, string, array, I/O, JSON
- NumPy-like numerical arrays (Rust-backed)
- Native GUI module
- REPL with history

---

## Roadmap

- [x] Core language
- [x] Rich standard library
- [x] Range syntax with steps
- [x] Lambda functions
- [x] Module system
- [x] Struct system
- [x] Advanced slicing (named, range, Python-style)
- [x] String interpolation
- [x] REPL with history
- [x] VS Code extension
- [ ] Package manager
- [ ] Performance optimizations

---

## VS Code Extension

```bash
cd vscode-ject
npm install
npm run package
code --install-extension ject-vscode-0.1.0.vsix
```

Supports `.ject` and `.jt` files: syntax highlighting, snippets, completions, hovers, formatting, run/check commands, and a REPL launcher.

---

## Documentation

Full language reference: [DOCS.md](DOCS.md)

---

MIT License — built with Rust
