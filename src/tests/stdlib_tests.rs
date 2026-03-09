#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::interpreter::Interpreter;

    fn run(input: &str) -> Result<(), String> {
        let mut lexer = Lexer::new(input);
        let located_tokens = lexer.tokenize_with_positions();
        let tokens: Vec<_> = located_tokens.into_iter().map(|lt| lt.token).collect();
        let mut parser = Parser::new_simple(tokens);
        let statements = parser.parse().map_err(|e| e.message)?;
        
        let mut interpreter = Interpreter::new();
        interpreter.interpret(&statements).map_err(|e| e.message)?;
        
        Ok(())
    }

    // ========== Math Functions Tests ==========

    #[test]
    fn test_abs_integer() {
        let result = run("assert(abs(-42) == 42, \"abs(-42) should be 42\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_abs_float() {
        let result = run("assert(abs(-3.14) == 3.14, \"abs(-3.14) should be 3.14\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_abs_positive() {
        let result = run("assert(abs(42) == 42, \"abs(42) should be 42\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sqrt() {
        let result = run("assert(sqrt(16) == 4.0, \"sqrt(16) should be 4.0\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sqrt_perfect_square() {
        let result = run("assert(sqrt(25) == 5.0, \"sqrt(25) should be 5.0\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pow() {
        let result = run("assert(pow(2, 8) == 256.0, \"pow(2, 8) should be 256.0\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pow_float_exponent() {
        let result = run("assert(pow(4, 0.5) == 2.0, \"pow(4, 0.5) should be 2.0\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sin() {
        let result = run("assert(sin(0) == 0.0, \"sin(0) should be 0.0\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_cos() {
        let result = run("assert(cos(0) == 1.0, \"cos(0) should be 1.0\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_tan() {
        let result = run("assert(tan(0) == 0.0, \"tan(0) should be 0.0\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_floor() {
        let result = run("assert(floor(3.7) == 3, \"floor(3.7) should be 3\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ceil() {
        let result = run("assert(ceil(3.2) == 4, \"ceil(3.2) should be 4\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_round() {
        let result = run("assert(round(3.5) == 4, \"round(3.5) should be 4\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_min() {
        let result = run("assert(min(5, 3, 8, 1, 9) == 1, \"min should be 1\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_max() {
        let result = run("assert(max(5, 3, 8, 1, 9) == 9, \"max should be 9\")");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sum() {
        let result = run("assert(sum([1, 2, 3, 4, 5]) == 15, \"sum should be 15\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sum_empty_array() {
        let result = run("assert(sum([]) == 0, \"sum of empty array should be 0\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_len_array() {
        let result = run("assert(len([1, 2, 3, 4, 5]) == 5, \"len should be 5\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_len_string() {
        let result = run("assert(len(\"hello\") == 5, \"len should be 5\"");
        assert!(result.is_ok());
    }

    #[test]
    fn test_len_empty() {
        let result = run("assert(len(\"\") == 0, \"len should be 0\"");
        assert!(result.is_ok());
    }

    // ========== Array Functions Tests ==========

    #[test]
    fn test_push() {
        let result = run(r#"
let arr = [1, 2, 3]
arr = push(arr, 4)
assert(len(arr) == 4, "len should be 4")
assert(arr[3] == 4, "last element should be 4")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pop() {
        let result = run(r#"
let arr = [1, 2, 3]
let last = pop(arr)
assert(last == 3, "popped value should be 3")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_map() {
        let result = run(r#"
let numbers = [1, 2, 3, 4, 5]
let doubled = map(numbers, lambda(n) -> n * 2)
assert(len(doubled) == 5, "length should be 5")
assert(doubled[0] == 2, "first element should be 2")
assert(doubled[4] == 10, "last element should be 10")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_filter() {
        let result = run(r#"
let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let evens = filter(numbers, lambda(n) -> n % 2 == 0)
assert(len(evens) == 5, "should have 5 even numbers")
assert(evens[0] == 2, "first even should be 2")
assert(evens[4] == 10, "last even should be 10")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reduce() {
        let result = run(r#"
let numbers = [1, 2, 3, 4, 5]
let sum = reduce(numbers, lambda(acc, n) -> acc + n, 0)
assert(sum == 15, "sum should be 15")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reduce_with_initial() {
        let result = run(r#"
let numbers = [1, 2, 3]
let product = reduce(numbers, lambda(acc, n) -> acc * n, 1)
assert(product == 6, "product should be 6")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_numbers() {
        let result = run(r#"
let numbers = [5, 2, 8, 1, 9, 3]
let sorted = sort(numbers)
assert(sorted[0] == 1, "first should be 1")
assert(sorted[5] == 9, "last should be 9")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reverse() {
        let result = run(r#"
let arr = [1, 2, 3, 4, 5]
let reversed = reverse(arr)
assert(reversed[0] == 5, "first should be 5")
assert(reversed[4] == 1, "last should be 1")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unique() {
        let result = run(r#"
let arr = [1, 2, 2, 3, 3, 3, 4]
let unique_arr = unique(arr)
assert(len(unique_arr) == 4, "should have 4 unique elements")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains() {
        let result = run(r#"
let arr = [1, 2, 3, 4, 5]
assert(contains(arr, 3) == true, "should contain 3")
assert(contains(arr, 10) == false, "should not contain 10")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_index_of() {
        let result = run(r#"
let arr = [10, 20, 30, 40, 50]
assert(index_of(arr, 30) == 2, "index of 30 should be 2")
assert(index_of(arr, 100) == -1, "index of missing should be -1")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_slice() {
        let result = run(r#"
let arr = [1, 2, 3, 4, 5]
let sliced = slice(arr, 1, 4)
assert(len(sliced) == 3, "slice length should be 3")
assert(sliced[0] == 2, "first element should be 2")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_first() {
        let result = run(r#"
let arr = [10, 20, 30]
assert(first(arr) == 10, "first should be 10")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_last() {
        let result = run(r#"
let arr = [10, 20, 30]
assert(last(arr) == 30), "last should be 30")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_concat() {
        let result = run(r#"
let arr1 = [1, 2, 3]
let arr2 = [4, 5, 6]
let result = concat(arr1, arr2)
assert(len(result) == 6, "length should be 6")
assert(result[3] == 4, "fourth element should be 4")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_flatten() {
        let result = run(r#"
let nested = [[1, 2], [3, 4], [5, 6]]
let flat = flatten(nested)
assert(len(flat) == 6, "length should be 6")
assert(flat[0] == 1, "first should be 1")
assert(flat[5] == 6, "last should be 6")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enumerate() {
        let result = run(r#"
let arr = ["a", "b", "c"]
let enumerated = enumerate(arr)
assert(len(enumerated) == 3, "length should be 3")
assert(enumerated[0][0] == 0, "first index should be 0")
assert(enumerated[0][1] == "a", "first value should be a")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_any() {
        let result = run(r#"
let numbers = [1, 2, 3, 4, 5]
assert(any(numbers, lambda(n) -> n > 3) == true, "some numbers are > 3")
assert(any(numbers, lambda(n) -> n > 10) == false, "no numbers are > 10")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_all() {
        let result = run(r#"
let numbers = [2, 4, 6, 8]
assert(all(numbers, lambda(n) -> n % 2 == 0) == true, "all are even")
assert(all(numbers, lambda(n) -> n > 5) == false, "not all are > 5")
"#);
        assert!(result.is_ok());
    }

    // ========== String Functions Tests ==========

    #[test]
    fn test_upper() {
        let result = run(r#"
assert(upper("hello") == "HELLO", "should be uppercase")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lower() {
        let result = run(r#"
assert(lower("HELLO") == "hello", "should be lowercase")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trim() {
        let result = run(r#"
assert(trim("  hello  ") == "hello", "should trim whitespace")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_split() {
        let result = run(r#"
let csv = "apple,banana,cherry"
let fruits = split(csv, ",")
assert(len(fruits) == 3, "should have 3 fruits")
assert(fruits[0] == "apple", "first should be apple")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_join() {
        let result = run(r#"
let fruits = ["apple", "banana", "cherry"]
let joined = join(fruits, ", ")
assert(joined == "apple, banana, cherry", "should join with separator")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_replace() {
        let result = run(r#"
let text = "Hello Python"
let fixed = replace(text, "Python", "Ject")
assert(fixed == "Hello Ject", "should replace Python with Ject")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_starts_with() {
        let result = run(r#"
assert(starts_with("Hello World", "Hello") == true, "should start with Hello")
assert(starts_with("Hello World", "World") == false, "should not start with World")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_ends_with() {
        let result = run(r#"
assert(ends_with("Hello World", "World") == true, "should end with World")
assert(ends_with("Hello World", "Hello") == false, "should not end with Hello")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repeat() {
        let result = run(r#"
assert(repeat("ab", 3) == "ababab", "should repeat 3 times")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reverse_str() {
        let result = run(r#"
assert(reverse_str("hello") == "olleh", "should reverse string")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_contains_str() {
        let result = run(r#"
assert(contains_str("Hello World", "World") == true, "should contain World")
assert(contains_str("Hello World", "Python") == false, "should not contain Python")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_char_at() {
        let result = run(r#"
assert(char_at("hello", 0) == "h", "first char should be h")
assert(char_at("hello", 4) == "o", "last char should be o")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_substring() {
        let result = run(r#"
assert(substring("hello", 1, 4) == "ell", "should extract substring")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_capitalize() {
        let result = run(r#"
assert(capitalize("hello") == "Hello", "should capitalize first letter")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_empty() {
        let result = run(r#"
assert(is_empty("") == true, "empty string should be empty")
assert(is_empty("hello") == false, "non-empty string should not be empty")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_numeric() {
        let result = run(r#"
assert(is_numeric("123") == true, "numeric string should be numeric")
assert(is_numeric("abc") == false, "alpha string should not be numeric")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_alpha() {
        let result = run(r#"
assert(is_alpha("abc") == true, "alpha string should be alpha")
assert(is_alpha("123") == false, "numeric string should not be alpha")
"#);
        assert!(result.is_ok());
    }

    // ========== Type Conversion Tests ==========

    #[test]
    fn test_to_int() {
        let result = run(r#"
assert(to_int("42") == 42, "should convert string to int")
assert(to_int(3.7) == 3, "should convert float to int")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_float() {
        let result = run(r#"
assert(to_float("3.14") == 3.14, "should convert string to float")
assert(to_float(42) == 42.0, "should convert int to float")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_string() {
        let result = run(r#"
assert(to_string(42) == "42", "should convert int to string")
assert(to_string(3.14) == "3.14", "should convert float to string")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_bool() {
        let result = run(r#"
assert(to_bool(true) == true, "true should be true")
assert(to_bool(false) == false, "false should be false")
assert(to_bool(0) == false, "0 should be false")
assert(to_bool(1) == true, "1 should be true")
"#);
        assert!(result.is_ok());
    }

    // ========== Base Conversion Tests ==========

    #[test]
    fn test_to_binary() {
        let result = run(r#"
assert(to_binary(42) == "101010", "42 in binary")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_binary() {
        let result = run(r#"
assert(from_binary("101010") == 42), "binary to decimal")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_hex() {
        let result = run(r#"
assert(to_hex(255) == "ff", "255 in hex")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_hex() {
        let result = run(r#"
assert(from_hex("ff") == 255), "hex to decimal")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_octal() {
        let result = run(r#"
assert(to_octal(64) == "100", "64 in octal")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_octal() {
        let result = run(r#"
assert(from_octal("100") == 64), "octal to decimal")
"#);
        assert!(result.is_ok());
    }

    // ========== Utility Functions Tests ==========

    #[test]
    fn test_type_of() {
        let result = run(r#"
assert(type_of(42) == "number", "int type")
assert(type_of(3.14) == "number", "float type")
assert(type_of("hello") == "string", "string type")
assert(type_of(true) == "boolean", "bool type")
assert(type_of(nil) == "nil", "nil type")
assert(type_of([1, 2, 3]) == "array", "array type")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_range_function() {
        let result = run(r#"
let r = range(1, 6)
assert(len(r) == 5, "range length should be 5")
assert(r[0] == 1, "first should be 1")
assert(r[4] == 5, "last should be 5")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_random() {
        let result = run(r#"
let r = random()
assert(r >= 0 and r < 1, "random should be in [0, 1)")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_random_int() {
        let result = run(r#"
let r = random_int(1, 10)
assert(r >= 1 and r <= 10, "random_int should be in range")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_true() {
        let result = run(r#"
assert(true == true, "true should equal true")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_false() {
        let result = run(r#"
assert(2 + 2 == 4, "math should work")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_assert_failure() {
        let result = run(r#"
assert(2 + 2 == 5, "this should fail")
"#);
        assert!(result.is_err());
    }

    // ========== Constants Tests ==========

    #[test]
    fn test_pi_constant() {
        let result = run(r#"
assert(PI > 3.14 and PI < 3.15, "PI should be approximately 3.14")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_e_constant() {
        let result = run(r#"
assert(E > 2.71 and E < 2.72, "E should be approximately 2.71")
"#);
        assert!(result.is_ok());
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_nested_function_calls() {
        let result = run(r#"
let result = sqrt(pow(abs(-16), 2))
assert(result == 16.0, "nested functions should work")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chained_string_methods() {
        let result = run(r#"
let text = "  HELLO  "
let result = lower(trim(text))
assert(result == "hello", "chained methods should work")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_of_arrays() {
        let result = run(r#"
let matrix = [[1, 2], [3, 4], [5, 6]]
let flat = flatten(matrix)
assert(sum(flat) == 21), "sum of flattened matrix should be 21")
"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_higher_order_functions() {
        let result = run(r#"
let numbers = [1, 2, 3, 4, 5]
let result = reduce(map(filter(numbers, lambda(n) -> n % 2 != 0), lambda(n) -> n * n), lambda(a, b) -> a + b, 0)
assert(result == 35), "sum of squares of odd numbers should be 35")
"#);
        assert!(result.is_ok());
    }
}
