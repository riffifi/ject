# The Ject Language Guide

Version 0.9.1

Ject is a small, dynamically typed scripting language implemented in Rust. It is
designed for readable programs, quick scripts, embeddable libraries, and a gradual
path from ordinary Ject code to high-performance Rust extensions.

This guide serves two purposes:

- Chapters 1–12 teach the language from the beginning.
- The later reference chapters collect syntax, built-ins, modules, CLI commands,
  packages, and native-extension details in one place.

Examples use the `.ject` file extension.

## Contents

1. [Install and run Ject](#1-install-and-run-ject)
2. [Your first program](#2-your-first-program)
3. [Values and variables](#3-values-and-variables)
4. [Operators and expressions](#4-operators-and-expressions)
5. [Control flow](#5-control-flow)
6. [Functions](#6-functions)
7. [Arrays, dictionaries, and collections](#7-arrays-dictionaries-and-collections)
8. [Strings](#8-strings)
9. [Structs](#9-structs)
10. [Errors](#10-errors)
11. [Modules](#11-modules)
12. [Packages](#12-packages)
13. [Standard library reference](#13-standard-library-reference)
14. [JGUI](#14-jgui)
15. [JNUM](#15-jnum)
16. [Mixed Ject and Rust libraries](#16-mixed-ject-and-rust-libraries)
17. [Command-line reference](#17-command-line-reference)
18. [Language reference](#18-language-reference)

## 1. Install and run Ject

### Build from source

Ject currently builds with Cargo:

```bash
git clone https://github.com/riffifi/ject.git
cd ject
cargo build --release
```

The executable is created at `target/release/ject`.

Check the installed version:

```bash
ject --version
```

### Run a file

Create `hello.ject`:

```ject
print "Hello, Ject!"
```

Run it:

```bash
ject hello.ject
```

The older file-oriented commands remain useful for individual scripts:

```bash
ject script.ject
ject --check script.ject
ject --test test_one.ject test_two.ject
```

### Use the REPL

Run `ject` without arguments:

```bash
ject
```

The REPL evaluates expressions immediately:

```text
>> 2 + 3
5
>> upper("hello")
"HELLO"
```

Use Ctrl+C to cancel the current input or interrupt a running loop. Use Ctrl+D or
enter `exit` to leave the REPL. Multi-line functions and blocks automatically use a
continuation prompt.

## 2. Your first program

Here is a complete small program:

```ject
fn greet(name, greeting = "Hello")
    return "$greeting, $name!"
end

let names = ["Ada", "Grace", "Linus"]

for name in names do
    print greet(name)
end
```

Important ideas in this example:

- `fn` defines a function.
- `let` creates a variable.
- Arrays use square brackets.
- `for ... do ... end` repeats a block.
- `$name` interpolates a variable into a string.
- Indentation makes code readable, but `end` is what closes a block.

### Comments and line endings

A single-line comment starts with `#`:

```ject
# This is a comment.
let answer = 42  # Comments may follow code.
```

Multi-line comments use `#*` and `*#` and may be nested:

```ject
#*
This whole section is ignored.
#* Nested comments work too. *#
*#
```

Newlines normally separate statements. Semicolons are accepted when several short
statements genuinely read better on one line:

```ject
let x = 1; let y = 2; print x + y
```

## 3. Values and variables

Ject is dynamically typed: variables do not have declared types, and a variable may
hold different kinds of values over its lifetime.

### Core value types

```ject
let count = 12                         # integer
let ratio = 0.75                       # float
let title = "Ject"                    # string
let enabled = true                    # boolean
let missing = nil                     # nil
let items = [1, 2, 3]                 # array
let user = {name: "Ada", active: true} # dictionary
```

`type_of(value)` returns the runtime type name:

```ject
print type_of(10)          # integer
print type_of(10.0)        # float
print type_of("hello")     # string
print type_of([1, 2])      # array
print type_of(nil)         # nil
```

Native libraries can introduce additional type names such as `ndarray`,
`jgui_window`, or a package-defined resource type.

### Declaring and assigning variables

Use `let` the first time a name is introduced:

```ject
let score = 10
score = score + 5
```

Assigning to an undeclared name is an error. This catches many spelling mistakes.

Compound assignments are available:

```ject
score += 5
score -= 2
score *= 3
score /= 2
score %= 4
```

Increment and decrement work in prefix or postfix form:

```ject
count++
++count
count--
--count
```

### Conversions

Use explicit conversion functions at data boundaries:

```ject
to_int("42")
to_float("3.5")
to_string(42)
to_bool(1)
```

Conversion failures produce runtime errors rather than silently inventing a value.

### Truthiness

Conditions accept any value. `false`, `nil`, zero, empty strings, and empty
containers behave as false-like values; non-empty and non-zero values behave as
true-like values. Prefer explicit comparisons when that makes intent clearer.

## 4. Operators and expressions

### Arithmetic

```ject
2 + 3       # 5
7 - 2       # 5
4 * 3       # 12
7 / 2       # 3.5
7 % 2       # 1
pow(2, 8)   # 256
```

Division returns a floating-point result. Division or modulo by zero is an error.

`+` also concatenates strings and arrays:

```ject
"hello" + " world"
[1, 2] + [3, 4]
```

### Comparison

```ject
a == b
a != b
a < b
a <= b
a > b
a >= b
```

### Logical operators

```ject
ready and connected
cached or fetch_value()
!finished
```

`and` and `or` short-circuit: the right side is evaluated only when needed.

### Membership

```ject
2 in [1, 2, 3]
"ect" in "Ject"
"name" in {name: "Ada"}
```

### Precedence

From tighter to looser binding, the commonly used groups are:

1. Calls, member access, and indexing
2. Unary `-` and `!`
3. `*`, `/`, `%`
4. `+`, `-`
5. Ranges
6. Comparisons and `in`
7. `and`
8. `or`
9. Conditional expressions

Use parentheses whenever they make an expression easier to read:

```ject
let total = (price + shipping) * quantity
```

## 5. Control flow

### `if`, `elseif`, and `else`

```ject
if score >= 90
    print "excellent"
elseif score >= 60
    print "passed"
else
    print "try again"
end
```

An `if` can also produce a value:

```ject
let label = if score >= 60 then "pass" else "fail" end
```

### `while`

```ject
let count = 3

while count > 0 do
    print count
    count -= 1
end
```

### `for`

Iterate over arrays, strings, collections, or ranges:

```ject
for item in ["a", "b", "c"] do
    print item
end

for i in 0..5 do
    print i
end
```

The end of a range is exclusive, so `0..5` produces `0, 1, 2, 3, 4`.

Ranges can have a step:

```ject
for even in 0..10:2 do
    print even
end

for descending in 5..0:-1 do
    print descending
end
```

`break` leaves the nearest loop. `continue` starts its next iteration:

```ject
for value in 0..10 do
    if value == 2
        continue
    end
    if value == 7
        break
    end
    print value
end
```

### `match`

`match` selects one arm and returns its value:

```ject
let description = match status
    200 -> "ok"
    400, 404 -> "client error"
    500 -> "server error"
    _ -> "unknown"
end
```

Range and comparison patterns are useful for classification:

```ject
let grade = match score
    > 89 -> "A"
    > 79 -> "B"
    0..60 -> "F"
    _ -> "C"
end
```

An arm may contain a block:

```ject
match command
    "save" ->
        write_file(path, content)
        print "saved"
    "quit", "exit" -> exit(0)
    _ -> print "unknown command"
end
```

## 6. Functions

### Named functions

```ject
fn add(a, b)
    return a + b
end

print add(2, 3)
```

If no explicit `return` runs, a function returns `nil`.

### Default parameters

```ject
fn greet(name, greeting = "Hello")
    return "$greeting, $name!"
end

greet("Ada")
greet("Ada", "Welcome")
```

Required parameters must come before parameters with defaults.

### Keyword arguments

Calls may name arguments:

```ject
greet(greeting: "Welcome", name: "Ada")
```

Keyword arguments make calls with several similar values easier to understand.

### Anonymous functions

Use the expression form for short functions:

```ject
let square = fn(value) -> value * value
print square(6)
```

Use a block for multi-step logic:

```ject
let classify = fn(value)
    if value < 0
        return "negative"
    end
    return "non-negative"
end
```

`lambda(...) -> ...` remains an alias for the short anonymous-function form.

### Closures

Functions remember variables from the scope where they were created:

```ject
fn multiplier(factor)
    return fn(value) -> value * factor
end

let triple = multiplier(3)
print triple(7)  # 21
```

### First-class functions

Functions can be stored, passed, and returned:

```ject
let doubled = map([1, 2, 3], fn(value) -> value * 2)
let positive = filter([-2, 0, 3], fn(value) -> value > 0)
let total = reduce([1, 2, 3], fn(a, b) -> a + b, 0)
```

### Method-call sugar

When a value has no actual member by a given name, Ject can rewrite:

```ject
items.map(transform)
```

as:

```ject
map(items, transform)
```

This is syntax convenience, not a separate method system. Real struct, dictionary,
and module members take priority.

## 7. Arrays, dictionaries, and collections

### Arrays

Arrays are ordered and mutable:

```ject
let colors = ["red", "green", "blue"]
print colors[0]
colors[1] = "emerald"
```

Negative indexes count from the end:

```ject
print colors[-1]
```

Assignment and function calls share the same array object. Operations such as
`push`, `map`, `filter`, `sort`, and slicing return a new array rather than modifying
the input.

Common operations:

```ject
len(items)
push(items, value)
pop(items)
first(items)
last(items)
contains(items, value)
index_of(items, value)
reverse(items)
sort(items)
unique(items)
flatten(nested)
```

### Slicing

Ject supports three equivalent slicing styles. Bounds are start-inclusive and
end-exclusive:

```ject
let values = [0, 1, 2, 3, 4, 5]

values[1:4]
values[1..4]
values[from:1 to:4]

values[::2]
values[0..6:2]
values[from:0 to:6 step:2]
```

### List comprehensions

Build an array by transforming and optionally filtering another iterable:

```ject
let squares = [x * x for x in 0..6]
let even_squares = [x * x for x in 0..10 if x % 2 == 0]
```

### Unique arrays

Unique arrays preserve insertion order while preventing duplicate values:

```ject
let tags = {|"rust", "ject", "rust"|}
print tags  # {|"rust", "ject"|}
```

Use `to_uarray(array)` and `to_array(unique_array)` to convert between forms.

### Dictionaries

Dictionaries map string keys to values:

```ject
let person = {
    name: "Ada",
    age: 36,
    active: true
}

print person["name"]
person["language"] = "Ject"
```

Like arrays, assignment and function calls share the same dictionary object. Cloning
a dictionary value is constant-time; mutations through an alias remain visible.
Helpers such as `delete` that document a returned dictionary still create a new one.

Identifier-like keys may use member syntax:

```ject
print person.name
person.age = 37
```

Useful dictionary functions:

```ject
keys(person)
values(person)
has_key(person, "name")
delete(person, "active")
len(person)
```

### Collections (sets)

The `collections` module provides set-oriented operations:

```ject
import "collections" as sets

let a = sets.from_array(["a", "b"])
let b = sets.from_array(["b", "c"])

sets.union(a, b)
sets.intersect(a, b)
sets.difference(a, b)
sets.has(a, "a")
sets.to_array(a)
```

## 8. Strings

Strings are UTF-8 text values:

```ject
let language = "Ject"
let escaped = "first line\nsecond line"
```

Common escape sequences include `\n`, `\t`, `\r`, `\\`, and `\"`.

### Interpolation

Use `$name` for a variable and `${expression}` for a full expression:

```ject
let name = "Ada"
print "Hello, $name"
print "2 + 3 = ${2 + 3}"
```

### Indexing and slicing

```ject
"hello"[1]    # "e"
"hello"[1:4]  # "ell"
"hello"[-1]   # "o"
```

### Common string functions

```ject
len(text)
upper(text)
lower(text)
trim(text)
split(text, separator)
join(parts, separator)
replace(text, old, new)
substring(text, start, end)
contains(text, fragment)
starts_with(text, prefix)
ends_with(text, suffix)
repeat(text, count)
char_at(text, index)
```

The `string` module adds higher-level helpers such as capitalization, title casing,
padding, word operations, and validation predicates.

## 9. Structs

Structs describe records with known fields:

```ject
struct Point { x, y }

let point = new Point { x: 10, y: 20 }
print point.x
point.x = 15
```

Fields not supplied during construction receive `nil`.

Structs are useful when a dictionary would be too loose and the program benefits
from a named shape. Functions that accept a struct can still be used with method-call
sugar:

```ject
fn distance_from_origin(point)
    return sqrt(pow(point.x, 2) + pow(point.y, 2))
end

print point.distance_from_origin()
```

## 10. Errors

### Throwing an error

```ject
fn divide(a, b)
    if b == 0
        throw "cannot divide by zero"
    end
    return a / b
end
```

Any Ject value can be thrown, though strings or structured dictionaries usually
produce the clearest errors.

### Catching errors

```ject
try
    let result = divide(10, 0)
    print result
catch error
    print "Failed: $error"
end
```

Use exceptions for exceptional failures. For ordinary expected choices, returning
`nil`, a boolean, or a result dictionary may be easier for callers.

### Assertions

Assertions are useful in tests and at API boundaries:

```ject
assert(total >= 0, "total must not be negative")
```

### Diagnostics

Ject uses one diagnostic format everywhere: parsing, linting, execution, packages,
and native libraries. Diagnostics are written to standard error, while normal
program output remains on standard output. This makes shell redirection and editor
integration predictable.

```text
error[E3001]: undefined variable `totla`
 --> src/main.ject:2:7
   |
 2 | print totla
   |       ^^^^^ not found in this scope
   = note: names are case-sensitive
   = help: did you mean `total`?
```

Every diagnostic consists of a severity, a stable code, a short explanation, and,
when useful, a source label, note, and actionable help. Color is enabled only when
standard error is a terminal, so redirected output contains no escape sequences.

Runtime diagnostics retain source spans from the positioned AST. The primary marker
points to the expression that produced the error, including operator expressions
such as division by zero. When an error crosses named Ject functions, the diagnostic
also prints the call chain as `at function_name` notes, innermost frame first.
Frames defined in imported files retain that module's canonical filename; the CLI
loads the corresponding source when rendering the primary error marker.

Code families identify the subsystem:

| Range | Meaning |
|---|---|
| `E11xx` | Syntax and parser errors |
| `E20xx` / `W20xx` | Static-analysis errors and warnings |
| `E30xx` | Runtime values, calls, indices, and operations |
| `E31xx` | Imports and modules |
| `E32xx` | Native calls and ABI failures |
| `E40xx` | Command usage, packages, and files |
| `E41xx` / `E42xx` | Native package builds and loading |

Treat a code as a searchable identifier; the explanatory text may improve between
releases. `ject check` parses and lints without executing user code and returns a
non-zero status if it finds an error. Warnings are shown with a summary but do not
make checking fail. Runtime and parser errors also return a non-zero status when a
file or package command is used. The REPL reports the same diagnostics but remains
open so the next expression can be entered.

## 11. Modules

A module is a `.ject` file that explicitly exports values.

`math_utils.ject`:

```ject
export PI = 3.14159

export fn circle_area(radius)
    return PI * radius * radius
end
```

Import with an alias:

```ject
import "./math_utils" as math
print math.circle_area(5)
```

Import selected names:

```ject
import {circle_area, PI} from "./math_utils"
```

Import every export into the current scope:

```ject
import "./math_utils"
```

Aliases are recommended for larger modules because they make the origin of a name
obvious.

### Module paths

```ject
import "./sibling"       # relative to the importing file
import "../shared/util"  # relative parent path
import "/absolute/path"  # absolute filesystem path
import "~/my_lib"        # home-relative path
import "math"            # package or standard module
```

Relative imports always resolve against the importing module, not the process's
current directory.

Modules execute once per interpreter. Later imports reuse their cached exports.
Circular imports produce a diagnostic rather than recursing forever.

## 12. Packages

A package is a directory containing `Ject.toml`.

Create an application:

```bash
ject new hello
cd hello
ject run
```

The generated layout is:

```text
hello/
  Ject.toml
  src/main.ject
```

Create a library:

```bash
ject new useful_math --lib
```

Libraries conventionally use `src/lib.ject` and export their public API.

### Manifest

```toml
[package]
name = "my_app"
version = "0.1.0"
edition = "2026"
entry = "src/main.ject"

[dependencies]
useful_math = { path = "../useful_math" }
```

Then import the dependency by its manifest key:

```ject
import "useful_math" as math
```

Path dependencies and their transitive path dependencies are resolved before the
program starts. Native artifacts in that graph are loaded automatically.

### Package commands

```bash
ject run
ject check
ject test
ject add useful_math --path ../useful_math
ject add colors --version '^1.2' --registry https://registry.example
ject add parser_tools --git https://example.com/parser_tools.git --branch main
ject install
ject install --locked
ject update [package]
ject remove useful_math
ject build
ject build --release
```

- `run` executes the package entry.
- `check` parses and lints the package entry.
- `test` runs every `tests/*.ject` file in sorted order.
- `add` validates and records a local library or selects the newest registry release
  matching a SemVer requirement, then refreshes the lockfile.
- `install` resolves the graph, writes `Ject.lock`, and builds native parts.
- `install --locked` verifies the committed dependency graph and SHA-256 package
  checksums without updating the lockfile, which is the recommended CI mode.
- `update` refreshes all direct registry dependencies, or one named dependency,
  within their saved SemVer requirements. It also advances Git dependencies that
  track HEAD, a branch, or a tag; dependencies added with `--rev` stay pinned.
- `remove` deletes a dependency and refreshes the lockfile.
- `build` checks Ject source and builds native components when present.

Commit `Ject.lock` for applications. The manifest preserves a registry dependency's
SemVer requirement and current exact selection. Installation remains deterministic;
only `ject update` changes the selection. Releases use immutable,
checksum-verified archives:

```bash
ject add colors --version '^1.2' --registry https://registry.example
ject update colors
ject publish --registry https://registry.example
```

Set `JECT_REGISTRY` for the default URL and `JECT_REGISTRY_TOKEN` when publishing to
an authenticated registry. `file://` URLs are supported for local registries.

Git dependencies are cloned into Ject's shared cache and pinned to a full commit ID
in both `Ject.toml` and `Ject.lock`. Use exactly one of `--branch`, `--tag`, or
`--rev`; without a selector, the dependency tracks the remote HEAD. Cached source is
content-verified before use, just like registry packages.

## 13. Standard library reference

Ject has two library layers:

- CorLib contains primitives that are always in scope.
- Standard modules are imported explicitly and are mostly written in Ject.

### CorLib: type and conversion

| Function | Purpose |
|---|---|
| `type_of(value)` | Return the runtime type name |
| `to_int(value)` | Convert to an integer |
| `to_float(value)` | Convert to a float |
| `to_string(value)` | Convert to text |
| `to_bool(value)` | Convert to a boolean |

### CorLib: arrays and iteration

| Function | Purpose |
|---|---|
| `len(value)` | Length of a string or container |
| `range(...)` | Construct a numeric range as values |
| `push(array, value)` | Return an array with a value appended |
| `pop(array)` | Return an array without its final value |
| `first(array)` / `last(array)` | First or last item |
| `slice(value, start, end, step)` | Slice an array or string |
| `map(array, function)` | Transform every element |
| `filter(array, predicate)` | Keep matching elements |
| `reduce(array, function, initial)` | Fold elements into one value |
| `sort(array)` / `reverse(array)` | Return reordered arrays |
| `flatten(array)` | Flatten nested arrays |
| `unique(array)` | Remove repeated values |
| `contains(container, value)` | Test membership |
| `index_of(container, value)` | Find an item index |

### CorLib: strings

| Function | Purpose |
|---|---|
| `upper` / `lower` | Change letter case |
| `trim` | Remove surrounding whitespace |
| `split` / `join` | Split text or join strings |
| `replace` | Replace text fragments |
| `substring` | Extract a text range |
| `char_at` | Read one character |
| `starts_with` / `ends_with` | Test prefixes and suffixes |
| `repeat` | Repeat text |

### CorLib: mathematics

Constants available globally include `PI`, `E`, `inf`, and `nan`.

Functions include:

```text
abs  ceil  floor  round  sqrt  pow
min  max  sum
sin  cos  tan
random  random_int
```

### CorLib: dictionaries and JSON

```text
keys  values  has_key  delete
to_json  parse_json
```

### CorLib: I/O, files, and processes

```text
print  input
read_file  write_file  append_file  read_lines
file_exists  is_file  is_dir
exec  exit
```

File and process functions operate with the permissions of the Ject process. Treat
untrusted paths and command strings carefully.

### CorLib: testing and utilities

```text
assert  now  timestamp  sleep
```

### Standard modules

Import standard modules by name:

```ject
import "math" as math
import "string" as strings
import "array" as arrays
import "io" as io
import "json" as json
import "system" as system
import "util" as util
import "datetime" as datetime
import "collections" as sets
import "color" as color
import "table" as table
```

#### `math`

Adds helpers beyond the global math primitives, including logarithms with a base,
number predicates, combinatorics, and common sequences.

Representative use:

```ject
import "math" as math

math.log(8, 2)
math.fibonacci(10)
math.factorial(5)
math.gcd(24, 18)
math.lcm(6, 8)
math.is_prime(97)
math.clamp(value, 0, 100)
```

#### `string`

Adds higher-level text helpers:

```ject
import "string" as strings

strings.capitalize("hello")
strings.title_case("hello world")
strings.is_alpha("Ject")
strings.is_numeric("123")
strings.reverse_str("abc")
```

#### `array`

Adds conversions and array-oriented convenience operations. Core functional
operations such as `map`, `filter`, and `reduce` remain globally available.

#### `io` and `json`

These modules group file and JSON operations for namespaced use. The underlying
primitives are also available globally where documented above.

#### `system`

Groups process, environment, input, and filesystem inspection helpers.

#### `util`

Contains general helpers built on conversion, random, and type primitives.

#### `datetime`

Groups time-related functions. `now`, `timestamp`, and `sleep` are also global.
Calendar extraction and formatting remain intentionally modest in 0.8.

#### `collections`

Provides set-like collections:

```text
from_array  to_array  add_to  remove_from  has
union  intersect  difference
is_subset  is_superset  size  clear_collection
```

#### `color`

Provides color creation, conversion, and formatting helpers.

#### `table`

Provides helpers for constructing and formatting text tables.

## 14. JGUI

JGUI is a mixed library. Its public API and convenience dialogs are Ject source;
the private Rust backend uses egui/eframe for operating-system windows. Add it to a
project before importing it:

```bash
ject add jgui --path packages/jgui
```

```ject
import "jgui" as gui

let widgets = [
    gui.heading("Profile"),
    gui.text_input("name", "Name", ""),
    gui.button("save", "Save", true)
]
let result = gui.run("Profile", widgets, 560, 380)
print result.values
```

### JGUI functions

| Function | Purpose |
|---|---|
| `heading(text)`, `label(text)`, `separator()` | Create display widgets |
| `text_input(id, label, initial="", on_change=nil)` | Create a single-line input |
| `multiline(id, label, initial="", on_change=nil)` | Create a multiline input |
| `checkbox(id, text, checked=false, on_change=nil)` | Create a checkbox |
| `slider(id, text, value=0, minimum=0, maximum=100, on_change=nil)` | Create a slider |
| `button(id, text, closes_window=false, on_click=nil)` | Create a button |
| `progress(value, text="")`, `spacer(points=8)` | Create layout/status widgets |
| `run(title, widgets, width=680, height=560)` | Run a document and return its state |
| `message(title, text)`, `confirm(title, question)` | Convenience dialogs |

Application code cannot import `@native/jgui`; only the public JGUI facade may
access that implementation module. The package uses a blocking, declarative window
lifecycle with the document API: `heading`, `label`,
`text_input`, `multiline`, `checkbox`, `slider`, `progress`, `button`, and `run`.
Its input widgets accept an optional `on_change` callback and buttons accept an
optional `on_click` callback. Rust invokes these callbacks synchronously through ABI
v2 with one event dictionary containing `kind`, `id`, `value`, and the current
`values` dictionary. Callback failures close the window and return as ordinary Ject
errors. This keeps widget construction and event policy in Ject while Rust owns only
rendering and operating-system integration.

## 15. JNUM

JNUM is Ject's native numerical-array package. Declare it in `Ject.toml` with
`ject add jnum ...` before importing it:

```ject
import "jnum" as numbers

let data = numbers.array([1, 2, 3, 4])
print numbers.mean(data)
print numbers.shape(data)
```

Its public Ject facade provides defaults and convenience wrappers. Rust owns
numerical storage and kernels. Array values have runtime type `ndarray`.

### Creation

```text
array  zeros  ones
arange  linspace  eye  identity
```

### Shape and manipulation

```text
shape  ndim  size
reshape  flatten  transpose
concatenate  stack
```

### Element-wise operations

```text
sqrt  exp  log  log10  abs
ceil  floor  round  clip
sin  cos  tan  arcsin  arccos  arctan  arctan2
degrees  radians
sinh  cosh  tanh
```

### Reductions and ordering

```text
sum  mean  std  var  min  max
argmin  argmax  cumsum  any  all
sort  argsort
```

### Linear algebra and comparisons

```text
dot  inner  outer  matmul  trace  diag
logical_and  logical_or  logical_not
greater  less  equal  not_equal  where_cond
```

### Random values and constants

```text
random  rand  randint
PI  E  INF  NAN
```

Not every NumPy behavior is implied by the JNUM name. Consult tests and runtime
errors for supported dimensions and argument forms; JNUM is an evolving native
library, not a compatibility promise with Python NumPy.

The installable JNUM package keeps this API in `src/lib.ject` and implements only
array storage and numerical kernels in its Rust plugin. Its arrays are opaque
plugin-owned resources. Use `to_array(value)` for an explicit conversion back to a
Ject array. This avoids repeatedly serializing numerical data across the native ABI.
Non-finite floats use dedicated ABI wire tags, so `NAN` and `INF` retain normal
floating-point behavior instead of becoming strings or errors.

## 16. Mixed Ject and Rust libraries

Create a mixed package:

```bash
ject new native_greeter --native
cd native_greeter
ject build
```

Generated layout:

```text
native_greeter/
  Ject.toml
  src/lib.ject
  native/Cargo.toml
  native/src/lib.rs
```

### Public facade

`src/lib.ject` is the API consumers import:

```ject
import "@native/native_greeter" as native

export fn hello(name)
    assert(type_of(name) == "string", "hello requires a string")
    return native.hello(name)
end
```

`@native/...` is package-private. It is available to the owning facade but rejected
when application code tries to bypass that facade.

### Rust backend

The Rust crate builds as a `cdylib` and uses `ject-native`:

```rust
use serde_json::{json, Value};

fn call(function: &str, args: Vec<Value>) -> Result<Value, String> {
    match function {
        "hello" => {
            let name = args
                .first()
                .and_then(Value::as_str)
                .ok_or_else(|| "hello expects a string".to_string())?;
            Ok(json!(format!("Hello, {name}!")))
        }
        _ => Err(format!("unknown function '{function}'")),
    }
}

ject_native::ject_plugin!("native_greeter", ["hello"], call);
```

The macro exports one stable symbol, `ject_plugin_entry_v1`. Ject reads its module
name and export list at runtime. No interpreter match arm is added for the package.

`ject new <name> --native` vendors the small `ject-native` SDK under
`native/ject-native` and uses a relative Cargo path dependency. Keep that directory
in version control. It makes the mixed package self-contained: publishing and a
subsequent `ject install --locked` do not depend on the original Ject source checkout.

### ABI values

The native ABI supports:

- `nil`
- booleans
- integers and floats, including tagged NaN and infinities
- strings
- arrays
- dictionaries
- opaque native resources
- structured success and error envelopes

Rust layouts never cross the dynamic-library boundary. Values are encoded using the
versioned ABI wire format.

### Native resources

A plugin can return an opaque resource marker:

```rust
Ok(ject_native::resource(id, "database_connection"))
```

Ject turns it into a typed native value containing the owning module and opaque ID.
Only that module can receive it again. When the final Ject reference disappears,
the SDK calls the handler with `__drop_resource` so Rust can release the object.

See `examples/native_double` and `examples/native_double_demo` for a complete,
tested library/consumer pair containing a normal function, an opaque counter resource,
and a Rust-to-Ject callback.

ABI v2 plugins use `ject_native::ject_plugin_v2!` and declare
`abi = "ject-native-2"` in `Ject.toml`. Callable Ject arguments arrive as
`$ject_callback` handles and can be invoked synchronously with
`ject_native::invoke_callback`. Handles must not be retained after the native export
returns or called from another thread. ABI v1 remains supported for plugins that do
not need callbacks.

```rust
fn call(name: &str, args: Vec<Value>, host: *const ject_native::HostV1)
    -> Result<Value, String>
{
    let callback = ject_native::callback_id(&args[0]).ok_or("expected a callback")?;
    // SAFETY: `host` is used synchronously during this ABI v2 call.
    unsafe { ject_native::invoke_callback(host, callback, vec![args[1].clone()]) }
}

ject_native::ject_plugin_v2!("events", ["apply"], call);
```

### Trust model

Native libraries are in-process dynamic libraries. They are appropriate
for trusted local code and have the same operating-system access as Ject itself.
Registry archives and lockfiles protect integrity and reproducibility, but do not
sandbox code. Capability enforcement, signatures, and a sandboxed
WebAssembly provider remain post-0.9 research. Do not load untrusted native packages.

## 17. Command-line reference

```text
ject
ject <file.ject>
ject --check <file.ject> [...]
ject --test <file.ject> [...]
ject --version
ject --introspect
ject lsp
ject --help

ject new <name> [--lib | --native]
ject init [--lib | --native]
ject run
ject check
ject test
ject build [--release]
```

| Command | Description |
|---|---|
| `ject` | Start the REPL |
| `ject file.ject` | Run one file |
| `ject --check files...` | Parse and lint files without running them |
| `ject --test files...` | Run explicit test scripts |
| `ject new name` | Create an application package |
| `ject new name --lib` | Create a source library |
| `ject new name --native` | Create a mixed Ject/Rust library |
| `ject init` | Initialize the current directory |
| `ject run` | Run the current package entry |
| `ject check` | Check the current package entry |
| `ject test` | Run `tests/*.ject` |
| `ject build` | Check source and build native dependencies |
| `ject --introspect` | Print native-kernel metadata as JSON |
| `ject lsp` | Start the Language Server Protocol server over standard input/output |

Commands search the current directory and its parents for `Ject.toml`.

## 18. Language reference

### Keywords

```text
let  fn  lambda  return
if  then  elseif  else  end
while  for  in  do  break  continue
match
true  false  nil
and  or  not
struct  new
try  catch  throw
import  export  from  as
print
```

### Delimiters

```text
( )     calls and grouping
[ ]     arrays, indexing, and slicing
{ }     dictionaries and struct fields
{| |}   unique arrays
,       item separator
:       key/value, keyword argument, or slice separator
.       member access and method-call sugar
```

### Operators

```text
+  -  *  /  %
==  !=  <  <=  >  >=
and  or  not  in
=  -=  *=  /=  %=
++  --
..
```

### Block forms

```ject
if condition
    statements
end

while condition do
    statements
end

for value in iterable do
    statements
end

fn name(parameters)
    statements
end

try
    statements
catch error
    statements
end
```

### Recommended style

- Use four spaces for indentation.
- Prefer one statement per line.
- Use `snake_case` for variables and functions.
- Use `PascalCase` for struct names.
- Import modules with short, meaningful aliases.
- Keep native APIs small and place validation and convenience behavior in Ject.
- Use `ject check` while editing and `ject test` before committing.

### Current boundaries

Ject 0.9 deliberately does not claim completed support for remote package registries,
publishing, capability sandboxes, or native callbacks. The implemented local package,
lockfile, diagnostics, and native ABI behavior is documented above. Planned architecture is
described separately in [PACKAGES.md](PACKAGES.md).
