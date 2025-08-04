# 🎨 Ject - The Elegant Scripting Language

**Ject** is a beautifully simple, Crystal-inspired scripting language designed for rapid development and elegant code. Built in Rust for maximum performance, Ject combines the simplicity you love about Python with the speed and elegance of modern language design.

## ✨ Features

- **End-based syntax** - Clean, readable blocks with `end` keywords
- **Dynamic typing** - Write code fast without type annotations
- **First-class functions** - Functions are values, pass them around freely
- **Beautiful arrays** - `[1, 2, 3]` syntax that just works
- **Intuitive control flow** - `if/else`, `while`, `for` loops that read like English
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

Ject provides intuitive control flow constructs:

```ject
# If-else statements
let score = 85

if score >= 90 then
    print "A grade!"
else if score >= 80 then
    print "B grade!"
else
    print "Keep trying!"
end

# While loops
let counter = 0
while counter < 5 do
    print "Count: " + counter
    counter = counter + 1
end

# For loops
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

- [ ] **Module system** - Import/export functionality
- [ ] **Standard library** - File I/O, HTTP, JSON, etc.
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
