# Ject

**Ject** is a scripting/programming language. Currently my 'pet' project so to speak. I'm trying my best

## Quick Start

### Installation

```bash
git clone <this-repo>
cd ject
cargo build --release
./target/release/ject
```

### Your First Ject Program

```ject
# Hello, World!
print "Hello, World!"

# Variables are simple
let name = "Ject"
let version = 1.0
let is_awesome = true

# Functions are beautiful
fn greet(name)
    print "Hello, " + name + "!"
end

greet("World")

# Arrays and loops
let numbers = [1, 2, 3, 4, 5]
let sum = 0

for num in numbers do
    sum = sum + num
end

print "Sum: " + sum  # Sum: 15
```
NOTICE that unlike Python or C-Based languages, Ject has Ruby or Crystal-like syntax that relies on end-based blocks instead of indentation or brackets!

## Language Guide

### Variables

Variables in Ject are declared with `let` and are dynamically typed:

```ject
# You declare them with 'let'

let name = "Alice"
let age = 30
let height = 5.8
let is_student = false
let empty = nil

# But after that you can rewrite them without using the 'let' keyword
name = "Walter White"
age = 50
height = 5.11
is_student = false
empty = "not so empty anymore!"
```

### Functions

Functions are first-class citizens in Ject (that means they can be treated like any other value):

```ject
# Simple function
fn say_hello()
    print "Hello!"
end

# Function with parameters
fn add(a, b)
    return a + b
end

# Functions can return values
fn fibonacci(n)
    if n <= 1 then
        return n
    else
        return fibonacci(n - 1) + fibonacci(n - 2)
    end
end

let result = fibonacci(10)
print result  # 55
```

### Lambda Functions

Ject supports beautiful lambda expressions for anonymous functions:

```ject
# Basic lambda with single expression
let square = lambda(x) -> x * x
print square(5)  # 25

# Lambda with multiple parameters
let add = lambda(a, b) -> a + b
print add(10, 5)  # 15

# Lambda with no parameters
let get_pi = lambda() -> 3.14159
print get_pi()  # 3.14159

# Using lambdas with arrays
let numbers = [1, 2, 3, 4, 5]
let double = lambda(n) -> n * 2

for num in numbers do
    print double(num)
end
# Output: 2, 4, 6, 8, 10

# Lambdas for quick calculations
let distance = lambda(x1, y1, x2, y2) -> ((x2-x1)*(x2-x1) + (y2-y1)*(y2-y1))
print distance(0, 0, 3, 4)  # 25 (distance squared)

# Assignment and reassignment
let operation = lambda(x) -> x + 1
print operation(5)  # 6

operation = lambda(x) -> x * x
print operation(5)  # 25
```

However, for more complex lambdas you would have to use a slightly different syntax
```ject
let test = lambda(x) -> {
    for i in 0..x do print "test" end
}
test(5)
```

### Control Flow

```ject
let score = 85

# You can use either elseif
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

# OR, if you prefer spaces more, else if, no judgement!
if temperature < 0
    print "Freezing!"
else if temperature < 30
    print "Nice weather"
else
    print "Getting warm!"
end

# While loops
let counter = 0
while counter < 5 do
    print "Count: " + counter
    counter = counter + 1
end

# For loops with range function (python-like)
for i in range(1, 6) do
    print "Number: " + i
end

# Or more Rust-like start..stop with addition of step -- start..stop:step
for i in 1..6 do
    print "Number: " + i
end

# Ranges with steps
for i in 2..10:2 do
    print "Even: " + i
end

# For loops with arrays
let fruits = ["apple", "banana", "cherry"]
for fruit in fruits do
    print "I love " + fruit
end
```

### Range Syntax

```ject
# Basic ranges
let numbers = 1..5        # [1, 2, 3, 4]
let sequence = 0..10      # [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]

# Ranges with steps using colon syntax
let odds = 1..10:2        # [1, 3, 5, 7, 9]
let evens = 2..10:2       # [2, 4, 6, 8]
let fives = 0..20:5       # [0, 5, 10, 15]

# Reverse ranges
let countdown = 10..0:-1  # [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]
let down_by_twos = 10..0:-2  # [10, 8, 6, 4, 2]

# Perfect for loops, btw, you have my word for it
for i in 1..6 do
    print "Count: " + i
end

for i in 5..0:-1 do
    print "Countdown: " + i
end

# Compare with traditional range() function
print range(1, 6)    # [1, 2, 3, 4, 5]
print 1..6           # [1, 2, 3, 4, 5] - same result, but looks doper and faster to write!!!
```

### Arrays

Arrays in Ject are dynamic and easy to use:

```ject
# Create arrays
let empty_array = []
let numbers = [1, 2, 3, 4, 5]
let mixed = [1, "hello", true, 3.14]

# Access elements
print numbers[0]  # 1
print numbers[2]  # 3

# Arrays in loops
for item in mixed do
    print item
end
```

### Operators

Ject supports all the operators you expect:

