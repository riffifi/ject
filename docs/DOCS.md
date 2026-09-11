# The Ject guide

This is the user guide and language reference for Ject 0.10.0. It starts with the
smallest useful programs, then builds toward modules, packages, editor tooling, and
the complete syntax. Package authors who need Rust should continue with
[Packages and native libraries](PACKAGES.md).

## 1. Running Ject

Start an interactive session with no arguments:

```sh
ject
```

Run a standalone source file:

```sh
ject program.ject
```

Inside a directory containing `Ject.toml`, use the project commands:

```sh
ject run
ject check
ject test
ject build
```

`run` executes the package entry. `check` parses and lints every source without
running it. `test` executes each file in `tests/`. `build` checks source and also
builds native components when the package has them.

The smallest program is:

```ject
print "Hello, Ject!"
```

Ject is case-sensitive. `score`, `Score`, and `SCORE` are different names.

## 2. Source files

Ject source files use the `.ject` extension and UTF-8 text. Newlines normally end
statements. A semicolon can separate short statements on the same line:

```ject
let left = 2; let right = 3; print left + right
```

Single-line comments begin with `#`:

```ject
# The tax rate is deliberately stable for this calculation.
const TAX_RATE = 0.2
```

Multi-line comments begin with `#*` and end at the first `*#`. They do not nest:

```ject
#*
Draft implementation.
*#
print "ready"
```

Blocks use indentation for readability and `end` for structure. Four spaces are the
recommended indentation width.

## 3. Values and types

Ject is dynamically typed. Values have runtime types; bindings do not need type
annotations.

```ject
let count = 12
let ratio = 0.75
let title = "Ject"
let enabled = true
let missing = nil
let items = [1, 2, 3]
let user = {name: "Ada", active: true}
```

The core runtime type names are:

| Kind | Examples | `type_of` result |
| --- | --- | --- |
| Integer | `0`, `-12`, `42` | `"integer"` |
| Float | `0.0`, `-1.5`, `2.0` | `"float"` |
| String | `"hello"` | `"string"` |
| Boolean | `true`, `false` | `"boolean"` |
| Nil | `nil` | `"nil"` |
| Array | `[1, 2]` | `"array"` |
| Unique array | `{|1, 2|}` | `"unique_array"` |
| Dictionary | `{name: "Ada"}` | `"dictionary"` |
| Function | `fn(value) -> value` | `"function"` |

Struct instances and native resources have their own runtime names. For example,
JNUM exposes `ndarray` and JGUI exposes `jgui_window`.

Integers and floats are intentionally distinct:

```ject
assert(type_of(2) == "integer")
assert(type_of(2.0) == "float")
```

Ject uses one 64-bit floating-point type. It does not have an `f` suffix or a second,
less-precise float type.

### Explicit conversions

Use conversions at input and serialization boundaries:

```ject
let count = to_int("42")
let ratio = to_float("3.5")
let text = to_string(42)
let flag = to_bool(1)
```

Invalid conversions report runtime errors rather than inventing a fallback value.

## 4. Bindings: `let` and `const`

Use `let` for a binding that may be assigned again:

```ject
let score = 10
score = score + 5
score += 2
score++
```

Assignment to a name that has never been declared is an error. This turns many
spelling mistakes into immediate diagnostics.

Use `const` when code must not assign through that binding:

```ject
const MAX_RETRIES = 3
const supported = ["wav", "flac"]
```

The following operations are rejected:

```text
MAX_RETRIES = 4
MAX_RETRIES++
supported[0] = "mp3"
```

`const` protects a binding; it is not a recursive object freezer. If a mutable alias
points to the same array or dictionary, mutation through that alias remains visible:

```ject
const settings = {theme: "dark"}
let editable = settings
editable.theme = "light"
assert(settings.theme == "light")
```

This distinction keeps binding semantics predictable without adding a hidden deep-copy
or deep-freeze cost.

