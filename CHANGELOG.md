# Changelog

## 0.4.0

### Language features
- **Match v2**: relational patterns (`> 90`), inclusive range patterns (`0..12`), comma-separated
  multi-value patterns (`"quit", "exit", "q" -> ...`), expression and block arm bodies, a
  `value.match ... end` method-call form, and validation that `_` is at most one arm and always last.
- **Unified `fn`/`lambda` syntax**: anonymous functions can now be written as `fn(x) -> expr`
  (OCaml-style value function) or `fn(x) ... end` (block body, with the trailing bare expression
  implicitly returned -- no `return` needed). Named functions gained the same expression-body sugar:
  `fn square(x) -> x * x`. `lambda(x) -> expr` still works unchanged as a compatibility alias.
- **Method-call syntax sugar**: `obj.method(args)` now works uniformly. If `method` is a genuine
  member (a struct field, dictionary key, or module export -- e.g. `import "math" as m; m.log(x, b)`),
  that's called directly; otherwise it desugars to `method(obj, args)`, so `arr.map(f)`, `arr.push(x)`,
  `arr.len()` etc. all work as methods without every container type needing real methods.
- Real closures: environments are now shared (`Rc<RefCell<_>>`) instead of deep-cloned, so nested
  functions correctly close over outer variables, closures see mutations made after they were
  created (a counter closure persists state across calls), and cloning an environment for a closure
  is O(depth) instead of O(size) -- this also fixes what would otherwise have been exponential memory
  blowup when a scope declares many functions in sequence (e.g. loading a stdlib module).
- `throw`/`try`/`catch` confirmed and hardened (works with any value, not just strings; a thrown
  value crossing a function-call boundary is no longer wrapped in confusing "Error in function:"
  text or quote-escaped).

### Correctness fixes
- `and`/`or` now short-circuit (previously always evaluated both sides).
- `reduce()` without an initial value now seeds the accumulator from the first element instead of
  `nil`.
- `map`/`filter`/`reduce`/`any`/`all` now validate callback arity instead of silently misbehaving,
  and no longer double-evaluate the receiver when called via method syntax (`arr.map(f)`).
- `==` now does structural equality for `Dictionary`, `UniqueArray`, and `StructInstance` (previously
  always `false`).
- Empty `UniqueArray` is now falsy, matching empty `Array`.
- Fixed a systemic bug where a runtime error escaping a lambda/function call, a `for` loop body, a
  list comprehension, or a `catch` block left the interpreter's environment corrupted instead of
  restoring the caller's environment -- introduced one shared, safe call-invocation path
  (`invoke_callable`) that guarantees cleanup on every exit path.
- `range()` with a step now errors clearly on integer overflow instead of wrapping/panicking.
- `take()`/`drop()` with a negative count now clamp to zero instead of wrapping to a huge count.
- String interpolation (`"$name"`) and `to_string()` no longer wrap string values in literal quote
  characters.
- Relative imports (`./x`) now resolve against the *importing file's own directory* instead of the
  process's working directory -- previously any multi-file library with internal relative imports
  broke outside one specific directory.
- Circular imports now error clearly instead of recursing until the stack overflows; importing the
  same module twice reuses its already-executed state (a real singleton, like Python/JS modules)
  instead of re-parsing and re-running the file.
- `"error"` is no longer a reserved keyword -- it had no parser support anywhere and only blocked a
  very natural identifier/function name.
- `np.NAN` / `np.INF` (and `import {NAN} from "numpy"`) now work -- the wrapper file that used to
  define these aliases was unreachable dead code, since `numpy` resolves natively.
- List comprehensions and generators now support iterating over a string's characters, matching
  `for` loops.
- `fn(x) ... end` / `fn(x) -> expr` as a bare statement (not assigned to anything) now parses
  correctly instead of erroring "Expected function name" -- it was only being routed to
  named-function parsing regardless of whether a name followed.
- Whole-number floats (`10.0 / 2`) now print with an explicit `.0` (`5.0`) instead of looking
  identical to an integer (`5`) -- there was previously no way to tell a Float from an Integer
  apart in printed output.
- `log()` and `exp()` now accept integer arguments (coerced to float) instead of requiring exact
  float literals.

### New in the standard library
- `inf` / `nan` as first-class corlib constants (`f64::INFINITY` / `f64::NAN`).
- `color` module: ANSI foreground/background/style helpers, 24-bit RGB, and semantic
  `success`/`error`/`warning`/`info` shortcuts. Pure Ject, no native code.
- `table` module: aligned ASCII table rendering from arrays or from an array of dictionaries
  (headers auto-derived). Pure Ject, no native code.

### VS Code extension (0.3.0)
- Fixed autocomplete firing on blank lines/whitespace and matching finished expressions (e.g.
  suggesting the `E` constant after `e(...)` had already been typed and closed) -- suggestions now
  require an actual identifier prefix at the cursor and are filtered to it.
- Added `match`/`when` to keywords and syntax highlighting (previously missing entirely).
- Added `inf`/`nan` to constant highlighting and completions.
- Added the `color`/`table` modules to module completions.
- Refreshed hover docs: fixed an outdated `lambda` doc that claimed `fn(...) -> expr` didn't work,
  documented the new `fn` forms, added `match`/`when`/`throw`/`and`/`or`/`inf`/`nan` docs.
