# Lox Interpreter (Made in Rust)

A small Lox interpreter written in Rust. Includes a scanner, parser, interpreter, and a tiny standard library (native functions).

# Use

Use cargo (Rusts package manager and compiler) to build and run

### Building

    Run `cargo build`
    Mostly just to ensure it can build and if you need an executable

### Running it

    Run `cargo run` for interactive terminal lox
    Run `cargo run -- <filename>` for file input (Also prints out file contents as if it was interactive, useful for testing)

### Logger

    The project has a small logger at `src/util/logger.rs`. The exported level enum is `LogLevel`:
    LogLevel: `Fatal`, `Error`, `Warn`, `Info`, `Debug`

    You can change the global logger level in the beginning of main
    Messages at the configured level and any higher-priority level are printed.
    Tests set the logger level so output stays readable during automated runs.

# Project structure

src dir has the source code and main.cpp
tests dir has the test code and test main.cpp
Files organized into sections such as types, core, and util

# Testing

### Test plan

    Separated into different directories and files based on what is being tested.
    Some of the low level stuff like the reader and scanner are tested but mostly focused on high level like parsing and interpreting.
    Cargo built in testing handles keeping track of successful and failing tests and printing them.
    Prints the name of successful tests and more info about failing tests.
    Each test function has many tests inside it but only prints one successful message, so even though it says only 30 something tests, each of those can have many inside.

### Run tests

    All tests: Run `cargo test` (Has extra useless output)
    Just my tests: Run `cargo test --test '*'` (Test output can be random)
    Best output: Run `RUST_TEST_THREADS=1 cargo test --test '*'` (Groups tests by type)

# Test output

### Automatic testing using `RUST_TEST_THREADS=1 cargo test --test '*'`

    $ RUST_TEST_THREADS=1 cargo test --test '*'
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
    Running tests\tests_main.rs (target\debug\deps\tests_main-91fbf3076ce7eeab.exe)

    running 34 tests
    test input::reader_test::reader_from_file ... ok
    test input::scanner_test::scanner_basic_tokens ... ok
    test input::scanner_test::scanner_nested_block_comments ... ok
    test input::scanner_test::scanner_unterminated_string ... ok
    test interpret::interpreter_test::interpret_assign_to_outer_from_inner_block ... ok
    test interpret::interpreter_test::interpret_assign_to_undefined_variable_reports_error_and_does_not_create ... ok
    test interpret::interpreter_test::interpret_assignment_expression_returns_value ... ok
    test interpret::interpreter_test::interpret_calling_non_callable_reports_error_but_preserves_value ... ok
    test interpret::interpreter_test::interpret_closure_makecounter ... ok
    test interpret::interpreter_test::interpret_comparisons_and_truthiness ... ok
    test interpret::interpreter_test::interpret_function_and_call ... ok
    test interpret::interpreter_test::interpret_function_arity_errors_prevent_assignment ... ok
    test interpret::interpreter_test::interpret_function_arity_success ... ok
    test interpret::interpreter_test::interpret_function_debug_string ... ok
    test interpret::interpreter_test::interpret_higher_order_function_call ... ok
    test interpret::interpreter_test::interpret_native_clock_smoke ... ok
    test interpret::interpreter_test::interpret_native_clock_wrong_arity ... ok
    test interpret::interpreter_test::interpret_recursion_factorial ... ok
    test interpret::interpreter_test::interpret_return_unwind_nested_blocks ... ok
    test interpret::interpreter_test::interpret_shadowing_inner_does_not_affect_outer ... ok
    test interpret::interpreter_test::interpret_string_number_concatenation ... ok
    test interpret::interpreter_test::interpret_value_equality_cases ... ok
    test interpret::interpreter_test::interpret_var_and_lookup ... ok
    test parser::parser_test::parser_assignment_and_block ... ok
    test parser::parser_test::parser_call_expression ... ok
    test parser::parser_test::parser_chained_calls_and_unary_grouping ... ok
    test parser::parser_test::parser_function_declaration ... ok
    test parser::parser_test::parser_function_no_params_empty_body ... ok
    test parser::parser_test::parser_if_else_and_block ... ok
    test parser::parser_test::parser_literal_expression ... ok
    test parser::parser_test::parser_logical_ops ... ok
    test parser::parser_test::parser_precedence_expression ... ok
    test parser::parser_test::parser_var_declaration ... ok
    test token::token_test::token_functionality ... ok

    test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