Bindings follow lexical scope. A binding created inside a function, branch, loop, or
match arm is not visible after that scope ends.

Declaring the same name again in one scope creates a new binding and emits a warning.
Prefer assignment for an existing `let` and a distinct name for an existing `const`.

## 5. Strings and printing

Strings use double quotes. Common escapes include `\n`, `\t`, `\r`, `\"`, and `\\`.

Insert a simple name with `$name`, or an expression with `${...}`:

```ject
let name = "Ada"
let completed = 4
let total = 6
print "Hello, $name"
print "Progress: ${completed * 100 / total}%"
```

`print` accepts multiple values. Named options use the same `name=value` spelling as
function calls:

```ject
print "red", "green", "blue", sep=" | ", end="\n"
```

The older `sep:` and `end:` spellings remain accepted for compatibility, but new code
should use `=`.

## 6. Operators

Arithmetic:

```ject
let total = 10 + 3
let difference = 10 - 3
let product = 10 * 3
let quotient = 10 / 3
let remainder = 10 % 3
```

Comparison:

```ject
assert(2 == 2)
assert(2 != 3)
assert(2 < 3)
assert(2 <= 2)
assert(3 > 2)
assert(3 >= 3)
```

Logical operators use words:

```ject
let can_start = ready and connected
let should_refresh = stale or forced
let should_wait = not ready
```

`!ready` is the compact equivalent of `not ready`. `and` and `or` short-circuit, so
the right side is evaluated only when needed.

Membership uses `in`:

```ject
assert(2 in [1, 2, 3])
assert("ect" in "Ject")
assert("name" in {name: "Ada"})
```

From strongest to weakest, the main precedence groups are calls/member/index access,
unary operators, multiplication/division/modulo, addition/subtraction, ranges,
comparisons and membership, equality, `and`, then `or`. Use parentheses when the
intended grouping would not be obvious to a reader.

## 7. Conditions and loops

### Conditions

```ject
if temperature > 30
    print "hot"
elseif temperature < 10
    print "cold"
else
    print "mild"
end
```

`then` is optional in statement-form conditions. It is required in the compact
conditional expression:

```ject
let label = if active then "online" else "offline" end
```

Conditions use truthiness. `false`, `nil`, zero, empty strings, and empty containers
are false-like; other values are true-like. Prefer an explicit comparison when it
makes intent clearer.

### `while`

```ject
let remaining = 3
while remaining > 0
    print remaining
    remaining--
end
```

`do` is accepted after a `while` condition but is optional.

### `for`

```ject
for name in ["Ada", "Lin"]
    print name
end

for index in 0..5
    print index
end
```

Ranges are end-exclusive. `0..5` produces `0`, `1`, `2`, `3`, and `4`. Add a step
after a colon:

```ject
for even in 0..10:2
    print even
end

for descending in 5..0:-1
    print descending
end
```

`break` leaves the nearest loop. `continue` begins its next iteration.

## 8. Functions and closures

Declare a named function with `fn`:

```ject
fn add(left, right)
    return left + right
end

print add(2, 3)
```

A function that reaches the end without `return` produces `nil`.

Parameters may have defaults. Required parameters must appear before defaulted ones:

```ject
fn greet(name, greeting="Hello", punctuation="!")
    return "$greeting, $name$punctuation"
end

print greet("Ada")
print greet("Ada", "Welcome")
print greet(punctuation="?", name="Lin")
```

Named arguments use `name=value`. They may be reordered, but a parameter cannot be
provided twice and unknown parameter names are errors.

Functions are values. A short anonymous function uses `fn(parameters) -> expression`:

```ject
let square = fn(value) -> value * value
print [1, 2, 3].map(square)
```

Use a block body for more work:

```ject
let describe = fn(value)
    if value > 0
        return "positive"
    end
    return "not positive"
end
```

Anonymous functions capture the lexical environment in which they are created:

