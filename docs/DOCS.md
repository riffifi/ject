# Ject Language Reference — v0.3.0

## Table of Contents

1. [Getting Started](#getting-started)
2. [Syntax Basics](#syntax-basics)
3. [Types & Values](#types--values)
4. [Variables](#variables)
5. [Operators](#operators)
6. [Control Flow](#control-flow)
7. [Match](#match)
8. [Functions](#functions)
9. [Arrays](#arrays)
10. [List Comprehensions](#list-comprehensions)
11. [Strings](#strings)
12. [Dictionaries](#dictionaries)
13. [Collections (Sets)](#collections-sets)
14. [Structs](#structs)
15. [Modules](#modules)
16. [Error Handling](#error-handling)
17. [Input & Output](#input--output)
18. [File System](#file-system)
19. [System](#system)
20. [CorLib Reference](#corlib-reference)
21. [Standard Library Modules](#standard-library-modules)
22. [CLI Reference](#cli-reference)
23. [Architecture: CorLib vs Stdlib](#architecture-corlib-vs-stdlib)

---

## Getting Started

### Building

```bash
git clone https://github.com/riffifi/ject.git
cd ject
cargo build --release
```

### Running

```bash
ject script.ject          # run a file
ject                      # start the REPL
ject --check script.ject  # parse + lint only, no execution
ject --test script.ject   # run; exit non-zero on failure
ject --version            # print version
ject --introspect         # print native kernel metadata as JSON
ject --help               # show help
```

### File extensions

`.ject` and `.jt` are both recognized.

---

## Syntax Basics

### Comments

```ject
# single-line comment only
```

There are no multi-line comment blocks.

### Statement separation

Statements are separated by newlines. Semicolons also work for multiple statements on one line:

```ject
let a = 1
let b = 2; let c = 3
```

### Blocks

Blocks open with a keyword (`fn`, `if`, `while`, `for`, `try`, `match`) and always close with `end`. No indentation-based parsing.

```ject
if x > 0 then
    print x
end
```

---

## Types & Values

Ject is dynamically typed. Every value has one of these types:

| Type | Examples | `type_of` result |
|------|----------|-----------------|
| Integer | `0`, `42`, `-7` | `"integer"` |
| Float | `3.14`, `-0.5` | `"float"` |
| String | `"hello"` | `"string"` |
| Boolean | `true`, `false` | `"boolean"` |
| Nil | `nil` | `"nil"` |
| Array | `[1, 2, 3]` | `"array"` |
| Dictionary | `{name: "Alice"}` | `"dictionary"` |
| Collection | `collection([1, 2])` | `"collection"` |
| Struct instance | `new Point {x: 0}` | `"struct"` |
| Function / Lambda | `fn(x) -> x * 2` | `"function"` |
| Unique Array | see below | `"array"` |

-  {|a, b|}
### Type conversion

```ject
to_int("42")       # 42
to_int(3.9)        # 3  (truncates)
to_float("3.14")   # 3.14
to_float(5)        # 5.0
to_string(42)      # "42"
to_bool(0)         # false
to_bool(1)         # true
to_bool("")        # false
to_bool("hi")      # true
type_of(42)        # "integer"
```

---

## Variables

Declare with `let`. Reassign without `let`.

```ject
let name = "Alice"
let count = 0
count = count + 1
```

### Compound assignment operators

```ject
count += 1
count -= 5
count *= 2
count /= 4
count %= 3
```

### Increment / decrement

Both prefix and postfix forms work:

```ject
count++
count--
++count
--count
```

Variables are block-scoped. A variable declared inside an `if` or loop body is not accessible outside it.

---

## Operators

### Arithmetic

```ject
a + b    # addition / string concatenation
a - b    # subtraction
a * b    # multiplication
a / b    # division
a % b    # modulo
```

### Comparison

```ject
a == b   # equal
a != b   # not equal
a < b    # less than
a > b    # greater than
a <= b   # less than or equal
a >= b   # greater than or equal
```

### Logical

```ject
a and b  # logical AND
a or b   # logical OR
!a       # logical NOT
```

### Membership

```ject
"l" in "hello"       # true — substring check
2 in [1, 2, 3]       # true — array membership
"key" in {key: 1}    # true — dictionary key check
```

`in` works for strings, arrays, and dictionaries.

---

## Control Flow

### if / elseif / else

```ject
if score >= 90
    print "A"
elseif score >= 80
    print "B"
elseif score >= 70
    print "C"
else
    print "F"
end
```

`then` is optional but accepted after the condition.

### if as an expression

`if` can return a value. All branches must produce a value.

```ject
let label = if score >= 60 then "Pass" else "Fail" end
let sign  = if x > 0 then 1 elseif x < 0 then -1 else 0 end
```

### while

```ject
let i = 0
while i < 10 do
    print i
    i += 1
end
```

`do` is optional.

### for

Iterates over arrays, strings, ranges, and unique arrays:

```ject
for item in ["apple", "banana"] do
    print item
end

for i in 1..6 do
    print i       # 1 2 3 4 5
end

for i in 0..10:2 do
    print i       # 0 2 4 6 8
end

for ch in "hello" do
    print ch
end
```

`do` is optional.

### Ranges

```ject
1..6        # exclusive end: 1, 2, 3, 4, 5
0..10:2     # with step: 0, 2, 4, 6, 8
```

Ranges can be assigned to variables and passed to `sum`, `len`, etc.:

```ject
let r = 1..11
print sum(r)   # 55
```

### break and continue

```ject
while true do
    let line = input("> ")
    if line == "quit" then
        break
    end
    if line == "" then
        continue
    end
    print "got: " + line
end
```

---

## Match

`match` is an expression that compares a value against a series of patterns and returns the matching arm's value. Closes with `end`.

```ject
let day = "Monday"

let kind = match day
    "Saturday" -> "weekend"
    "Sunday"   -> "weekend"
    _          -> "weekday"
end

print kind   # weekday
```

Patterns can be literals (`42`, `"hello"`, `true`, `nil`), identifiers (bind the value to a name), or `_` (wildcard that matches anything).

`print` is also valid as a match arm body:

```ject
match score
    100 -> print "perfect"
    _   -> print "not perfect"
end
```

---

## Functions

### Definition

```ject
fn add(a, b)
    return a + b
end
```

A function with no `return` returns `nil`.

### Default parameters

```ject
fn greet(name, greeting = "Hello")
    print greeting + ", " + name + "!"
end

greet("Alice")          # Hello, Alice!
greet("Bob", "Hey")     # Hey, Bob!
```

### Keyword arguments at call site

Any parameter can be passed by name:

```ject
fn connect(host, port = 80, timeout = 30)
    print "connecting to $host:$port"
end

connect("example.com", timeout=60)
connect(port=443, host="example.com")
```

### Lambdas

`lambda` creates anonymous functions.

```ject
let add    = lambda(a, b) -> a + b
```

### Functions are first-class

```ject
fn double(x)
    return x * 2
end

let transform = double
print transform(5)    # 10

let ops = [fn(x) -> x + 1, fn(x) -> x * 2]
for op in ops do
    print op(3)
end
```

### Recursion

```ject
fn fib(n)
    if n <= 1 then return n end
    return fib(n - 1) + fib(n - 2)
end
```

---

## Arrays

### Creation

```ject
let empty  = []
let nums   = [1, 2, 3, 4, 5]
let mixed  = [1, "hello", true, nil]
let nested = [[1, 2], [3, 4]]
```

### Index access

```ject
nums[0]    # 1  (first)
nums[-1]   # 5  (last)
nums[-2]   # 4  (second to last)
```

### Index assignment

```ject
nums[0]      = 99   # direct mutation
grid[y][x]   = 1    # nested mutation works too
```

### Slicing — three equivalent syntaxes

**Named parameters:**
```ject
arr[from:1 to:4]           # [1, 2, 3]
arr[from:0 to:6 step:2]    # [0, 2, 4]
arr[from:5 to:0 step:-1]   # [5, 4, 3, 2, 1]
```

**Range syntax:**
```ject
arr[1..4]     # [1, 2, 3]
arr[0..6:2]   # [0, 2, 4]
```

**Python-style:**
```ject
arr[1:4]    # [1, 2, 3]
arr[::2]    # every other element
arr[1:]     # from index 1 to end
arr[:3]     # first 3 elements
arr[-3:]    # last 3 elements
```

### Unique arrays

Automatically deduplicate on creation and on `push`:

```ject
let tags = {|"rust", "ject", "rust"|}   # {|"rust", "ject"|}
tags = push(tags, "ject")               # no change
tags = push(tags, "go")                 # adds "go"
```

### Core array functions

```ject
len([1, 2, 3])                           # 3
push([1, 2], 3)                          # [1, 2, 3]
pop([1, 2, 3])                           # [1, 2]
sum([1, 2, 3, 4])                        # 10
first([10, 20, 30])                      # 10
last([10, 20, 30])                       # 30
reverse([1, 2, 3])                       # [3, 2, 1]
sort([3, 1, 2])                          # [1, 2, 3]
unique([1, 2, 2, 3])                     # [1, 2, 3]
contains([1, 2, 3], 2)                   # true
index_of([1, 2, 3], 2)                   # 1
slice([0,1,2,3,4], 1, 4)                 # [1, 2, 3]
map([1,2,3], fn(x) -> x * 2)            # [2, 4, 6]
filter([1,2,3,4], fn(x) -> x % 2 == 0)  # [2, 4]
reduce([1,2,3], fn(a,b) -> a + b, 0)    # 6
range(5)                                 # [0, 1, 2, 3, 4]
range(1, 6)                              # [1, 2, 3, 4, 5]
range(0, 10, 2)                          # [0, 2, 4, 6, 8]
```

---

## List Comprehensions

Build arrays from iterables in a single expression:

```ject
let squares = [x * x for x in 1..6]                  # [1, 4, 9, 16, 25]
let evens   = [x for x in 1..11 if x % 2 == 0]       # [2, 4, 6, 8, 10]
let upper   = [upper(w) for w in ["hi", "hello"]]     # ["HI", "HELLO"]
```

Syntax: `[expression for variable in iterable]` or `[expression for variable in iterable if condition]`.

---

## Strings

### Literals and escapes

```ject
let s = "Hello, World!"
let t = "line one\nline two"
let u = "tab\there"
let v = "with a quote: \""
```

### String interpolation

`$variable` for simple names, `${expression}` for anything else:

```ject
let name = "Alice"
let age  = 30
print "Hello, $name!"
print "In 5 years: ${age + 5}"
print "Type: ${type_of(name)}"
```

### Indexing and slicing

String indexing works exactly like array indexing:

```ject
"hello"[0]             # "h"
"hello"[-1]            # "o"
"hello"[1:4]           # "ell"
"hello"[from:1 to:4]   # "ell"
"hello"[-3:]           # "llo"
```

### Core string functions

```ject
len("hello")                   # 5
upper("hello")                 # "HELLO"
lower("HELLO")                 # "hello"
trim("  hi  ")                 # "hi"
split("a,b,c", ",")            # ["a", "b", "c"]
join(["a", "b"], "-")          # "a-b"
replace("hello", "l", "r")    # "herro"
char_at("hello", 1)            # "e"
substring("hello", 1, 4)       # "ell"
repeat("ab", 3)                # "ababab"
```

---

## Dictionaries

Key-value stores with string keys.

```ject
let person = {name: "Alice", age: 30, active: true}
let empty  = {}
```

### Access and mutation

```ject
person["name"]            # "Alice"
person["age"] = 31        # update existing key
person["email"] = "a@b"   # add new key
```

### Dictionary builtins

```ject
has_key(person, "name")   # true
delete(person, "email")   # returns new dict without the key
keys(person)              # array of keys, sorted
values(person)            # array of values in key-sorted order
```

---

## Collections (Sets)

A `Collection` is a hash set of unique string values. Useful when you need fast membership checks and automatic deduplication without order guarantees.

```ject
import "collections" as c

let fruits = collection(["apple", "banana", "apple"])
# contains only "apple" and "banana"
```

### Collection functions (require `import "collections"`)

```ject
collection(arr)              # create from array
add_to(s, "cherry")          # returns new collection with item added
remove_from(s, "banana")     # returns new collection with item removed
has(s, "apple")              # true/false
size(s)                      # element count
union(s1, s2)
intersect(s1, s2)
difference(s1, s2)           # in s1 but not s2
is_subset(s1, s2)
is_superset(s1, s2)
clear_collection(s)
to_array(s)
```

From `stdlib/collections.ject`:

```ject
c.from_array(arr)             # same as collection(arr)
c.is_empty_collection(coll)   # size(coll) == 0
```

---

## Structs

### Definition

```ject
struct Point { x, y }

struct Person {
    name,
    age,
    email
}
```

### Instantiation

```ject
let p     = new Point { x: 10, y: 20 }
let alice = new Person { name: "Alice", age: 30, email: "a@b.com" }
```

### Access and mutation

```ject
print p.x    # 10
p.x = 99
p.y += 5
```

---

## Modules

### Import styles

```ject
import "math"                    # all into scope directly
import "math" as m               # with alias
import {sqrt, PI} from "math"    # selective
```

### User modules

```ject
import "modules/utils"           # relative path
import "~/Documents/mylib"       # home-relative
import "/absolute/path/to/lib"   # absolute
```

The `.ject` extension is added automatically if omitted.

### Exporting

```ject
# utils.ject
export PI = 3.14159

export fn circle_area(r)
    return PI * r * r
end
```

Use `export fn` for functions, `export name = value` for constants.

### Available standard modules

| Module | Description |
|--------|-------------|
| `"math"` | Advanced math — trig, logs, primes, stats, geometry |
| `"string"` | Advanced string manipulation |
| `"array"` | Advanced array operations |
| `"io"` | File helpers, JSON file I/O |
| `"json"` | JSON validation, path access |
| `"system"` | Shell/CWD helpers |
| `"datetime"` | Timestamp helpers |
| `"util"` | Functional utilities, random float |
| `"collections"` | Set operations |
| `"numpy"` | Numerical arrays (Rust-backed) |
| `"gui"` | Native dialog windows (Rust-backed) |

---

## Error Handling

### throw

```ject
fn divide(a, b)
    if b == 0 then
        throw "division by zero"
    end
    return a / b
end
```

Any value can be thrown — strings are most common.

### try / catch

```ject
try
    let result = divide(10, 0)
    print result
catch err
    print "Error: " + err
end
```

`catch` can omit the variable name:

```ject
try
    risky()
catch
    print "something went wrong"
end
```

Execution continues normally after the block.

---

## Input & Output

### print

```ject
print "Hello"
print x, y, z
print "a", "b", "c" sep:", "    # a, b, c
print "no newline" end:""
print(42)                        # parenthesized form works too
```

### input

```ject
let name = input("Name: ")
let n    = to_int(input("Number: "))
```

---

## File System

Available globally without any import:

```ject
read_file("notes.txt")               # full content as string, or nil
write_file("out.txt", "hello")       # create or overwrite
append_file("log.txt", "new line\n") # append without overwriting
read_lines("data.txt")               # array of lines
file_exists("config.txt")            # true/false
is_file("notes.txt")                 # true if regular file
is_dir("/home/leo")                  # true if directory
```

---

## System

Available globally without any import:

```ject
exec("ls -la")      # run shell command, return stdout as string
env("HOME")         # get environment variable value
exit(0)             # exit program with status code
now()               # current time as float (Unix seconds)
timestamp()         # current Unix timestamp as integer
sleep(500)          # sleep for 500 milliseconds
```

---

## CorLib Reference

CorLib is always in scope — no import needed. All implemented in Rust.

### Type

| Function | Description |
|----------|-------------|
| `type_of(value)` | Type name as string |
| `to_int(value)` | Convert to integer (truncates floats) |
| `to_float(value)` | Convert to float |
| `to_string(value)` | Convert to string |
| `to_bool(value)` | Convert to boolean |

### Array

| Function | Description |
|----------|-------------|
| `len(value)` | Length of array or string |
| `range(stop)` / `range(start, stop)` / `range(start, stop, step)` | Integer array |
| `push(arr, item)` | New array with item appended |
| `pop(arr)` | New array with last item removed |
| `sum(arr)` | Sum of numeric elements |
| `contains(arr, item)` | Membership check |
| `index_of(arr, item)` | Index or -1 |
| `first(arr)` | First element |
| `last(arr)` | Last element |
| `slice(arr, start, end)` | Subarray |
| `sort(arr)` | Sorted copy |
| `reverse(arr)` | Reversed copy |
| `unique(arr)` | Deduplicated copy |
| `map(arr, fn)` | Apply fn to each element |
| `filter(arr, fn)` | Keep elements where fn is truthy |
| `reduce(arr, fn, initial)` | Fold left |

### String

| Function | Description |
|----------|-------------|
| `upper(str)` | Uppercase |
| `lower(str)` | Lowercase |
| `trim(str)` | Strip whitespace |
| `split(str, sep)` | Split by separator |
| `join(arr, sep)` | Join array into string |
| `replace(str, old, new)` | Replace all occurrences |
| `char_at(str, i)` | Character at index (negative supported) |
| `substring(str, start, end)` | Substring by index |
| `repeat(str, n)` | Repeat string |

### Math

| Function | Description |
|----------|-------------|
| `abs(x)` | Absolute value |
| `sqrt(x)` | Square root |
| `pow(base, exp)` | Exponentiation |
| `sin(x)` / `cos(x)` / `tan(x)` | Trig (radians) |
| `floor(x)` / `ceil(x)` / `round(x)` | Rounding |
| `min(a, b)` / `max(a, b)` | Min/max of two values |
| `random()` | Float in `[0.0, 1.0)` |
| `random_int(min, max)` | Integer in `[min, max)` |
| `PI` | 3.141592653589793 |
| `E` | 2.718281828459045 |

### I/O

| Function | Description |
|----------|-------------|
| `print ...` | Print with optional `sep:` and `end:` |
| `input(prompt)` | Read line from stdin |
| `read_file(path)` | File contents as string |
| `write_file(path, content)` | Create or overwrite |
| `append_file(path, content)` | Append to file |
| `read_lines(path)` | File as array of lines |
| `file_exists(path)` | Check existence |
| `is_file(path)` | Is regular file |
| `is_dir(path)` | Is directory |

### Dictionary

| Function | Description |
|----------|-------------|
| `has_key(dict, key)` | Check key existence |
| `delete(dict, key)` | New dict without key |
| `keys(dict)` | Array of keys (sorted) |
| `values(dict)` | Array of values (key-sorted order) |

### JSON

| Function | Description |
|----------|-------------|
| `parse_json(str)` | Parse JSON string |
| `to_json(value)` | Serialize to JSON string |

### Misc

| Function | Description |
|----------|-------------|
| `assert(cond)` | Throw if condition is falsy |
| `exec(cmd)` | Run shell command |
| `env(name)` | Get environment variable |
| `exit(code)` | Exit program |
| `now()` | Unix time as float |
| `timestamp()` | Unix time as integer |
| `sleep(ms)` | Sleep milliseconds |

---

## Standard Library Modules

All modules below are written in Ject and embedded in the binary at compile time via `include_str!()`. When you import one, the interpreter executes the `.ject` source with a small set of seed builtins pre-injected.

### math

```ject
import "math" as m
```

**Extra constants:**
```ject
m.PHI    # 1.618... golden ratio
m.SQRT2  # 1.414...
m.SQRT3  # 1.732...
```

**Logarithms** (not in CorLib — need this import):
```ject
m.log(x, base)
m.log10(x)
m.log2(x)
m.ln(x)
m.exp(x)
```

**Angle conversion:**
```ject
m.deg_to_rad(d)
m.rad_to_deg(r)
m.degrees(r)     # alias
m.radians(d)     # alias
```

**Inverse trig:**
```ject
m.asin(x)
m.acos(x)
m.atan(x)
m.atan2(y, x)
```

**Hyperbolic:**
```ject
m.sinh(x)
m.cosh(x)
m.tanh(x)
```

**Number theory (written in Ject):**
```ject
m.gcd(a, b)
m.lcm(a, b)
m.factorial(n)
m.fibonacci(n)
m.is_prime(n)
m.primes_up_to(n)
```

**Statistics (written in Ject):**
```ject
m.average(arr)
m.median(arr)
m.variance(arr)
m.stddev(arr)
m.product(arr)
```

**Combinatorics:**
```ject
m.permutations(n, k)
m.combinations(n, k)
```

**Geometry:**
```ject
m.distance_2d(x1, y1, x2, y2)
m.distance_3d(x1, y1, z1, x2, y2, z2)
m.dot_2d(x1, y1, x2, y2)
m.cross_2d(x1, y1, x2, y2)
m.magnitude(x, y)
m.normalize_2d(x, y)             # returns [nx, ny]
m.quadratic_roots(a, b, c)       # returns [], [r], or [r1, r2]
```

**Utilities:**
```ject
m.round_to(x, decimals)
m.clamp(value, min, max)
m.sign(x)                        # -1, 0, or 1
m.lerp(a, b, t)
m.remap(val, from_min, from_max, to_min, to_max)
m.is_even(n)
m.is_odd(n)
m.is_power_of_2(n)
m.next_power_of_2(n)
m.nth_root(x, n)
```

---

### string

```ject
import "string" as s
```

```ject
s.capitalize(str)
s.title_case(str)
s.trim_left(str)
s.trim_right(str)
s.pad_left(str, width, char)
s.pad_right(str, width, char)
s.pad_center(str, width, char)
s.starts_with(str, prefix)
s.ends_with(str, suffix)
s.contains_str(str, substr)
s.find(str, substr, start)
s.count(str, substr)
s.replace_all(str, old, new)
s.replace_first(str, old, new)
s.remove(str, substr)
s.reverse_str(str)
s.left(str, n)
s.right(str, n)
s.truncate(str, max_len)          # truncates with "..."
s.is_empty(str)
s.is_numeric(str)
s.is_alpha(str)
s.word_count(str)
s.sentence_count(str)
s.lines(str)                      # split on newlines → array
s.format(template, args)          # "{0} and {1}" with array
s.escape(str)
s.unescape(str)
s.wrap_text(str, width)
s.extract_numbers(str)
```

---

### array

```ject
import "array" as a
```

```ject
a.any(arr, fn)              # true if fn(item) for any item
a.all(arr, fn)              # true if fn(item) for all items
a.average(arr)
a.median(arr)
a.find(arr, fn)             # first element where fn is truthy
a.count(arr, fn)            # count where fn is truthy
a.take(arr, n)              # first n elements
a.drop(arr, n)              # all but first n
a.initial(arr)              # all but last
a.rest(arr)                 # all but first
a.concat(arr1, arr2)
a.zip(arr1, arr2)           # [[a0,b0], [a1,b1], ...]
a.union(arr1, arr2)
a.intersection(arr1, arr2)
a.difference(arr1, arr2)
a.flatten(arr)
a.chunk(arr, n)
a.shuffle(arr)
a.rotate_left(arr, n)
a.rotate_right(arr, n)
a.insert_at(arr, idx, item)
a.remove_at(arr, idx)
a.compact(arr)              # remove nil and false values
a.enumerate(arr)            # [[0, item0], [1, item1], ...]
a.fill_arr(count, value)
a.range_arr(start, end, step)
a.sample(arr)               # random element
a.sort_by(arr, key_fn)
```

---

### io

```ject
import "io"
```

```ject
io.write_lines(path, lines_arr)   # join with \n and write
io.read_json(path)                # read_file + parse_json
io.write_json(path, value)        # to_json + write_file
```

Also exposes: `file_exists`, `is_file`, `is_dir`, `append_file`, `read_lines`, `parse_json`, `to_json`.

---

### json

```ject
import "json" as j
```

```ject
j.to_json_pretty(value)          # pretty-printed JSON
j.is_valid_json(str)             # true if parseable
j.json_get(obj, "key.nested")    # dot-path access into parsed JSON
```

---

### system

```ject
import "system" as sys
```

```ject
sys.get_cwd()          # current working directory
sys.change_dir(path)   # change directory
```

Also exposes: `input`, `exec`, `env`, `exit`, `file_exists`, `is_file`, `is_dir`.

---

### util

```ject
import "util" as u
```

```ject
u.random_float(min, max)   # float in [min, max)
u.identity(x)
u.constant(value)          # returns fn(_) -> value
u.compose(f, g)            # fn(x) -> f(g(x))
u.apply(func, value)
u.is_nil(value)
u.is_truthy(value)
u.deep_equal(a, b)
u.copy(value)
```

Also exposes: `type_of`, `random`, `random_int`, `to_int`, `to_float`, `to_string`, `to_bool`.

---

### datetime

```ject
import "datetime" as dt
```

`now()`, `timestamp()`, and `sleep()` are globally available and don't require this import. The module provides placeholder helpers (`format_time`, `get_year`, `get_month`, `get_day`, `create_timestamp`) pending a proper date library implementation.

---

### gui

```ject
import {window, label, separator, button, input, run} from "gui"
```

Builds a native dialog window. `run(app)` blocks until the user closes the window and returns a result dictionary.

```ject
let app = window("My App", 560, 380)
label(app, "Fill in your details:")
separator(app)
input(app, "name", "Name:", "")
input(app, "email", "Email:", "")
separator(app)
button(app, "ok", "OK")
button(app, "cancel", "Cancel")

let result = run(app)
print result["buttons"]    # ["ok"]
print result["inputs"]     # {name: "Alice", email: "a@b.com"}
```

The GUI module is native-only (Rust/egui). It cannot be reimplemented in Ject itself.

---

### numpy

```ject
import "numpy" as np
```

Rust-backed numerical arrays. Native-only.

**Creation:** `np.array([...])`, `np.zeros(n)`, `np.ones(n)`, `np.arange(start, stop, step)`, `np.linspace(start, stop, n)`, `np.eye(n)`, `np.identity(n)`

**Properties:** `np.shape(arr)`, `np.ndim(arr)`, `np.size(arr)`

**Manipulation:** `np.reshape(arr, dims)`, `np.flatten(arr)`, `np.transpose(arr)`, `np.concatenate([a,b])`, `np.stack([a,b])`

**Element-wise math:** `np.sqrt`, `np.exp`, `np.log`, `np.abs`, `np.sin`, `np.cos`, `np.add`, `np.subtract`, `np.multiply`, `np.divide`

**Aggregation:** `np.sum`, `np.min`, `np.max`, `np.mean`, `np.std`, `np.var`

**Linear algebra:** `np.dot(a,b)`, `np.norm(arr)`, `np.det(m)`, `np.inv(m)`, `np.solve(A,b)`

---

## CLI Reference

```
ject                         Start REPL  (prompt: >> )
ject <file.ject>             Run a script
ject --check <file> [...]    Parse + lint only, no execution
ject --test <file> [...]     Run; exit non-zero on failure
ject --version               Print version
ject --introspect            Print native kernel metadata as JSON
ject --help                  Show help
```

---

## Architecture: CorLib vs Stdlib

Ject has a two-layer design:

**CorLib** — always in scope, no import needed, implemented in Rust. These are primitives that can't be expressed in Ject: type introspection, array/string operations, math primitives, I/O, system calls.

**Standard library** — written in Ject, embedded in the binary via `include_str!()`. When you `import "math"`, the interpreter loads and executes `stdlib/math.ject`, which builds on top of CorLib. The `.ject` source is readable and patchable without touching Rust.

The three exceptions that stay native-only: `"gui"`, `"base"`, `"numpy"` — they need direct access to native APIs.

```
import "math"
  → not native-only
  → load embedded stdlib/math.ject source
  → inject seed builtins (log, log10, asin, ...)
  → execute the Ject source
  → return the exported scope

import "gui"
  → native-only
  → return pre-built Rust map of builtin functions
```

---

## Quick Reference

### Keywords

```
let  fn  lambda  return
if  elseif  else  then  end
while  for  in  do
match
struct  new
try  catch  throw
import  export  from  as
break  continue
true  false  nil
print  and  or
```

### Operators

```
+  -  *  /  %            arithmetic
+=  -=  *=  /=  %=       compound assignment
++  --                   increment / decrement
==  !=  <  >  <=  >=     comparison
and  or  !               logical
in                       membership
..                       range (exclusive end)
->                       lambda arrow / match arm
```

### Common patterns

```ject
# iterate with index
let i = 0
for item in arr do
    print "${i}: $item"
    i += 1
end

# list comprehension
let doubled = [x * 2 for x in nums]
let big     = [x for x in nums if x > 10]

# match on value
let msg = match status
    200 -> "OK"
    404 -> "Not Found"
    _   -> "Unknown"
end

# safe dict access
let val = if has_key(d, "key") then d["key"] else "default" end

# functional chain
let result = reduce(
    filter(map(data, fn(x) -> x * 2), fn(x) -> x > 5),
    fn(a, b) -> a + b,
    0
)
```