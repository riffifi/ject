# Ject

A scripting language built in Rust. Clean syntax, dynamic typing, first-class functions, and a rich standard library — no imports needed for most things.

```ject
fn greet(name, greeting = "Hello")
    print "$greeting, $name!"
end

greet("World")
greet("Leo", "Hey")

let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map(fn(n) -> n * 2)
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

Or create and run a Ject package:

```bash
ject new hello
cd hello
ject run
ject check
ject test
```

Create a mixed Ject/Rust library with the same executable:

```bash
ject new my_native_lib --native
cd my_native_lib
ject build
```

The generated Ject facade imports its private `@native/my_native_lib` backend. Consumers
only import `"my_native_lib"`; local dependencies use
`my_native_lib = { path = "../my_native_lib" }` in `Ject.toml`.

Packages use `Ject.toml` and `src/main.ject` (or `src/lib.ject` for `ject new --lib`).
Install a local source or mixed library with `ject add name --path ../name`, then use
`ject install` to lock the transitive graph and build native components. CI can use
`ject install --locked` to reject missing or stale lockfiles without rewriting them.
The package/native-extension architecture and delivery plan are described in
[docs/PACKAGES.md](docs/PACKAGES.md).

Parse and lint only (does **not** execute code — used by the VS Code extension):

```bash
./target/release/ject --check hello.ject
```

Editors can instead start the persistent compiler-backed language server over
standard input/output. It analyzes unsaved in-memory documents without temporary files:

```bash
ject lsp
```

Print native-kernel metadata (for tooling; see `NATIVE_KERNEL.md`):

```bash
./target/release/ject --introspect
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

# Anonymous functions: fn(x) -> expr (a value, OCaml-let-style) or
# fn(x) ... end (block body -- the trailing bare expression is returned,
# no `return` needed). lambda(x) -> expr still works as an alias.
let square = fn(x) -> x * x
let clamp = fn(v, lo, hi)
    max(lo, min(hi, v))
end

# obj.method(args) is sugar for method(obj, args) when obj has no such
# member of its own -- arr.map(f), arr.push(x), arr.len() all just work.
print [1, 2, 3].map(fn(x) -> x * x)   # [1, 4, 9]
```

### Match

```ject
let grade = match score
    > 89     -> "A"
    > 79     -> "B"
    0..59    -> "F"
    _        -> "C"
end

# Multiple patterns per arm, and block bodies:
match command
    "quit", "exit", "q" -> exit(0)
    "help", "h" ->
        print "Available commands: ..."
        show_help()
    _ -> print "Unknown command"
end
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
print m.fibonacci(10)   # 55
print m.is_prime(17)    # true
print m.log(8, 2)       # 3.0  (log base 2 -- not in CorLib, needs this import)

import "string" as s
print s.title_case("hello world")  # "Hello World"
```

`sqrt`, `pow`, `sin`, `cos`, `PI`, and friends are already in CorLib -- no import needed for those; the `math`/`string`/etc. modules add the less-common stuff on top.

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
filter([1,2,3,4], lambda(x) -> x % 2 == 0)  # [2,4]
map([1,2,3], lambda(x) -> x * x)            # [1,4,9]

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

The `math`, `string`, `array`, `io`, `json`, `color`, `table`, and `jgui` modules extend this further. See [DOCS.md](DOCS.md) for the full reference.

---

## What's in the box

- Dynamic typing, clean syntax
- First-class functions, real closures, and pattern matching (`match` with relational/range/multi-value patterns)
- Method-call syntax sugar (`arr.map(f)`, `arr.push(x)`) alongside real member access (`import "math" as m; m.log(x, base)`)
- Three slicing syntaxes for arrays and strings
- Unique arrays `{| |}` with auto-deduplication
- Range expressions (`1..6`, `0..10:2`)
- String interpolation (`"Hello, $name!"`, `"${expr}"`)
- Module system with selective imports, singleton module state, and circular-import detection
- Structs and dictionaries
- `try`/`catch`/`throw` error handling (throw any value, not just strings)
- Built-in math, string, array, I/O, JSON
- NumPy-like numerical arrays (Rust-backed)
- Native JGUI module
- ANSI colors and table formatting (`color`, `table` -- pure Ject, no native code)
- Smarter REPL: auto-prints expression results, multi-line input for unfinished blocks, Ctrl+C interrupts a running script (not just line input)

---

## Roadmap

- [x] Core language
- [x] Rich standard library
- [x] Range syntax with steps
- [x] Lambda / anonymous functions
- [x] Pattern matching (`match`, with relational/range/multi-value patterns)
- [x] Module system
- [x] Struct system
- [x] Advanced slicing (named, range, Python-style)
- [x] String interpolation
- [x] REPL with history
- [x] VS Code extension
- [x] Local package manager (`add --path`, `remove`, `install`, lockfiles, native builds)
- [x] Remote package registries and publishing (immutable HTTP(S) archives)
- [ ] Performance optimizations (array/dict value semantics still copy-heavy)
- [x] AST source spans and runtime stack traces

---

## VS Code Extension

```bash
cd vscode-ject
npm install
npm run package
code --install-extension ject-vscode-0.3.0.vsix
```

Supports `.ject` and `.jt` files: syntax highlighting, snippets, completions, hovers, formatting, run/check commands, and a REPL launcher.

---

## Documentation

Full language reference: [DOCS.md](docs/DOCS.md)

---

MIT License — built with Rust