```ject
fn multiplier(factor)
    return fn(value) -> value * factor
end

let triple = multiplier(3)
assert(triple(4) == 12)
```

`lambda(...)` remains a compatibility alias for anonymous `fn(...)`; `fn` is the
canonical spelling for new code.

## 9. Arrays, dictionaries, and comprehensions

### Arrays

```ject
let values = [10, 20, 30]
assert(values[0] == 10)
assert(values[-1] == 30)

values[1] = 25
values.push(40)
let last_value = values.pop()
```

Indexing is zero-based. Negative indices count backward. Out-of-bounds access is an
error.

### Slices

The compact slice syntax is `start..end:step` inside brackets:

```ject
let values = [0, 1, 2, 3, 4, 5]
let middle = values[1..4]
let alternating = values[0..6:2]
```

Bounds are start-inclusive and end-exclusive. Named slice parts are also supported for
omitted bounds and clarity:

```ject
let tail = values[from:2]
let prefix = values[to:3]
let reversed = values[step:-1]
```

### Dictionaries

```ject
let user = {name: "Ada", active: true}
print user.name
print user["active"]

user.name = "Lin"
user["role"] = "admin"
```

Dot access is convenient for identifier-shaped string keys. Bracket access works for
computed keys.

### Comprehensions

```ject
let squares = [value * value for value in 0..6]
let evens = [value for value in 0..10 if value % 2 == 0]
```

### Unique arrays

Unique arrays preserve insertion order and omit duplicate values:

```ject
let tags = {|"rust", "ject", "rust"|}
assert(len(tags) == 2)
```

## 10. Method-call sugar

When a value has no real member with a requested name, this:

```ject
let clean = "  hello  ".trim().upper()
```

is shorthand for passing the receiver as the first argument:

```ject
let clean = upper(trim("  hello  "))
```

The same rule enables `items.map(transform)`, `items.push(value)`, and package helpers
written as ordinary functions. This is syntax convenience, not a separate class or
method dispatch system.

## 11. Pattern matching

`match` evaluates to the selected arm's result:

```ject
let status = 404
let message = match status
    200 -> "ok"
    400, 404 -> "client error"
    >= 500 -> "server error"
    _ -> "unknown"
end
```

Range patterns follow the same end-exclusive rule as every other range:

```ject
let grade = match score
    90..101 -> "A"
    80..90 -> "B"
    0..60 -> "F"
    _ -> "C"
end
```

An identifier pattern binds the matched value. A `when` guard can refine it:

```ject
let category = match amount
    value when value > 1000 -> "large"
    value when value > 0 -> "positive"
    _ -> "zero or negative"
end
```

The binding exists in the guard and arm body. A guarded binding pattern must be the
only pattern in its arm. Multiple literal patterns may share one guard. An unguarded
wildcard must be the final arm; a guarded wildcard may fall through.

Put the arrow at the end of a line to use a block body:

```ject
match command
    "save" ->
        write_file(path, content)
        print "saved"
    "quit", "exit" -> exit(0)
    _ -> print "unknown command"
end
```

## 12. Structs

Structs define a fixed set of named fields:

```ject
struct Point { x, y }

let origin = new Point { x: 0, y: 0 }
let cursor = new Point { x: 12, y: 8 }

cursor.x = cursor.x + 1
print cursor.x
```

Omitted fields receive `nil`. Unknown fields are rejected. Structs are useful when a
dictionary would be too loose but a full class system would be unnecessary.

Functions can provide method-like behavior through method-call sugar:

```ject
fn distance_from_origin(point)
    return sqrt(point.x * point.x + point.y * point.y)
end

print cursor.distance_from_origin()
```

## 13. Errors

Throw any value with `throw` and catch it with `try`/`catch`:

```ject
fn require_positive(value)
    if value <= 0
        throw "expected a positive value"
    end
    return value
end

try
    print require_positive(-1)
catch error
    print "validation failed: $error"
end
```

