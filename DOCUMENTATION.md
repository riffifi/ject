# Ject Programming Language Documentation

**Version:** 0.1.0  
**A modern, expressive scripting language built in Rust**

---

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Basic Syntax](#basic-syntax)
4. [Variables and Data Types](#variables-and-data-types)
5. [Operators](#operators)
6. [Control Flow](#control-flow)
7. [Functions](#functions)
8. [Lambda Expressions](#lambda-expressions)
9. [Arrays and Slicing](#arrays-and-slicing)
10. [Dictionaries](#dictionaries)
11. [Strings](#strings)
12. [Structs](#structs)
13. [Module System](#module-system)
14. [Error Handling](#error-handling)
15. [Standard Library](#standard-library)
16. [REPL](#repl)
17. [Best Practices](#best-practices)

---

## Introduction

Ject is a dynamically-typed scripting language designed for **simplicity**, **readability**, and **performance**. Built in Rust, it combines the ease of Python with elegant syntax inspired by Ruby and Crystal.

### Design Philosophy

- **Simplicity First** - Clean, uncomplicated syntax
- **Readable Code** - Easy to read and understand
- **Fast Development** - Quick iteration, minimal boilerplate
- **Performance** - Built in Rust for speed and safety
- **Enjoyable** - Pleasant to write and maintain

### Key Features

- - Clean `elseif` keyword (no `elif` confusion)
- - Explicit `end` blocks (no indentation fragility)
- - Rich built-in standard library (no `import math` needed)
- - Beautiful lambda expressions
- - **Three slicing syntaxes** (better than Python!)
- - Module system with selective imports
- - Struct support
- - Try/catch error handling

---

## Getting Started

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd ject

# Build in release mode
cargo build --release

# Run the REPL
./target/release/ject
```

### Your First Program

Create a file `hello.ject`:

```ject
# Hello, World!
print "Hello, World!"

# Variables
let name = "Ject"
let version = 1.0

# Functions
fn greet(person)
    print "Hello, " + person + "!"
end

greet("World")
```

Run it:

```bash
./target/release/ject hello.ject
```

---

## Basic Syntax

### Comments

```ject
# Single-line comment

#*
    Multi-line comment
    Spans multiple lines
*#
```

### Statements

- No semicolons required
- Newlines separate statements
- Use `end` to close blocks

```ject
let x = 10
let y = 20
print x + y  # 30
```

---

## Variables and Data Types

### Declaration

```ject
# Declare with 'let'
let name = "Alice"
let age = 30
let height = 5.8
let is_student = false
let nothing = nil

# Reassign without 'let'
name = "Bob"
age = 25
```

### Data Types

| Type | Example | Description |
|------|---------|-------------|
| `int` | `42`, `-17`, `0` | Integer numbers |
| `float` | `3.14`, `-0.5`, `1.0` | Floating-point numbers |
| `string` | `"hello"`, `"world"` | Text |
| `bool` | `true`, `false` | Boolean values |
| `nil` | `nil` | Null/none value |
| `array` | `[1, 2, 3]` | Ordered collection |
| `unique_array` | `{|1, 2, 3|}` | Array with unique values |
| `dictionary` | `{key: "value"}` | Key-value pairs |

**Note:** Ject distinguishes between `int` and `float` types, unlike some languages that lump them together as "number".

### Type Inspection

```ject
let x = 42
print type_of(x)  # "integer"

let y = "hello"
print type_of(y)  # "string"
```

---

## Operators

### Arithmetic

```ject
let sum = 10 + 5       # 15
let diff = 10 - 5      # 5
let product = 10 * 5   # 50
let quotient = 10 / 5  # 2.0
let remainder = 10 % 3 # 1
let power = 2 ^ 3      # 8 (if supported)
```

### Compound Assignment

```ject
let x = 10
x += 5  # x = 15
x -= 3  # x = 12
x *= 2  # x = 24
x /= 4  # x = 6.0
x %= 5  # x = 1.0
```

### Increment/Decrement

```ject
let x = 5
x++  # 5 (post-increment)
++x  # 7 (pre-increment)

x--  # 7 (post-decrement)
--x  # 5 (pre-decrement)
```

### Comparison

```ject
10 == 10  # true
10 != 5   # true
10 > 5    # true
10 < 5    # false
10 >= 10  # true
10 <= 10  # true
```

### Logical

```ject
true and false   # false
true or false    # true
!true            # false
not true         # false (if supported)
```

### String Concatenation

```ject
let greeting = "Hello" + " " + "World"
print greeting  # "Hello World"
```

---

## Control Flow

### If/ElseIf/Else

```ject
let score = 85

if score >= 95
    print "A+ Excellent!"
elseif score >= 90
    print "A grade!"
elseif score >= 80
    print "B grade!"
elseif score >= 70
    print "C grade!"
else
    print "Keep trying!"
end
```

**Note:** Ject uses `elseif` (not Python's `elif`)

### Alternative: `else if`

```ject
if temperature < 0
    print "Freezing!"
else if temperature < 30
    print "Nice weather"
else
    print "Getting warm!"
end
```

### While Loops

```ject
let counter = 0

while counter < 5 do
    print "Count: " + counter
    counter = counter + 1
end
```

### For Loops

#### With Arrays

```ject
let fruits = ["apple", "banana", "cherry"]

for fruit in fruits do
    print "I love " + fruit
end
```

#### With Ranges

```ject
# Basic range (exclusive end)
for i in 1..6 do
    print "Number: " + i  # 1, 2, 3, 4, 5
end

# Range with step
for i in 2..10:2 do
    print "Even: " + i  # 2, 4, 6, 8
end

# Reverse range
for i in 5..0:-1 do
    print "Countdown: " + i  # 5, 4, 3, 2, 1
end
```

#### With Strings

```ject
for ch in "hello" do
    print ch  # h, e, l, l, o (each on new line)
end
```

### Break and Continue

```ject
for i in 1..20 do
    if i == 5 then continue end  # Skip 5
    if i == 15 then break end    # Stop at 15
    print i
end
```

---

## Functions

### Definition

```ject
fn greet(name)
    print "Hello, " + name + "!"
end

greet("World")  # "Hello, World!"
```

### Parameters with Defaults

```ject
fn greet(name, greeting = "Hello")
    print greeting + ", " + name + "!"
end

greet("Alice")              # "Hello, Alice!"
greet("Bob", "Good morning") # "Good morning, Bob!"
```

### Return Values

```ject
fn add(a, b)
    return a + b
end

let result = add(3, 5)
print result  # 8
```

### Recursive Functions

```ject
fn fibonacci(n)
    if n <= 1 then
        return n
    else
        return fibonacci(n - 1) + fibonacci(n - 2)
    end
end

print fibonacci(10)  # 55
```

### Higher-Order Functions

```ject
fn apply(numbers, fn)
    let result = []
    for n in numbers do
        push(result, lambda(n))
    end
    return result
end

let double = lambda(x) -> x * 2
print apply([1, 2, 3], double)  # [2, 4, 6]
```

---

## Lambda Expressions

### Basic Syntax

```ject
# Single expression
let square = lambda(x) -> x * x
print square(5)  # 25

# Multiple parameters
let add = lambda(a, b) -> a + b
print add(10, 5)  # 15

# No parameters
let get_pi = fn() -> 3.14159
print get_pi()  # 3.14159
```

### Multi-Statement Lambdas

```ject
let complex = lambda(x) -> {
    let y = x * 2
    let z = y + 1
    return z
}
print complex(5)  # 11
```

### Lambdas with Arrays

```ject
let numbers = [1, 2, 3, 4, 5]

# Map
let doubled = map(numbers, lambda(n) -> n * 2)
print doubled  # [2, 4, 6, 8, 10]

# Filter
let evens = filter(numbers, lambda(n) -> n % 2 == 0)
print evens  # [2, 4]

# Reduce
let sum = reduce(numbers, lambda(acc, n) -> acc + n, 0)
print sum  # 15
```

### Closures

```ject
fn make_counter()
    let count = 0
    return fn() -> {
        count = count + 1
        return count
    }
end

let counter = make_counter()
print counter()  # 1
print counter()  # 2
print counter()  # 3
```

---

## Arrays and Slicing

### Creating Arrays

```ject
let empty = []
let numbers = [1, 2, 3, 4, 5]
let mixed = [1, "two", true, nil]
let nested = [[1, 2], [3, 4]]
```

### Unique Arrays (Sets)

Use `{| |}` syntax to create arrays that automatically deduplicate values:

```ject
let tags = {|"rust", "python", "ject", "rust", "python"|}
print tags  # {|"rust", "python", "ject"|}

let nums = {|1, 2, 3, 2, 1, 4|}
print nums  # {|1, 2, 3, 4|}

let empty = {|}
print empty  # {||}

# Mixed types
let mixed = {|1, "one", 2, "one"|}
print mixed  # {|1, "one", 2|}
```

### Converting Arrays to Unique Arrays

Use `to_uarray()` to convert a regular array to a unique array:

```ject
let arr = [1, 2, 2, 3, 3, 3, 4]
let uarr = to_uarray(arr)
print uarr  # {|1, 2, 3, 4|}

# Works on unique arrays too (returns copy)
let uarr2 = to_uarray(uarr)
```

### Converting Unique Arrays to Arrays

Use `to_array()` to convert a unique array back to a regular array:

```ject
let uarr = {|1, 2, 3|}
let arr = to_array(uarr)
print arr       # [1, 2, 3]
print type_of(arr)  # "array"
```

### Map, Filter, Reduce with Unique Arrays

These functions preserve the unique array type:

```ject
let uarr = {|1, 2, 3, 4, 5|}

# map preserves uniqueness
let doubled = map(uarr, lambda(x) -> x * 2)
print doubled  # {|2, 4, 6, 8, 10|}

# filter preserves uniqueness
let filtered = filter(uarr, lambda(x) -> x > 2)
print filtered  # {|3, 4, 5|}

# Results are automatically deduplicated
let mapped = map({|1, 2, 3|}, lambda(x) -> x * 2)
print mapped  # {|2, 4, 6|}
```

Unique arrays are perfect for:
- Tag collections
- Unique ID lists
- Membership testing
- Removing duplicates from data

### Accessing Elements

```ject
let arr = [10, 20, 30, 40, 50]

print arr[0]    # 10 (first element)
print arr[2]    # 30 (third element)
print arr[-1]   # 50 (last element)
print arr[-2]   # 40 (second to last)
```

### Modifying Arrays

```ject
let arr = [1, 2, 3]

arr[0] = 10      # [10, 2, 3]
arr[-1] = 30     # [10, 2, 30]

push(arr, 4)     # [10, 2, 30, 4]
pop(arr)         # returns 4, arr = [10, 2, 30]
```

### Array Operations

```ject
let arr = [1, 2, 3]

# Length
print len(arr)  # 3

# Sum
print sum(arr)  # 6

# Min/Max
print min(arr)  # 1
print max(arr)  # 3

# Concatenation
let combined = arr + [4, 5]  # [1, 2, 3, 4, 5]
```

### List Comprehensions

Ject supports Python-style list comprehensions with cleaner syntax:

```ject
# Basic comprehension
let squares = [x * x for x in 1..6]
print squares  # [1, 4, 9, 16, 25]

# With filter condition
let evens = [x for x in 1..11 if x % 2 == 0]
print evens  # [2, 4, 6, 8, 10]

# With transformation AND filter
let even_squares = [x * x for x in 1..11 if x % 2 == 0]
print even_squares  # [4, 16, 36, 64, 100]

# Works with arrays too
let nums = [1, 2, 3, 4, 5]
let doubled = [x * 2 for x in nums]
print doubled  # [2, 4, 6, 8, 10]

# Complex expressions
let pairs = [[x, x * 2] for x in 1..5]
print pairs  # [[1, 2], [2, 4], [3, 6], [4, 8], [5, 10]]
```

**Syntax:** `[expression for variable in iterable if condition]`

The `if condition` part is optional. List comprehensions are more concise than equivalent `for` loops and often more readable.

---

##  Slicing (Better Than Python!)

Ject supports **three** slicing syntaxes, all more expressive than Python's `start:stop:step`.

### 1. Named Parameters (Recommended)

**Most readable and self-documenting:**

```ject
let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# Basic slice
print arr[from:2 to:5]      # [2, 3, 4]

# With step
print arr[from:0 to:10 step:2]  # [0, 2, 4, 6, 8]

# Just from
print arr[from:5]           # [5, 6, 7, 8, 9]

# Just to
print arr[to:3]             # [0, 1, 2]

# Just step
print arr[step:2]           # [0, 2, 4, 6, 8]

# Negative indices
print arr[from:-5 to:-1]    # [5, 6, 7, 8]

# Reverse
print arr[from:5 to:0 step:-1]  # [5, 4, 3, 2, 1]

# Full reverse
print arr[from:9 to:-1 step:-1]  # [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
```

### 2. Range Syntax (Concise)

**Perfect for loops and quick slices:**

```ject
let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# Basic range
print arr[2..5]         # [2, 3, 4]

# With step
print arr[0..8:2]       # [0, 2, 4, 6]

# Reverse
print arr[5..0:-1]      # [5, 4, 3, 2, 1]
```

### 3. Python-Style Colon Syntax

**For Python refugees:**

```ject
let arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# Basic slice
print arr[:3]           # [0, 1, 2]
print arr[5:]           # [5, 6, 7, 8, 9]
print arr[1:6]          # [1, 2, 3, 4, 5]

# With step
print arr[1:6:2]        # [1, 3, 5]
print arr[::2]          # [0, 2, 4, 6, 8]

# Reverse
print arr[::-1]         # [9, 8, 7, 6, 5, 4, 3, 2, 1, 0]
print arr[5:0:-1]       # [5, 4, 3, 2, 1]
```

### String Slicing

**All slicing syntaxes work on strings too:**

```ject
let text = "Hello, World!"

# Named parameters
print text[from:0 to:5]     # "Hello"
print text[from:7 to:12]    # "World"
print text[step:2]          # "Hlo ol!"

# Range syntax
print text[0..5]            # "Hello"

# Python-style
print text[:5]              # "Hello"
print text[::-1]            # "!dlroW ,olleH"
```

### Slicing Comparison

| Syntax | Example | Best For |
|--------|---------|----------|
| Named | `arr[from:1 to:5 step:2]` | Readability, complex slices |
| Range | `arr[1..5:2]` | Conciseness, loops |
| Python | `arr[1:5:2]` | Familiarity |

---

## Dictionaries

### Creating Dictionaries

```ject
let empty = {}

let person = {
    name: "Alice",
    age: 30,
    email: "alice@example.com"
}

# Single line
let point = {x: 10, y: 20}
```

### Accessing Values

```ject
let person = {name: "Alice", age: 30}

# Dot notation
print person.name  # "Alice"
print person.age   # 30

# Index notation
print person["name"]  # "Alice"
```

### Modifying Dictionaries

```ject
let person = {name: "Alice", age: 30}

person.age = 31           # Update
person.email = "new@example.com"  # Add new key
```

### Dictionary Operations

```ject
let person = {name: "Alice", age: 30}

# Check if key exists
if "name" in person then
    print "Name is present"
end

# Length
print len(person)  # 2
```

---

## Strings

### Creating Strings

```ject
let single = 'hello'
let double = "world"

# Escape sequences
let with_newline = "Hello\nWorld"
let with_tab = "Col1\tCol2"
let with_quote = "She said \"hello\""
```

### String Interpolation

```ject
let name = "Alice"
let age = 30

# Using ${} syntax
print "Hello, ${name}! You are ${age} years old."

# Using $variable syntax
print "Name: $name, Age: $age"

# Expressions in interpolation
print "Next year: ${age + 1}"
```

### String Operations

```ject
let text = "  Hello World  "

# Case conversion
print upper(text)  # "  HELLO WORLD  "
print lower(text)  # "  hello world  "

# Trimming
print trim(text)   # "Hello World"

# Split and join
let csv = "apple,banana,cherry"
let fruits = split(csv, ",")  # ["apple", "banana", "cherry"]
let joined = join(fruits, " | ")  # "apple | banana | cherry"

# Replace
let fixed = replace("Hello Python", "Python", "Ject")
print fixed  # "Hello Ject"

# Length
print len("hello")  # 5
```

### String Indexing and Slicing

```ject
let text = "Hello, World!"

# Indexing
print text[0]    # "H"
print text[-1]   # "!"

# Slicing (see Slicing section above)
print text[0..5]     # "Hello"
print text[:5]       # "Hello"
print text[from:7 to:12]  # "World"
```

---

## Structs

### Defining Structs

```ject
struct Point {
    x,
    y
}

struct Person {
    name,
    age,
    email
}
```

### Creating Instances

```ject
# Using new keyword
let p1 = new Point {x: 10, y: 20}
let p2 = new Point {x: 30, y: 40}

# Fields can be in any order
let person = new Person {
    email: "alice@example.com",
    name: "Alice",
    age: 30
}
```

### Accessing Fields

```ject
let p = new Point {x: 10, y: 20}

print p.x  # 10
print p.y  # 20

# Modify fields
p.x = 100
```

### Struct Methods (via Functions)

```ject
struct Point {x, y}

fn distance_from_origin(point)
    return sqrt(point.x * point.x + point.y * point.y)
end

let p = new Point {x: 3, y: 4}
print distance_from_origin(p)  # 5.0
```

---

## Module System

### Exporting

```ject
# math_utils.ject

export fn add(a, b)
    return a + b
end

export fn multiply(a, b)
    return a * b
end

export PI = 3.14159
```

### Importing

#### Selective Import

```ject
import {add, PI} from "math_utils"

print add(2, 3)  # 5
print PI         # 3.14159
```

#### Import with Alias

```ject
import "math_utils" as math

print math.add(2, 3)
print math.PI
```

#### Full Import

```ject
import "math_utils"

# Exports are available directly
print add(2, 3)
print PI
```

### Standard Library Modules

```ject
# Math module
import "math"
print sqrt(16)  # 4
print sin(PI / 2)  # 1

# String utilities
import "strings"
print capitalize("hello")  # "Hello"

# File I/O
import "io"
let content = read_file("data.txt")
write_file("output.txt", "Hello")
```

---

## Error Handling

### Try/Catch

```ject
try
    let result = risky_operation()
    print "Success: " + result
catch error
    print "Error occurred: " + error
end
```

### Throwing Errors

```ject
fn divide(a, b)
    if b == 0 then
        throw "Division by zero!"
    end
    return a / b
end

try
    print divide(10, 0)
catch error
    print "Caught: " + error
end
```

### Catch Without Variable

```ject
try
    do_something_risky()
catch
    print "Something went wrong"
end
```

---

## Standard Library

Ject has a two-tier standard library system:

### CorLib (Core Library - Always Available)

These essential functions are **always available** without any import:

```ject
# Type inspection
type_of(42)         # "int"
type_of(3.14)       # "float"

# Collection operations
len([1, 2, 3])      # 3
range(1, 4)         # [1, 2, 3]

# Array mutations
let arr = [1, 2]
push(arr, 3)        # [1, 2, 3]
pop(arr)            # returns 3, arr = [1, 2]

# Higher-order functions (functional programming)
map([1, 2, 3], lambda(x) -> x * 2)     # [2, 4, 6]
filter([1, 2, 3], lambda(x) -> x > 1)  # [2, 3]
reduce([1, 2, 3], lambda(a, b) -> a + b, 0)  # 6
```

### Standard Library Modules (Require Import)

Additional functionality is organized into **standalone modules** that must be explicitly imported:

#### Math Module
```ject
import "math"
# or
import {sqrt, PI} from "math"

abs(-42)        # 42
sqrt(16)        # 4
pow(2, 8)       # 256
sin(PI / 2)     # 1
cos(0)          # 1
floor(3.7)      # 3
ceil(3.2)       # 4
round(3.5)      # 4
min(1, 5, 3)    # 1
max(1, 5, 3)    # 5
log(10)         # 2.302...
exp(1)          # 2.718...
degrees(PI)     # 180.0
radians(180)    # 3.14159...
clamp(5, 0, 10) # 5

# Constants
PI    # 3.141592653589793
E     # 2.718281828459045
```

#### String Module
```ject
import "string"
# or
import {upper, lower, trim} from "string"

upper("hello")           # "HELLO"
lower("HELLO")           # "hello"
capitalize("hello")      # "Hello"
title_case("hello world") # "Hello World"
trim("  hi  ")           # "hi"
pad_left("5", 3, "0")    # "005"
pad_right("5", 3, "0")   # "500"
starts_with("hello", "he")  # true
ends_with("hello", "lo")    # true
contains_str("hello", "ell") # true
replace("hello", "l", "L")   # "heLLo"
repeat("ab", 3)         # "ababab"
split("a,b,c", ",")     # ["a", "b", "c"]
join(["a", "b"], "-")   # "a-b"
```

#### Array Module
```ject
import "array"
# or
import {sort, sum, unique} from "array"

let arr = [3, 1, 4, 1, 5]

sum(arr)           # 14
any(arr, lambda(x) -> x > 4)  # true
all(arr, lambda(x) -> x > 0)  # true
sort(arr)          # [1, 1, 3, 4, 5]
reverse(arr)       # [5, 1, 4, 1, 3]
contains(arr, 4)   # true
index_of(arr, 4)   # 2
unique(arr)        # [3, 1, 4, 5]
slice(arr, 1, 3)   # [1, 4]
take(arr, 2)       # [3, 1]
drop(arr, 2)       # [4, 1, 5]
first(arr)         # 3
last(arr)          # 5
concat([1, 2], [3, 4])  # [1, 2, 3, 4]
zip([1, 2], [3, 4])     # [[1, 3], [2, 4]]
enumerate(arr)     # [[0, 3], [1, 1], [2, 4], ...]
flatten([[1, 2], [3, 4]])  # [1, 2, 3, 4]
```

#### Other Modules

```ject
# IO module
import "io"
let content = read_file("data.txt")
write_file("output.txt", "Hello")

# JSON module
import "json"
let obj = parse_json("{\"name\": \"Alice\"}")
let json_str = to_json(obj)

# System module
import "system"
let home = env("HOME")
sleep(1000)  # Sleep for 1000ms
now()        # Current datetime
timestamp()  # Unix timestamp

# Base conversion module
import "base"
to_binary(42)    # "101010"
to_hex(255)      # "ff"
from_binary("101010")  # 42
from_hex("ff")   # 255
```

---

## REPL

### Starting the REPL

```bash
./target/release/ject
```

### REPL Features

```
Ject REPL - v0.1.0
Use arrow keys to access history
'exit' to, well, exit

ject› let x = 10
ject› let y = 20
ject› print x + y
30
ject› exit
Goodbye!
```

### REPL Commands

- `exit` - Exit the REPL
- Arrow keys - Navigate history
- Ctrl+C - Cancel current line

---

## Best Practices

### Naming Conventions

```ject
# Use snake_case for variables and functions
let user_name = "Alice"
fn calculate_total()

# Use PascalCase for structs
struct UserProfile {
    name,
    email
}
```

### Code Organization

```ject
# 1. Imports at the top
import {add, multiply} from "math_utils"

# 2. Struct definitions
struct Point {x, y}

# 3. Function definitions
fn main()
    # Your code here
end

# 4. Main execution
main()
```

### Error Handling

```ject
# Prefer specific error messages
try
    let data = read_file("config.json")
catch error
    print "Failed to load config: " + error
end

# Validate inputs
fn divide(a, b)
    if b == 0 then
        throw "Cannot divide by zero"
    end
    return a / b
end
```

### Performance Tips

```ject
# Use ranges instead of creating arrays
for i in 1..1000 do  # Good
    # ...
end

# Prefer local variables
fn process()
    let cache = {}  # Local is faster than global
    # ...
end
```

---

## Comparison with Other Languages

### Ject vs Python

```ject
# Ject - Clean elseif
if score >= 90
    print "A"
elseif score >= 80
    print "B"
else
    print "C"
end
```

```python
# Python - elif (inconsistent)
if score >= 90:
    print("A")
elif score >= 80:
    print("B")
else:
    print("C")
```

### Ject vs JavaScript

```ject
# Ject - Explicit blocks
fn greet(name)
    print "Hello, " + name
end
```

```javascript
// JavaScript - Braces required
function greet(name) {
    console.log("Hello, " + name);
}
```

### Ject vs Ruby

```ject
# Ject - Similar elegance
let numbers = [1, 2, 3]
for n in numbers do
    print n
end
```

```ruby
# Ruby
numbers = [1, 2, 3]
numbers.each do |n|
  puts n
end
```

---

## Quick Reference Card

### Variables
```ject
let x = 10      # Declare
x = 20          # Reassign
```

### Functions
```ject
fn name(params)
    return value
end

let lambda = lambda(x) -> x * 2
```

### Control Flow
```ject
if cond
    # ...
elseif cond
    # ...
else
    # ...
end

while cond do
    # ...
end

for item in array do
    # ...
end
```

### Collections
```ject
let arr = [1, 2, 3]
let dict = {key: "value"}

arr[from:1 to:3]    # Slice
arr[1..3]           # Range slice
arr[1:3]            # Python slice
```

### Strings
```ject
"Hello ${name}!"    # Interpolation
text[0..5]          # Slice
upper(text)         # Uppercase
```

---

## Getting Help

- Check the `examples/` directory for sample code
- Review `stdlib/` for available modules
- Report issues on the project repository

---

**Made with ❤️ in Rust**

*Ject - Simple, Expressive, Fast*
