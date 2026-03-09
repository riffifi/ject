# Ject Language Architecture - Import System

## Core Design (C++ Style)

### Core Builtins (Auto-Imported)
Implemented in Rust, always available without import:

```ject
# No import needed!
print abs(-42)
print sqrt(144)
print PI
print len([1,2,3])
```

**Includes:**
- Math: `abs`, `sqrt`, `pow`, `sin`, `cos`, `tan`, `floor`, `ceil`, `round`, `min`, `max`
- Arrays: `len`, `push`, `pop`, `sum`
- Strings: `upper`, `lower`, `trim`, `split`, `join`, `replace`
- Utils: `type_of`, `range`, `random`
- Constants: `PI`, `E`

### Standard Library (Explicit Import Required)
Implemented in Ject, must be imported:

```ject
# Must import explicitly (C++ style #include logic)
import "math"
print factorial(6)

import "array"  
print flatten([[1,2],[3,4]])

import "string"
print capitalize("hello")

# Or selective imports
import {factorial, fibonacci} from "math"
import {sort, filter} from "array"
```

## Module Files

### stdlib/math.ject
- `factorial(n)`, `fibonacci(n)`, `gcd(a,b)`, `lcm(a,b)`
- `is_prime(n)`, `primes_up_to(n)`
- `deg_to_rad(deg)`, `rad_to_deg(rad)`
- `sinh`, `cosh`, `tanh`, `asinh`, `acosh`, `atanh`
- `lerp`, `inverse_lerp`, `remap`
- `distance_2d`, `distance_3d`
- And 40+ more functions...

### stdlib/array.ject  
- `flatten`, `chunk`, `group_by`, `partition`
- `shuffle`, `rotate_left`, `rotate_right`
- `union`, `intersection`, `difference`
- `sort_by`, `compact`, `sample`
- And 50+ more functions...

### stdlib/string.ject
- `capitalize`, `title_case`, `truncate`
- `pad_left`, `pad_right`, `pad_center`
- `word_count`, `sentence_count`, `paragraph_count`
- `wrap_text`, `extract_numbers`
- And 40+ more functions...

## Usage Examples

### Minimal Program (Core Builtins Only)
```ject
# No imports needed for basic operations
let numbers = [1, 2, 3, 4, 5]
print sum(numbers)  # 15
print sqrt(25)      # 5
```

### With Stdlib
```ject
import "math"
import "array"

let data = [[1, 2], [3, 4]]
print flatten(data)  # [1, 2, 3, 4]
print factorial(5)   # 120
```

### Selective Imports
```ject
import {PI, factorial} from "math"
import {sort, filter} from "array"

print PI
print factorial(6)
```

## Implementation Status

✅ Core builtins auto-imported (Rust implementations)
✅ Import syntax working (`import "module"`)
✅ Selective imports working (`import {x, y} from "module"`)
✅ Module aliases working (`import "math" as m`)
⚠️ Stdlib module exports need minor fix (functions not being collected properly)

## Next Steps

The import architecture is correct. The stdlib modules need the export collection in `load_module()` to properly expose `export fn` declarations. This is a minor fix in the interpreter's module loading logic.

---

**Architecture:** C++ style - import what you use  
**Core Builtins:** Always available  
**Stdlib:** Explicit import required  
**Philosophy:** Lean, explicit dependencies