### From file test_input.txt

    $ cargo run -- test_input.txt
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
    Running `target\debug\lox_interpreter.exe test_input.txt`
    Logger initialized with minimum level: Info
    Logger level set to: Debug
    [Debug] - main: Start
    [Info] - main: Initializing configuration settings...
    [Info] - main: Input file provided: test_input.txt
    > // test_big.txt - broad interpreter integration test
    >
    > // Arithmetic and grouping
    > print 1 + 2 * 3;           // 1 + (2*3) = 7
    7
    > print (1 + 2) * 3;         // (1+2)*3 = 9
    9
    > print 3.0;                 // prints 3 (no trailing .0)
    3
    >
    > // Strings and concatenation
    > print "hello";
    hello
    > print "one " + "two";
    one two
    > print "one " + 1;        // string + number -> string concatenation
    one 1
    > print 1 + "two";         // number + string -> string concatenation
    1two
    >
    > // Variables and assignment-as-expression
    > var beverage = "espresso";
    > print beverage;
    espresso
    > var x = 10;
    > print x;
    10
    > var r = (x = 42);
    > print r;                   // should print 42
    42
    > print x;                   // x updated to 42
    42
    >
    > // Functions, calls and return
    > fun add(a, b) {
    >   return a + b;
    > }
    > print add(2, 3);
    5
    >
    > // Closures: factory that returns a counter function
    > fun makeCounter() {
    >   var i = 0;
    >   fun count() {
    >     i = i + 1;
    >     return i;
    >   }
    >   return count;
    > }
    > var c = makeCounter();
    > print c();
    1
    > print c();
    2
    >
    > // Recursion (factorial)
    > fun fact(n) {
    >   if (n <= 1) return 1;
    >   return n * fact(n - 1);
    > }
    > print fact(6); // 720
    720
    >
    > // Logical ops + short-circuiting side effects
    > var sc = false;
    > true or (sc = true);
    > print sc; // false
    false
    > false or (sc = true);
    > print sc; // true
    true
    >
    > // Scoping and shadowing
    > var vol = 11;
    > {
    >   var vol = 3 * 4 * 5;
    >   print vol; // inner
    > }
    60
    > print vol; // outer
    11
    >
    > // While and for loops
    > var i = 0;
    > while (i < 2) {
    >   print i;
    >   i = i + 1;
    > }
    0
    1
    >
    > for (var j = 0; j < 3; j = j + 1) print j;
    0
    1
    2
    >
    > // Native function usage
    > var t = clock();
    > print t;
    1762752501.709
    >
    > // Intentional runtime errors (should be reported but not crash)
    > print before_decl;
    Undefined variable 'before_decl'.
    [line 78]
    > "not a function"();
    Can only call functions and classes.
    [line 79]
    > clock(1); // wrong arity for native clock
    Expected 0 arguments but got 1.
    [line 80]
    >
    > // Block comment / nested comment test
    > /* outer /* nested */ still outer */
    > print "block comments ok";
    block comments ok
    >
    > // End
    [Debug] - scanner: Reached EOF or unrecognized character, setting EOF token

# Documentation

### Limitations

    Only went to chapter 10, did not get to 11, 12, and 13 because Rust was a pain to work in.
    Error reporting is not perfect. Line number can be wrong in certain situations, mostly fixed but still probably some edge cases. Does print in a very pretty way.
    Regular output can be weird at times. Like prints inside a block won't show in terminal until block is complete. That kind of stuff is mostly due to how my parser and interpreters work.
    Didn't do all of the challenges.
    Chapter 4 challenge: Did not add support for implicit semicolon
    Chapter 6 challenge: Did not add ternary operators
    Chapter 7 challenge: Did not add support for multi type comparisons but did add some multi type operators like adding a string and number to form a string.
    Chapter 8 challenge: Added REPL support for interaction. Did not make it an error to access uninitialized variable but as a design choice to let it be nil.
    Chapter 9: Did not add break or continue but would be somewhat easy to do.
    Chapter 10: Have not added anonymous functions yet but plan to soon.

### Design choices

    Not very cross type friendly. Can add strings to numbers but comparisons and stuff like that is an error.
    Most other stuff is meant to work rather than error such as accessing an uninitialized var giving nil.
    Tries its best not to crash on errors. Mostly works, fixed all the full crashes that I found.
    REPL based interpreter. Works well for terminal input. May not have clean output for file based input since it print each file line before interpreting it but I wanted easier testing and developing.
