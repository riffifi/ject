# Ject Missing Features Report

**Date:** March 5, 2026
**Method:** Discovered by attempting to build real programs in Ject

---

## Critical Missing Features

### 1. Array/Dictionary Index Assignment ❌
**Priority:** CRITICAL

Cannot modify array elements or dictionary keys:
```ject
let arr = [1, 2, 3]
arr[0] = 10      # ERROR: Unexpected token: Equal

let dict = {x: 1}
dict.x = 5       # ERROR: Cannot assign to struct field
dict["x"] = 5    # ERROR: Not supported
```

**Impact:** Makes data manipulation extremely cumbersome. Need to rebuild entire arrays for simple changes.

**Fix Required:** 
- Parse index expressions on left side of assignment
- Support both `arr[i] = val` and `dict[key] = val`
- Support struct field assignment: `obj.field = val`

---

### 2. Compound Assignment Operators ❌
**Priority:** HIGH

No shorthand for common operations:
```ject
x = x + 1   # Works but verbose
x += 1      # ERROR: Not supported

x = x * 2
x *= 2      # ERROR: Not supported

message = message + "hello"
message += "hello"  # ERROR: Not supported
```

**Missing operators:** `+=`, `-=`, `*=`, `/=`, `%=`, `&&=`, `||=`

**Impact:** Verbose code, especially for counters and string building.

---

### 3. Increment/Decrement Operators ❌
**Priority:** MEDIUM

No shorthand for incrementing:
```ject
counter = counter + 1  # Works
counter++              # ERROR: Not supported
++counter              # ERROR: Not supported
counter--              # ERROR: Not supported
```

**Impact:** Minor inconvenience for common pattern.

---

### 4. Break and Continue Statements ❌
**Priority:** HIGH

Cannot control loop flow:
```ject
for item in items do
    if item == target then
        break    # ERROR: Not supported
    end
    
    if skip_condition then
        continue  # ERROR: Not supported
    end
end
```

**Impact:** 
- Cannot exit loops early (search patterns inefficient)
- Cannot skip iterations cleanly (need nested if)
- Makes some algorithms awkward or impossible

---

### 5. Multi-line Array/Object Literals ❌
**Priority:** MEDIUM

Parser doesn't handle newlines in literals:
```ject
let matrix = [
    [1, 2, 3],  # ERROR: Unexpected token: Newline
    [4, 5, 6],
    [7, 8, 9]
]
```

**Impact:** Hard to read complex data structures.

---

## Standard Library Gaps

### 6. Missing Common Functions ❌

**String:**
- `format()` - String formatting with placeholders
- `starts_with()`, `ends_with()` - Already in Rust, need Ject wrappers
- `pad_left()`, `pad_right()` - Already in Rust, need Ject wrappers

**Array:**
- `shift()`, `unshift()` - Remove/add from beginning
- `insert()` - Insert at position
- `remove()` - Remove at position

**Dictionary:**
- `has_key()` - Check if key exists
- `delete()` - Remove key
- `keys()`, `values()` - Get keys/values as arrays

**File I/O:**
- `append_file()` - Append to file
- `read_lines()` - Read file line by line
- `file_exists()` - Check if file exists (in Rust, need Ject wrapper)

**System:**
- `args()` - Get command line arguments
- `cwd()` - Get current working directory
- `exit()` - Exit program (in Rust, need Ject wrapper)

---

### 7. No Module System Implementation ❌
**Priority:** HIGH

Import/export keywords exist but no working module system:
```ject
import "math"           # Doesn't work
import {PI} from "math" # Doesn't work
```

**Impact:** Cannot organize code into modules, no code reuse.

---

## Nice-to-Have Features

### 8. String Interpolation Improvements ❌
**Priority:** LOW

Current:
```ject
let greeting = "Hello, " + name + "! You are " + age + " years old."
```

Would be nicer:
```ject
let greeting = "Hello, {name}! You are {age} years old."
# or
let greeting = $"Hello, {name}! You are {age} years old."
```

---

### 9. Default Function Parameters ⚠️
**Priority:** LOW

Parser supports it but unclear if interpreter handles it:
```ject
fn greet(name, greeting = "Hello")
    print greeting + ", " + name
end
```

---

### 10. Variadic Functions ❌
**Priority:** MEDIUM

Cannot accept variable number of arguments:
```ject
fn sum(...numbers)  # ERROR: Not supported
    # Would need to iterate over numbers
end
```

---

### 11. Pattern Matching Enhancements ❌
**Priority:** LOW

Match exists but limited:
```ject
match value
    1 => print "one"
    2 => print "two"
    _ => print "other"  # Wildcard works
end
```

Missing:
- Guard conditions: `n if n > 10 => ...`
- Destructuring: `{name, age} => ...`

---

### 12. Const/Let Immutability ⚠️
**Priority:** LOW

`let` variables can be reassigned:
```ject
let x = 10
x = 20  # Works (should this?)
```

No `const` for true constants.

---

## Summary by Priority

### Must Have (Breaking for usability)
1. ✅ Array/Dictionary index assignment
2. ✅ Break/continue statements  
3. ✅ Compound assignment operators

### Should Have (Significant quality of life)
4. Multi-line literals
5. Module system
6. Missing stdlib functions (has_key, delete, etc.)

### Nice to Have
7. Increment/decrement operators
8. String interpolation improvements
9. Variadic functions
10. Pattern matching enhancements

---

## Next Steps

1. **Implement index assignment** - Most critical for basic programming
2. **Implement break/continue** - Essential for loop control
3. **Implement compound assignment** - Major quality of life improvement
4. **Fix multi-line literals** - Parser enhancement
5. **Add missing stdlib functions** - Complete the standard library