```ject
# Arithmetic
let sum = 10 + 5      # 15
let diff = 10 - 5     # 5
let product = 10 * 5  # 50
let quotient = 10 / 5 # 2.0
let remainder = 10 % 3 # 1

# Comparison
let equal = 5 == 5        # true
let not_equal = 5 != 3    # true
let greater = 10 > 5      # true
let less_equal = 5 <= 10  # true

# Logical
let and_result = true and false  # false
let or_result = true or false    # true
let not_result = !true           # false
```

### Module System

Ject supports a clean module system with import/export functionality:

```ject
# Selective imports - import specific functions/values
import {RED, GREEN, colorize} from "colors"
print colorize("Hello!", RED)

# Module aliases - import whole modules with custom names
import "ascii_art" as art
print art.banner("Welcome!")

# Full imports - import all exports directly
import "game_utils"
let dice_roll = roll_dice(6)
```

### Standard Library

Ject comes with a standart library (does not require import, UNLIKE SOME LANGUAGES. I'M LOOKING AT YOU C++):

```ject
# Mathematical functions
let absolute = abs(-42)        # 42
let square_root = sqrt(16)     # 4
let power = pow(2, 8)          # 256
let rounded = round(3.14159)   # 3

# Trigonometric functions
let sine = sin(PI / 2)         # 1
let cosine = cos(0)            # 1
let tangent = tan(PI / 4)      # 1

# Array functions
let data = [1, 5, 3, 9, 2]
let length = len(data)         # 5
let total = sum(data)          # 20
let maximum = max(1, 5, 3, 9, 2)  # 9
let minimum = min(1, 5, 3, 9, 2)  # 1
let sequence = range(1, 6)     # [1, 2, 3, 4, 5]

# String functions  
let text = "  Hello World  "
let upper_case = upper(text)   # "  HELLO WORLD  "
let lower_case = lower(text)   # "  hello world  "
let trimmed = trim(text)       # "Hello World"

# Advanced string operations
let csv = "apple,banana,cherry"
let fruits = split(csv, ",")   # ["apple", "banana", "cherry"]
let joined = join(fruits, " | ")  # "apple | banana | cherry"
let fixed = replace("Hello Python", "Python", "Ject")  # "Hello Ject"

# Utility functions
let data_type = type_of(42)    # "integer"
print "Value: " + data_type

# Mathematical constants
print "PI = " + PI            # 3.141592653589793
print "E = " + E              # 2.718281828459045
```

## Why Choose Ject Over Python?

Ject improves upon Python's design with several key advantages:

### Cleaner Conditional Logic
```ject
# Ject - Native elseif keyword
if score >= 95
    print "A+"
elseif score >= 90  
    print "A"
elseif score >= 80
    print "B"
else
    print "Below B"
end
```

```python
# Python - Awkward elif keyword
if score >= 95:
    print("A+")
elif score >= 90:  # elif feels inconsistent
    print("A")
elif score >= 80:
    print("B")
else:
    print("Below B")
```

### Explicit Block Endings
```ject
# Ject - Clear block boundaries with 'end'
if condition
    do_something()
    if nested_condition
        do_nested_thing()
    end
end
```

```python
# Python - Indentation-dependent (fragile)
if condition:
    do_something()
    if nested_condition:
        do_nested_thing()  # Easy to mess up indentation
```

### Built-in Mathematical Functions
```ject
# Ject - Rich math stdlib included
let result = sqrt(pow(abs(-16), 2))
let angle = sin(PI / 4)
```

```python
# Python - Need to import math module
import math
result = math.sqrt(math.pow(abs(-16), 2))
angle = math.sin(math.pi / 4)
```

## Design Philosophy

Ject was designed with these principles in mind:

1. **Simplicity First** - Clean, uncomplicated syntax without unnecessary complexity
2. **Readable Code** - Code should be easy to read and understand
3. **Fast Development** - Quick iteration and minimal boilerplate
4. **Performance Matters** - Built in Rust for speed and memory safety
5. **Enjoyable Syntax** - Pleasant to write and maintain

## Architecture

- **Lexer** - Tokenizes source code with Unicode support
- **Parser** - Recursive descent parser building a clean AST
- **Interpreter** - Tree-walking interpreter with proper scoping
- **Value System** - Dynamic typing with efficient representations
- **Environment** - Lexical scoping with nested environments

## Roadmap

- [x] **Native elseif keyword** - Clean conditional chains
- [x] **Rich standard library** - Math, arrays, strings, utilities
- [x] **Comprehensive examples** - Real-world data analysis demos
- [x] **Range syntax** - Python-inspired `start..end:step` notation
- [x] **Enhanced REPL** - Arrow key navigation, command history, and line editing
- [x] **Lambda functions** - Anonymous function expressions
- [x] **Module system** - Import/export functionality
- [ ] **Extended standard library** - File I/O, HTTP, JSON, etc.
- [ ] **Package manager** - Easy dependency management
- [ ] **Advanced REPL features** - Syntax highlighting, autocomplete, better error messages
- [ ] **Compiled mode** - Optional compilation for production use
- [ ] **VSCode extension** - Syntax highlighting and language support

## Inspiration

Ject draws inspiration from:
- **Crystal** - Elegant syntax and performance focus
- **Ruby** - Expressiveness and readability
- **Python** - User-friendly design
- **Rust** - Safety and performance

---

**Made with passion and dedication**

*Hope you enjoy it!*