Runtime failures such as invalid operands, missing arguments, and out-of-range indices
use the same error channel and can be caught.

Use `assert(condition, message)` in tests and for internal invariants:

```ject
const result = 2 + 2
assert(result == 4, "arithmetic is working")
```

## 14. Modules

A module exposes only declarations marked `export`:

```ject
export const VERSION = "1.0.0"
export let requests = 0

export fn greet(name)
    requests++
    return "Hello, $name"
end
```

`export const` creates an immutable public binding. `export let` creates a mutable
module-local binding. The legacy `export name = value` shorthand remains accepted and
means `export let name = value`.

Exported bindings are ordinary declarations inside their defining module, so later
declarations and functions can use them. Module value declarations and top-level side
effects execute in source order. Functions are predeclared, which permits forward calls
and mutual recursion.

Import a module under a namespace:

```ject
import "./greeter" as greeter
print greeter.greet("Ada")
```

Import selected exports:

```ject
import {VERSION, greet} from "./greeter"
print VERSION
print greet("Lin")
```

An unaliased import places every export in the current scope:

```ject
import "./greeter"
print greet("Ada")
```

Aliases are preferred in application code because they make ownership obvious and
avoid collisions.

Names introduced by any import form are immutable bindings. Reassigning a local import
would not update the defining module, so Ject rejects it instead of creating misleading
split state. Mutable state exported with `export let` is changed by functions inside its
defining module.

Module paths resolve as follows:

| Form | Meaning |
| --- | --- |
| `"./helper"` | File relative to the importing source |
| `"../shared"` | Relative parent path |
| `"/full/path/tool"` | Absolute source path |
| `"~/tool"` | Home-relative source path |
| `"math"` | Declared package dependency or standard module |
| `"package/submodule"` | Source inside a dependency package |

Relative paths are never resolved from the shell's current directory; they are resolved
from the file containing the import.

## 15. The core and standard libraries

CorLib is always present. It includes common operations such as:

```text
type_of  to_int  to_float  to_string  to_bool
len  range  push  pop  contains  index_of
map  filter  reduce  sum  min  max
abs  sqrt  pow  sin  cos  tan  floor  ceil  round
print  input  assert
read_file  write_file  append_file
```

Larger groups live in source modules:

```ject
import "math" as math
import "string" as string
import "array" as array
import "collections" as collections
import "file" as file
import "system" as system
import "time" as time
import "color" as color
```

JGUI and JNUM are installable mixed packages, not hardcoded interpreter modules. Add
them as dependencies before importing them.

## 16. Packages and projects

Create packages through `ject`:

```sh
ject new player
ject new audio_tools --lib
ject new fast_dsp --native
```

An application normally uses `src/main.ject`; a library uses `src/lib.ject`. The
manifest can choose another entry path.

Add dependencies without manually editing TOML:

```sh
ject add helpers --path ../helpers
ject add colors --version '^2.1' --registry https://packages.example
ject add widgets --git https://github.com/example/widgets --branch main
ject install
```

`ject add` records the dependency and resolves the graph. `ject install` resolves all
entries, writes `Ject.lock`, verifies cached content, and builds native components.
`ject install --locked` refuses dependency drift and is the correct CI command.

See [Packages and native libraries](PACKAGES.md) for manifests, lockfiles, publishing,
mixed packages, native values, resources, callbacks, and ABI rules.

## 17. Diagnostics and editor support

Ject errors follow one stable shape:

```text
error[E2001]: undefined variable `naem`
 --> src/main.ject:3:7
  |
3 | print naem
  |       ^^^^ not found in this scope
  = help: did you mean `name`?
```

Codes identify the subsystem:

| Range | Area |
| --- | --- |
| `E10xx` | Lexing and parsing |
| `E20xx` | Static name, scope, and call checks |
| `E30xx` | Runtime values and operations |
| `E31xx` | Imports and modules |
| `E32xx` | Native calls and ABI failures |
| `E40xx` | Commands, manifests, packages, and files |
| `E41xx`–`E42xx` | Native builds and loading |
| `W20xx` | Non-fatal source warnings |

