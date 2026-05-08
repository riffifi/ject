# Ject Language Reference

A complete reference for the Ject programming language.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Syntax Basics](#syntax-basics)
3. [Types & Values](#types--values)
4. [Variables](#variables)
5. [Operators](#operators)
6. [Control Flow](#control-flow)
7. [Functions](#functions)
8. [Arrays](#arrays)
9. [Strings](#strings)
10. [Dictionaries](#dictionaries)
11. [Structs](#structs)
12. [Modules](#modules)
13. [Error Handling](#error-handling)
14. [Input & Output](#input--output)
15. [File System](#file-system)
16. [System](#system)
17. [Built-in Functions Reference](#built-in-functions-reference)
18. [Standard Library Modules](#standard-library-modules)
19. [GUI Module](#gui-module)
20. [NumPy Module](#numpy-module)

---

## Getting Started

### Installation

```bash
git clone https://github.com/riffifi/ject.git
cd ject
cargo build --release
```

### Running a file

```bash
./target/release/ject hello.ject
```

### REPL

Running without arguments starts the interactive REPL with history support:

```bash
./target/release/ject
```

### File extensions

Ject uses `.ject` or `.jt` for source files.

---

## Syntax Basics

### Comments

```ject
# This is a single-line comment
```

Ject only has single-line comments. There are no multi-line comment blocks.

### Statements

Statements are separated by newlines. Semicolons can also be used as separators if you want multiple statements on one line:

```ject
let a = 1
let b = 2
let c = 3; let d = 4
```

### Blocks

Blocks are opened with context-dependent keywords (`if`, `fn`, `while`, `for`, `struct`) and always closed with `end`. There is no indentation-based parsing — blocks are explicit.

```ject
if condition
    do_something()
end
```

---

## Types & Values

Ject is dynamically typed. The available value types are:

| Type | Examples |
|------|----------|
| Integer | `0`, `42`, `-7` |
| Float | `3.14`, `-0.5`, `1.0` |
| String | `"hello"`, `""` |
| Boolean | `true`, `false` |
| Nil | `nil` |
| Array | `[1, 2, 3]` |
| Unique Array | `{|"a", "b"|}` |
| Dictionary | `{name: "Alice", age: 30}` |
| Struct instance | `new Point { x: 0, y: 0 }` |
| Function | `fn(x) -> x * 2` |

### Type checking

```ject
type_of(42)        # "integer"
type_of(3.14)      # "float"
type_of("hello")   # "string"
type_of(true)      # "boolean"
type_of(nil)       # "nil"
type_of([1, 2])    # "array"
type_of({a: 1})    # "dictionary"
```

### Type conversions

```ject
to_int("42")       # 42
to_int(3.9)        # 3 (truncates)
to_float("3.14")   # 3.14
to_float(5)        # 5.0
to_string(42)      # "42"
to_bool(0)         # false
to_bool(1)         # true
to_bool("")        # false
to_bool("hi")      # true
```

---

## Variables

Variables are declared with `let`. They are mutable and can be reassigned freely.

```ject
let name = "Alice"
let age = 30
let active = true
let nothing = nil
```

Reassignment doesn't need `let`:

```ject
let x = 10
x = 20
```

Variables are scoped to their block. A variable declared inside an `if` or loop body is not accessible outside it.

---

## Operators

### Arithmetic

```ject
a + b    # addition (also string concatenation)
a - b    # subtraction
a * b    # multiplication
a / b    # division (returns float if either operand is float)
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

### String concatenation

The `+` operator concatenates strings. Non-string values can be appended by converting them:

```ject
print "Score: " + score           # works if score is a number
print "Values: " + to_string(arr) # explicit conversion for arrays
```

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

The `then` keyword is optional but accepted:

```ject
if x > 0 then
    print "positive"
end
```

### Inline if expression

`if` can also be used as an expression, returning a value:

```ject
let label = if score >= 60 then "Pass" else "Fail" end
let sign = if x > 0 then 1 elseif x < 0 then -1 else 0 end
```

### while loop

```ject
let i = 0
while i < 10 do
    print i
    i = i + 1
end
```

The `do` keyword is optional.

### for loop

Iterates over arrays, ranges, or strings:

```ject
# Over an array
for item in ["apple", "banana", "cherry"] do
    print item
end

# Over a range
for i in 1..6 do
    print i   # prints 1 through 5
end

# Over range with step
for i in 0..10:2 do
    print i   # prints 0, 2, 4, 6, 8
end
```

The `do` keyword is optional.

### Ranges

Ranges are created with `..` syntax and are exclusive of the end value:

```ject
1..6        # integers 1, 2, 3, 4, 5
0..10:2     # 0, 2, 4, 6, 8 (step of 2)
```

Ranges can be assigned to variables, passed to functions, or used in `for` loops. `sum()`, `len()`, and other array functions work on ranges.

### break and continue

```ject
while true do
    let input = input("> ")
    if input == "quit" then
        break
    end
    if input == "" then
        continue
    end
    print "You said: " + input
end
```

---

## Functions

### Definition

```ject
fn greet(name)
    print "Hello, " + name + "!"
end
```

### Return values

```ject
fn add(a, b)
    return a + b
end

let result = add(3, 4)  # 7
```

A function with no `return` statement returns `nil`.

### Default parameters

```ject
fn greet(name, greeting = "Hello")
    print greeting + ", " + name + "!"
end

greet("Alice")              # Hello, Alice!
greet("Bob", "Hi")          # Hi, Bob!
```

### Recursion

```ject
fn factorial(n)
    if n <= 1 then
        return 1
    else
        return n * factorial(n - 1)
    end
end
```

### Lambda expressions

Lambdas are anonymous functions. Both `fn` and `lambda` are valid keywords for them.

```ject
# Arrow syntax (expression body)
let square = fn(x) -> x * x
let add = lambda(a, b) -> a + b

# Block body
let describe = fn(x)
    if x > 0 then
        return "positive"
    else
        return "non-positive"
    end
end
```

Lambdas are first-class values and can be passed to functions or stored in variables:

```ject
let numbers = [1, 2, 3, 4, 5]
let doubled = map(numbers, fn(n) -> n * 2)
let evens = filter(numbers, fn(n) -> n % 2 == 0)
let total = reduce(numbers, fn(acc, n) -> acc + n, 0)
```

### Functions as values

Functions defined with `fn name(...)` can be stored and passed around:

```ject
fn double(x)
    return x * 2
end

let transform = double
print transform(5)  # 10
```

---

## Arrays

### Creation

```ject
let empty = []
let numbers = [1, 2, 3, 4, 5]
let mixed = [1, "hello", true, nil]
let nested = [[1, 2], [3, 4], [5, 6]]
```

### Access

```ject
numbers[0]    # first element: 1
numbers[-1]   # last element: 5
numbers[-2]   # second to last: 4
```

### Slicing

Ject has three slicing syntaxes — all are equivalent:

**Named parameter style:**
```ject
let arr = [0, 1, 2, 3, 4, 5]
arr[from:1 to:4]         # [1, 2, 3]
arr[from:0 to:6 step:2]  # [0, 2, 4]
arr[from:5 to:0 step:-1] # [5, 4, 3, 2, 1]
```

**Range style:**
```ject
arr[1..4]    # [1, 2, 3]
arr[0..6:2]  # [0, 2, 4]
```

**Python-style:**
```ject
arr[1:4]     # [1, 2, 3]
arr[::2]     # [0, 2, 4]
arr[1:]      # [1, 2, 3, 4, 5]
arr[:3]      # [0, 1, 2]
```

Slicing works the same way on strings.

### Modification

Arrays are not mutated in place with most operations — you get a new array back. `push` and `pop` return new arrays:

```ject
let arr = [1, 2, 3]
arr = push(arr, 4)     # [1, 2, 3, 4]
arr = pop(arr)         # [1, 2, 3] (removes last element)
```

Direct index assignment does mutate in place:

```ject
let arr = [1, 2, 3]
arr[0] = 99            # arr is now [99, 2, 3]
```

### Unique arrays

Unique arrays (sets) are declared with `{| |}` syntax and automatically deduplicate their contents:

```ject
let tags = {|"rust", "python", "rust", "ject"|}
print tags  # {|"rust", "python", "ject"|}

tags = push(tags, "python")  # no-op, already exists
tags = push(tags, "go")      # {|"rust", "python", "ject", "go"|}
```

### Useful array functions

All of these are available globally without any import:

```ject
len([1, 2, 3])                     # 3
sum([1, 2, 3, 4])                  # 10
first([10, 20, 30])                # 10
last([10, 20, 30])                 # 30
reverse([1, 2, 3])                 # [3, 2, 1]
sort([3, 1, 2])                    # [1, 2, 3]
unique([1, 2, 2, 3, 3])            # [1, 2, 3]
contains([1, 2, 3], 2)             # true
index_of([1, 2, 3], 2)             # 1
slice([0, 1, 2, 3, 4], 1, 4)       # [1, 2, 3]
map([1, 2, 3], fn(x) -> x * 2)    # [2, 4, 6]
filter([1,2,3,4], fn(x) -> x > 2) # [3, 4]
reduce([1,2,3], fn(a,b) -> a+b, 0) # 6
```

---

## Strings

### Creation and escapes

```ject
let greeting = "Hello, World!"
let path = "C:\\Users\\leo"
let multiline = "line one\nline two\nline three"
let tabbed = "col1\tcol2"
```

### String interpolation

Use `$variable` for simple variable interpolation, or `${expression}` for arbitrary expressions:

```ject
let name = "Alice"
let age = 30
print "Hello, $name!"              # Hello, Alice!
print "In 5 years: ${age + 5}"     # In 5 years: 35
print "Type: ${type_of(name)}"     # Type: string
```

### String operations (built-in)

```ject
len("hello")                  # 5
upper("hello")                # "HELLO"
lower("HELLO")                # "hello"
trim("  hello  ")             # "hello"
split("a,b,c", ",")           # ["a", "b", "c"]
join(["a", "b", "c"], ",")    # "a,b,c"
replace("hello", "l", "r")   # "herro" (replaces all)
char_at("hello", 1)           # "e"
substring("hello", 1, 4)      # "ell"
repeat("ab", 3)               # "ababab"
```

Slicing works on strings with all three slicing syntaxes:

```ject
let s = "hello"
s[1:4]       # "ell"
s[from:1 to:4]  # "ell"
s[-3:]       # "llo"
```

---

## Dictionaries

Dictionaries are key-value stores with string keys.

### Creation

```ject
let person = {name: "Alice", age: 30, active: true}
let empty = {}
```

### Access

```ject
person["name"]   # "Alice"
person["age"]    # 30
```

### Assignment

```ject
person["email"] = "alice@example.com"
person["age"] = 31
```

### Checking keys

```ject
if person["email"] != nil then
    print "has email"
end
```

---

## Structs

Structs define named data types with specific fields.

### Definition

```ject
struct Point {
    x, y
}

struct Person {
    name,
    age,
    email
}
```

Trailing commas after field names are optional.

### Instantiation

```ject
let p = new Point { x: 10, y: 20 }
let alice = new Person {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
}
```

### Field access and mutation

```ject
print p.x          # 10
p.x = 99
print p.x          # 99
```

### Structs in arrays

```ject
struct Todo { id, task, completed }

let todos = []
let item = new Todo { id: 1, task: "write docs", completed: false }
todos = push(todos, item)

for todo in todos do
    print todo.task
end
```

---

## Modules

### Importing a module

```ject
import "math"           # imports math module into scope as "math"
import "math" as m      # imports with alias "m"
import "string" as str  # string module with alias
```

After importing with an alias, functions are accessed via the alias:

```ject
import "math" as m
print m.sqrt(16)  # 4.0
print m.PI        # 3.141592653589793
```

### Selective imports

```ject
import {sqrt, PI, sin} from "math"
print sqrt(25)   # 5.0
print PI         # 3.14...
```

### Importing your own modules

```ject
import "modules/math_utils" as utils
print utils.circle_area(5)
```

Paths are relative to the current file.

### Exporting from a module

In your module file, mark values and functions with `export`:

```ject
# my_module.ject
export PI = 3.14159

export fn circle_area(r)
    return PI * r * r
end

export fn square(x)
    return x * x
end
```

Constants are exported directly with `export name = value`. Functions use `export fn`.

### Available standard library modules

| Module | Description |
|--------|-------------|
| `"math"` | Advanced math: trig, logarithms, primes, combinatorics, etc. |
| `"string"` | Advanced string manipulation |
| `"array"` | Advanced array utilities |
| `"io"` | File I/O helpers (read/write JSON, write lines) |
| `"json"` | JSON validation and path access |
| `"datetime"` | Timestamps and time utilities |
| `"system"` | CWD, directory change |
| `"util"` | Type utilities, higher-order functions |
| `"numpy"` | Numerical arrays with Rust-backed performance |
| `"gui"` | Native GUI window builder |

---

## Error Handling

### throw

```ject
fn divide(a, b)
    if b == 0 then
        throw "Division by zero"
    end
    return a / b
end
```

You can throw any value — strings are most common.

### try / catch

```ject
try
    let result = divide(10, 0)
    print "Result: " + result
catch err
    print "Error: " + err
end

print "Program continues here"
```

The `catch` clause binds the thrown value to the named variable (`err` here). Execution after the `try/catch` block continues normally.

---

## Input & Output

### print

`print` accepts any number of values separated by spaces. No parentheses needed (though they work too):

```ject
print "Hello"
print "x =", x
print 1, 2, 3        # 1 2 3
print("also fine")   # parenthesized form works
```

**Optional `sep` and `end` options:**

```ject
print "a", "b", "c" sep:","      # a,b,c
print "hello" end:""             # no newline at the end
print "x =", x sep:":" end:"\n"
```

### input

Prompts the user and returns their input as a string:

```ject
let name = input("What's your name? ")
let age_str = input("Age: ")
let age = to_int(age_str)
```

---

## File System

These functions are available globally:

```ject
# Read entire file as string
let content = read_file("notes.txt")

# Write string to file (creates or overwrites)
write_file("output.txt", "Hello, file!")

# Check if file exists
if file_exists("config.txt") then
    print "Config found"
end

# Check if path is a file or directory
is_file("notes.txt")    # true/false
is_dir("/home/leo")     # true/false
```

From the `io` module:

```ject
import "io"

# Write an array of lines
io.write_lines("log.txt", ["line 1", "line 2", "line 3"])

# Read/write JSON files
io.write_json("data.json", {name: "Alice", age: 30})
let data = io.read_json("data.json")
```

---

## System

These require either importing `system` or using the global builtins directly (they're available without import):

```ject
# Run a shell command, returns stdout as string
let output = exec("ls -la")
print output

# Get environment variable
let home = env("HOME")
let path = env("PATH")

# Exit the program
exit(0)   # success
exit(1)   # failure
```

From the `system` module:

```ject
import "system" as sys
let cwd = sys.get_cwd()
```

---

## Built-in Functions Reference

These are available in every Ject program without any import.

### Type & Conversion

| Function | Description |
|----------|-------------|
| `type_of(value)` | Returns the type as a string |
| `to_int(value)` | Converts to integer |
| `to_float(value)` | Converts to float |
| `to_string(value)` | Converts to string |
| `to_bool(value)` | Converts to boolean |

### Array

| Function | Description |
|----------|-------------|
| `len(arr)` | Length of array or string |
| `push(arr, item)` | Returns new array with item appended |
| `pop(arr)` | Returns new array with last item removed |
| `first(arr)` | First element |
| `last(arr)` | Last element |
| `sum(arr)` | Sum of numeric elements |
| `sort(arr)` | Sorted copy |
| `reverse(arr)` | Reversed copy |
| `unique(arr)` | Deduplicated copy |
| `contains(arr, item)` | Whether item is in array |
| `index_of(arr, item)` | Index of item, or -1 |
| `slice(arr, start, end)` | Slice by start/end index |
| `map(arr, fn)` | Apply function to each element |
| `filter(arr, fn)` | Keep elements where fn returns true |
| `reduce(arr, fn, init)` | Fold left with initial value |
| `range(n)` | Array `[0..n-1]` |
| `range(start, end)` | Array from start to end (exclusive) |
| `range(start, end, step)` | With step |

### String

| Function | Description |
|----------|-------------|
| `len(str)` | Number of characters |
| `upper(str)` | Uppercase |
| `lower(str)` | Lowercase |
| `trim(str)` | Strip leading/trailing whitespace |
| `split(str, sep)` | Split into array |
| `join(arr, sep)` | Join array into string |
| `replace(str, old, new)` | Replace all occurrences |
| `char_at(str, i)` | Character at index (supports negative) |
| `substring(str, start, end)` | Substring by index range |
| `repeat(str, n)` | Repeat string n times |

### Math

| Function | Description |
|----------|-------------|
| `abs(x)` | Absolute value |
| `sqrt(x)` | Square root |
| `pow(x, y)` | x to the power of y |
| `sin(x)` | Sine (radians) |
| `cos(x)` | Cosine (radians) |
| `tan(x)` | Tangent (radians) |
| `floor(x)` | Round down |
| `ceil(x)` | Round up |
| `round(x)` | Round to nearest |
| `min(a, b)` | Minimum of two values |
| `max(a, b)` | Maximum of two values |
| `PI` | π ≈ 3.141592653589793 |
| `E` | e ≈ 2.718281828459045 |

### Random

| Function | Description |
|----------|-------------|
| `random()` | Float in `[0.0, 1.0)` |
| `random_int(min, max)` | Integer in `[min, max)` |

### I/O

| Function | Description |
|----------|-------------|
| `print ...` | Print to stdout with newline |
| `input(prompt)` | Read a line from stdin |
| `read_file(path)` | Read file as string |
| `write_file(path, content)` | Write string to file |
| `file_exists(path)` | Check if file exists |
| `is_file(path)` | Check if path is a regular file |
| `is_dir(path)` | Check if path is a directory |

### JSON

| Function | Description |
|----------|-------------|
| `parse_json(str)` | Parse JSON string into Ject value |
| `to_json(value)` | Serialize Ject value to JSON string |

### System

| Function | Description |
|----------|-------------|
| `exec(cmd)` | Run shell command, return stdout |
| `env(name)` | Get environment variable value |
| `exit(code)` | Exit program with status code |
| `now()` | Current time as float (Unix seconds) |
| `timestamp()` | Current Unix timestamp as integer |
| `sleep(ms)` | Sleep for milliseconds |

---

## Standard Library Modules

### math

```ject
import "math" as m
# or: import {sqrt, log, ...} from "math"
```

**Extra constants:**

```ject
m.PHI    # 1.618... (golden ratio)
m.SQRT2  # 1.414...
m.SQRT3  # 1.732...
```

**Logarithms** (note: these require importing `math` — they're not in the global scope):

```ject
m.log(x, base)   # log base `base` of x
m.log10(x)       # log base 10
m.log2(x)        # log base 2
m.ln(x)          # natural log (base e)
m.exp(x)         # e^x
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

**Number theory:**

```ject
m.gcd(a, b)
m.lcm(a, b)
m.factorial(n)
m.is_prime(n)
m.primes_up_to(n)
m.fibonacci(n)
```

**Statistics:**

```ject
m.average(arr)
m.median(arr)
m.variance(arr)
m.stddev(arr)
m.product(arr)
```

**Combinatorics:**

```ject
m.permutations(n, k)   # P(n,k)
m.combinations(n, k)   # C(n,k)
```

**Geometry:**

```ject
m.distance_2d(x1, y1, x2, y2)
m.distance_3d(x1, y1, z1, x2, y2, z2)
m.dot_2d(x1, y1, x2, y2)
m.cross_2d(x1, y1, x2, y2)
m.magnitude(x, y)
m.normalize_2d(x, y)      # returns [nx, ny]
m.quadratic_roots(a, b, c) # returns [] or [r] or [r1, r2]
```

**Utilities:**

```ject
m.round_to(x, decimals)
m.clamp(value, min, max)
m.sign(x)                  # -1, 0, or 1
m.lerp(a, b, t)
m.remap(value, from_min, from_max, to_min, to_max)
m.deg_to_rad(degrees)
m.rad_to_deg(radians)
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
s.capitalize(str)                 # "hello" -> "Hello"
s.title_case(str)                 # "hello world" -> "Hello World"
s.trim_left(str)                  # trim leading whitespace
s.trim_right(str)                 # trim trailing whitespace
s.pad_left(str, width, char)      # right-align in field
s.pad_right(str, width, char)     # left-align in field
s.pad_center(str, width, char)    # center in field
s.starts_with(str, prefix)        # true/false
s.ends_with(str, suffix)          # true/false
s.contains_str(str, substr)       # true/false
s.find(str, substr, start)        # index or -1
s.count(str, substr)              # count of occurrences
s.replace_all(str, old, new)      # same as built-in replace
s.replace_first(str, old, new)    # replace only first occurrence
s.remove(str, substr)             # remove all occurrences
s.reverse_str(str)
s.left(str, n)                    # first n chars
s.right(str, n)                   # last n chars
s.truncate(str, max_len)          # truncate with "..."
s.is_empty(str)
s.is_numeric(str)                 # all digit characters?
s.is_alpha(str)                   # all letter characters?
s.word_count(str)
s.sentence_count(str)
s.lines(str)                      # split on newlines -> array
s.format(template, args)          # "{0} and {1}" with array
s.wrap_text(str, width)           # word-wrap to column width
s.extract_numbers(str)            # pull all numbers out as array
```

---

### array

```ject
import "array" as arr
```

```ject
arr.average(a)
arr.median(a)
arr.take(a, n)             # first n elements
arr.drop(a, n)             # all but first n elements
arr.initial(a)             # all but last element
arr.rest(a)                # all but first element
arr.concat(a, b)           # concatenate two arrays
arr.zip(a, b)              # [[a0,b0], [a1,b1], ...]
arr.union(a, b)            # unique elements from both
arr.intersection(a, b)     # elements in both
arr.difference(a, b)       # elements in a not in b
arr.flatten(a)             # recursively flatten nested arrays
arr.chunk(a, n)            # split into chunks of size n
arr.shuffle(a)             # random order
arr.rotate_left(a, n)
arr.rotate_right(a, n)
arr.insert_at(a, idx, item)
arr.remove_at(a, idx)
arr.compact(a)             # remove nil and false values
arr.enumerate(a)           # [[0, item0], [1, item1], ...]
arr.fill_arr(count, value) # array of count copies of value
arr.range_arr(start, end, step)  # like range() but more flexible
arr.sample(a)              # random element
arr.sort_by(a, key_fn)    # sort by computed key
```

---

### io

```ject
import "io"
```

```ject
io.write_lines(path, lines_arr)  # write array of strings as lines
io.read_json(path)               # read and parse JSON file
io.write_json(path, value)       # serialize and write JSON file
```

---

### json

```ject
import "json" as j
```

```ject
j.to_json_pretty(value)          # pretty-print JSON (currently same as to_json)
j.is_valid_json(str)             # true if string is valid JSON
j.json_get(obj, "key.nested")    # dot-path access into parsed JSON
```

---

### datetime

```ject
import "datetime" as dt
```

`now()` and `timestamp()` are global builtins. The `datetime` module is a thin wrapper with placeholder implementations — for most uses, call the builtins directly:

```ject
let t = now()         # current time as float (Unix seconds)
let ts = timestamp()  # current Unix timestamp as integer
sleep(500)            # sleep 500ms
```

---

### util

```ject
import "util" as u
```

```ject
u.identity(x)           # returns x unchanged
u.constant(value)       # returns fn(_) -> value
u.compose(f, g)         # returns fn(x) -> f(g(x))
u.apply(func, value)    # func(value)
u.is_nil(value)         # value == nil
u.is_truthy(value)      # explicit truthiness check
u.deep_equal(a, b)      # recursive equality for arrays
u.copy(value)           # deep copy arrays/dicts
```

---

## GUI Module

The `gui` module lets you build simple native GUI dialogs.

```ject
import {window, label, separator, button, input, run} from "gui"

let app = window("My App", 560, 380)

label(app, "Enter your details:")
separator(app)

input(app, "name", "Name:", "")
input(app, "email", "Email:", "default@example.com")

separator(app)
button(app, "ok", "OK")
button(app, "cancel", "Cancel")

let result = run(app)

print "Button clicked:", result["buttons"]
print "Input values:", result["inputs"]
```

`run(app)` blocks until the window is closed and returns a dictionary:

```ject
{
    "buttons": ["ok"],       # array of clicked button IDs
    "inputs": {              # dictionary of input field values
        "name": "Alice",
        "email": "alice@example.com"
    }
}
```

---

## NumPy Module

Ject has a `numpy`-like module backed by Rust for high-performance numerical computing.

```ject
import "numpy" as np
```

### Creating arrays

```ject
np.array([1, 2, 3, 4, 5])   # from Ject array
np.zeros(5)                   # [0, 0, 0, 0, 0]
np.ones(5)                    # [1, 1, 1, 1, 1]
np.arange(0, 10, 2)          # [0, 2, 4, 6, 8]
np.linspace(0, 1, 5)         # 5 evenly spaced from 0 to 1
np.eye(3)                     # 3x3 identity matrix
np.identity(3)
```

### Properties

```ject
np.shape(arr)   # [rows, cols] or [length]
np.ndim(arr)    # number of dimensions
np.size(arr)    # total element count
```

### Manipulation

```ject
np.reshape(arr, [2, 3])
np.flatten(arr)
np.transpose(arr)
np.concatenate([arr1, arr2])
np.stack([arr1, arr2])
```

### Math operations (element-wise)

```ject
np.sqrt(arr)
np.exp(arr)
np.log(arr)
np.abs(arr)
np.sin(arr)
np.cos(arr)
np.add(arr1, arr2)
np.subtract(arr1, arr2)
np.multiply(arr1, arr2)
np.divide(arr1, arr2)
```

### Aggregation

```ject
np.sum(arr)
np.min(arr)
np.max(arr)
np.mean(arr)
np.std(arr)
np.var(arr)
```

### Linear algebra

```ject
np.dot(a, b)          # dot product / matrix multiply
np.norm(arr)          # vector norm
np.det(matrix)        # determinant
np.inv(matrix)        # matrix inverse
np.solve(A, b)        # solve Ax = b
```

---

## Quick Reference

### Keywords

```
let  fn  return  if  elseif  else  then  end
while  for  in  do  break  continue
struct  new  try  catch  throw
import  export  from  as
true  false  nil
lambda
```

### Operators summary

```
+  -  *  /  %          arithmetic
==  !=  <  >  <=  >=   comparison
and  or  !              logical
=                       assignment
```

### Comment syntax

```ject
# single line only
```

### Common patterns

**Swap two variables:**
```ject
let temp = a
a = b
b = temp
```

**Check if array is empty:**
```ject
if len(arr) == 0 then
    print "empty"
end
```

**Loop with index:**
```ject
let i = 0
for item in arr do
    print i + ": " + item
    i = i + 1
end
```

**Build a string from parts:**
```ject
let parts = ["Hello", "World"]
let result = join(parts, ", ")
```

**Parse and validate input:**
```ject
let raw = input("Enter a number: ")
try
    let n = to_int(raw)
    print "Got: " + n
catch err
    print "Not a valid number"
end
```
