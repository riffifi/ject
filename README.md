# 🎨 Ject - The Elegant Scripting Language

**Ject** is a beautifully simple, Crystal-inspired scripting language designed for rapid development and elegant code. Built in Rust for maximum performance, Ject combines the simplicity you love about Python with the speed and elegance of modern language design.

## ✨ Features

- **End-based syntax** - Clean, readable blocks with `end` keywords
- **Native elseif keyword** - Elegant conditional chains without nested mess
- **Rich standard library** - Math, arrays, strings, and utility functions built-in
- **Dynamic typing** - Write code fast without type annotations
- **First-class functions** - Functions are values, pass them around freely
- **Beautiful arrays** - `[1, 2, 3]` syntax with powerful built-in operations
- **Intuitive control flow** - `if/elseif/else`, `while`, `for` loops that read like English
- **Lightning fast** - Built in Rust for optimal performance
- **Zero dependencies** - Single binary, install anywhere

## 🚀 Quick Start

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

## 📖 Language Guide

### Variables

Variables in Ject are declared with `let` and are dynamically typed:

```ject
let name = "Alice"
let age = 30
let height = 5.8
let is_student = false
let empty = nil
```

### Functions

Functions are first-class citizens in Ject:

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

### Control Flow

Ject provides intuitive control flow constructs with elegant elseif support:

```ject
# Native elseif keyword - clean and readable!
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

# Traditional else if still works too
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

# For loops with range function
for i in range(1, 6) do
    print "Number: " + i
end

# For loops with arrays
let fruits = ["apple", "banana", "cherry"]
for fruit in fruits do
    print "I love " + fruit
end
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

### Standard Library

Ject comes with a rich standard library for common tasks:

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

# Utility functions
let data_type = type_of(42)    # "integer"
print "Value: " + data_type

# Mathematical constants
print "PI = " + PI            # 3.141592653589793
print "E = " + E              # 2.718281828459045
```

## 🥇 Why Choose Ject Over Python?

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

## 🎯 Design Philosophy

Ject was designed with these principles in mind:

1. **Simplicity First** - If it's not simple, it doesn't belong
2. **Readable Code** - Code should read like natural language
3. **Fast Development** - Get things done quickly without ceremony
4. **Performance Matters** - Built in Rust for speed where it counts
5. **Beautiful Syntax** - Code should be a joy to write and read

## 🏗️ Architecture

Ject is built with a clean, modular architecture:

- **Lexer** - Tokenizes source code with Unicode support
- **Parser** - Recursive descent parser building a clean AST
- **Interpreter** - Tree-walking interpreter with proper scoping
- **Value System** - Dynamic typing with efficient representations
- **Environment** - Lexical scoping with nested environments

## 🤝 Contributing

Ject is built with love and we welcome contributions! Here are some ways you can help:

- 🐛 Report bugs and issues
- 💡 Suggest new features
- 📝 Improve documentation
- 🔧 Submit pull requests
- ⭐ Star the repository

## 📈 Roadmap

- [x] **Native elseif keyword** - Clean conditional chains ✅
- [x] **Rich standard library** - Math, arrays, strings, utilities ✅
- [x] **Comprehensive examples** - Real-world data analysis demos ✅
- [ ] **Module system** - Import/export functionality
- [ ] **Extended standard library** - File I/O, HTTP, JSON, etc.
- [ ] **Lambda functions** - Anonymous function expressions
- [ ] **Package manager** - Easy dependency management
- [ ] **REPL improvements** - Better error messages, syntax highlighting
- [ ] **Compiled mode** - Optional compilation for production use
- [ ] **VSCode extension** - Syntax highlighting and language support

## 📄 License

Ject is released under the MIT License. See [LICENSE](LICENSE) for details.

## 🙏 Inspiration

Ject draws inspiration from:
- **Crystal** - For its beautiful syntax and performance focus
- **Ruby** - For its expressiveness and developer happiness
- **Python** - For its simplicity and readability
- **Rust** - For its safety and performance guarantees

---

**Made with ❤️ for developers who believe code should be beautiful**

*Start your Ject journey today - where elegance meets performance!*