`ject check` is safe for editor and CI use because it never executes application code.
The built-in language server is started with `ject lsp` and provides diagnostics,
hover types, definitions, references, rename, signatures, symbols, highlights, and
completion from the same parser and semantic index as the CLI.

## 18. Command reference

| Command | Result |
| --- | --- |
| `ject` | Interactive REPL |
| `ject FILE` | Run a source file |
| `ject new NAME` | Create an application |
| `ject new NAME --lib` | Create a source library |
| `ject new NAME --native` | Create a mixed Ject/Rust library |
| `ject init ...` | Initialize the current directory |
| `ject run` | Run the current package |
| `ject check` | Parse and lint all package sources |
| `ject test` | Run every `tests/*.ject` script |
| `ject build [--release]` | Check source and build native artifacts |
| `ject add ...` | Add and resolve a dependency |
| `ject remove NAME` | Remove a dependency |
| `ject update [NAME]` | Refresh allowed dependency versions or revisions |
| `ject install [--locked]` | Resolve, verify, lock, and build dependencies |
| `ject publish --registry URL` | Publish the current immutable version |
| `ject tree` | Print the source-module graph |
| `ject lsp` | Run the language server over standard I/O |
| `ject --check FILE...` | Check standalone files |
| `ject --test FILE...` | Execute explicit test files |
| `ject --introspect` | Print native-kernel metadata as JSON |
| `ject --version` | Print the installed version |

Project commands search the current directory and its parents for `Ject.toml`.

## 19. Syntax reference

### Keywords

```text
let  const  fn  lambda  return
if  then  elseif  else  end
while  for  in  do  break  continue
match  when
true  false  nil
and  or  not
struct  new
try  catch  throw
import  export  from  as
print  to
```

`to` is reserved by named slice syntax.

### Operators

```text
+  -  *  /  %
==  !=  <  <=  >  >=
and  or  not  !  in
=  +=  -=  *=  /=  %=
++  --
..
```

### Delimiters

```text
( )     calls and grouping
[ ]     arrays, indexing, comprehensions, and slices
{ }     dictionaries and struct fields
{| |}   unique arrays
,       item separator
:       dictionary, range-step, and slice separator
.       member access and method-call sugar
->      short function and match-arm body
```

### Canonical block forms

```text
if CONDITION
    STATEMENTS
elseif CONDITION
    STATEMENTS
else
    STATEMENTS
end

while CONDITION
    STATEMENTS
end

for NAME in ITERABLE
    STATEMENTS
end

fn NAME(PARAMETERS)
    STATEMENTS
end

try
    STATEMENTS
catch NAME
    STATEMENTS
end
```

`then` and `do` are accepted in statement blocks for compatibility, but the forms above
are the recommended house style. The one-line conditional expression requires `then`.

## 20. Recommended style

- Indent with four spaces and put one statement on each line.
- Use `snake_case` for functions and local bindings, `PascalCase` for structs, and
  `UPPER_SNAKE_CASE` for constants.
- Default to `const`; choose `let` when reassignment or mutation through that binding is
  part of the design.
- Prefer `fn` over the `lambda` compatibility alias, and `not` over `!` in prose-like
  conditions.
- Use `name=value` for defaults, named arguments, and `print` options.
- Keep imports namespaced unless selective imports genuinely improve readability.
- Put validation and convenience behavior in a Ject facade; keep native APIs small.
- Run `ject check` while editing and `ject test` before committing.

## 21. Current boundaries

Ject is pre-1.0. It has local, Git, and registry dependencies; integrity-checked caches
and lockfiles; publishing; native resources; and synchronous native-to-Ject callbacks.
It does not sandbox native libraries. Capability controls, a WebAssembly provider, and
long-term 1.0 compatibility guarantees remain future work.
