use compiler::{Compiler, symbol_table::SymbolTable};
use core::panic;
use lexer::Lexer;
use object::{self, ObjectType};
use parser::{Parser, test_setup};
use std::{any::Any, collections::HashMap};
use vm::*;

struct VmTestCase {
    input: &'static str,
    expected: Box<dyn Any>,
}

fn run_vm_tests(tests: Vec<VmTestCase>) {
    for test in tests {
        let program = test_setup!(&test.input);
        let mut constants = Vec::new();
        let symbol_table = SymbolTable::new();
        let mut comp = Compiler::new(&mut constants, symbol_table);
        let mut globals = [const { ObjectType::NullObj }; GLOBAL_SIZE];

        comp.compile(program).unwrap();

        let mut vm = VM::new(comp, &mut globals);
        vm.run().unwrap();

        let stack_elem = vm.last_popped_stack_elem();
        test_expected_object(test.expected, &stack_elem);
    }
}

fn test_expected_object(expected: Box<dyn Any>, actual: &object::ObjectType) {
    if expected.is::<f64>() {
        return test_integer_object(*expected.downcast::<f64>().unwrap(), actual);
    }
    if expected.is::<bool>() {
        return test_bool_object(*expected.downcast::<bool>().unwrap(), actual);
    }
    if expected.is::<&'static str>() {
        return test_string_object(*expected.downcast::<&'static str>().unwrap(), actual);
    }
    if expected.is::<Vec<f64>>() {
        return test_array_object(*expected.downcast::<Vec<f64>>().unwrap(), actual);
    }
    if expected.is::<HashMap<u64, f64>>() {
        return test_hash_object(*expected.downcast::<HashMap<u64, f64>>().unwrap(), actual);
    }

    if expected.is::<ObjectType>() {
        return test_object_type(*expected.downcast::<ObjectType>().unwrap(), actual);
    }
    // Special null case, probably should be last
    if expected.is::<ObjectType>() {
        if *actual != NULL {
            panic!("object is not null: {expected:?}");
        }
        return;
    }

    todo!("type not yet ready for testing");
}

fn test_object_type(expected: ObjectType, actual: &ObjectType) {
    match expected {
        ObjectType::NullObj => assert_eq!(expected, *actual),
        ObjectType::ErrorObj(s) => {
            if let ObjectType::ErrorObj(actual_s) = actual {
                assert_eq!(s, *actual_s);
            }
        }
        _ => panic!("object type not handled: {:?}", expected),
    }
}

fn test_hash_object(expected: HashMap<u64, f64>, actual: &ObjectType) {
    match actual {
        ObjectType::HashObj(hash) => {
            assert_eq!(hash.len(), expected.len());
            for (expected_key, expected_value) in expected.iter() {
                test_integer_object(*expected_value, &hash.get(expected_key).unwrap().value);
            }
        }
        _ => panic!("expected a hash object, got: {:?}", actual),
    }
}

fn test_array_object(expected: Vec<f64>, actual: &ObjectType) {
    match actual {
        ObjectType::ArrayObj(objs) => {
            assert_eq!(expected.len(), objs.len());
            for (i, obj) in objs.iter().enumerate() {
                test_integer_object(expected[i], obj);
            }
        }
        _ => panic!("expected an array object, got: {:?}", actual),
    }
}

fn test_string_object(expected: &str, actual: &ObjectType) {
    match actual {
        ObjectType::StringObj(s) => assert_eq!(expected, *s),
        _ => panic!("expected a string object, got: {:?}", actual),
    }
}

fn test_bool_object(expected: bool, actual: &object::ObjectType) {
    match actual {
        ObjectType::BoolObj(b) => assert_eq!(expected, *b),
        _ => panic!("expected only bool objects, got: {:?}", actual),
    }
}

fn test_integer_object(expected: f64, actual: &object::ObjectType) {
    match actual {
        ObjectType::IntegerObj(x) => assert_eq!(expected, *x),
        _ => panic!("expected only integer objects, got: {:?}", actual),
    }
}

macro_rules! vm_test_case {
    ($input:expr, $expected:expr) => {{
        VmTestCase {
            input: $input,
            expected: Box::new($expected),
        }
    }};
}

#[test]
fn test_integer_arithmetic() {
    run_vm_tests(vec![
        vm_test_case!("1", 1.0f64),
        vm_test_case!("2", 2.0f64),
        vm_test_case!("1 + 2", 3.0f64),
        vm_test_case!("1 - 2", -1.0f64),
        vm_test_case!("1 * 2", 2.0f64),
        vm_test_case!("2 / 1", 2.0f64),
        vm_test_case!("5 * (2 + 10)", 60.0f64),
        vm_test_case!("-5", -5.0f64),
        vm_test_case!("-10", -10.0f64),
        vm_test_case!("-50 + 100 + -50", 0.0f64),
        vm_test_case!("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50.0f64),
    ]);
}

#[test]
fn test_bool_expressions() {
    run_vm_tests(vec![
        vm_test_case!("true", true),
        vm_test_case!("false", false),
        vm_test_case!("1 < 2", true),
        vm_test_case!("1 > 2", false),
        vm_test_case!("1 < 1", false),
        vm_test_case!("1 > 1", false),
        vm_test_case!("1 == 1", true),
        vm_test_case!("1 != 1", false),
        vm_test_case!("1 == 2", false),
        vm_test_case!("1 != 2", true),
        vm_test_case!("true == true", true),
        vm_test_case!("false == false", true),
        vm_test_case!("true == false", false),
        vm_test_case!("true != false", true),
        vm_test_case!("false != true", true),
        vm_test_case!("(1 < 2) == true", true),
        vm_test_case!("(1 < 2) == false", false),
        vm_test_case!("(1 > 2) == true", false),
        vm_test_case!("!true", false),
        vm_test_case!("!false", true),
        vm_test_case!("!5", false),
        vm_test_case!("!!true", true),
        vm_test_case!("!!false", false),
        vm_test_case!("!!5", true),
        vm_test_case!("!(if (false) { 5; })", true),
    ]);
}

#[test]
fn test_conditional() {
    run_vm_tests(vec![
        vm_test_case!("if (true) { 10 }", 10.0f64),
        vm_test_case!("if (true) { 10 } else { 20 }", 10.0f64),
        vm_test_case!("if (false) { 10 } else { 20 }", 20.0f64),
        vm_test_case!("if (1) { 10 }", 10.0f64),
        vm_test_case!("if (1 < 2) { 10 }", 10.0f64),
        vm_test_case!("if (1 < 2) { 10 } else { 20 }", 10.0f64),
        vm_test_case!("if (1 > 2) { 10 } else { 20 }", 20.0f64),
        vm_test_case!("if (1 > 2) { 10 }", NULL),
        vm_test_case!("if (false) { 10 }", NULL),
        vm_test_case!("if ((if (false) { 10 })) { 10 } else { 20 }", 20.0f64),
    ]);
}

#[test]
fn test_global_let_statements() {
    run_vm_tests(vec![
        vm_test_case!("let one = 1; one", 1.0f64),
        vm_test_case!("let one = 1; let two = 2; one + two", 3.0f64),
        vm_test_case!("let one = 1; let two = one + one; one + two", 3.0f64),
    ]);
}

#[test]
fn test_string_expressions() {
    run_vm_tests(vec![
        vm_test_case!(r#""monkey""#, "monkey"),
        vm_test_case!(r#""mon" + "key""#, "monkey"),
        vm_test_case!(r#""mon" + "key" + "banana""#, "monkeybanana"),
    ]);
}

#[test]
fn test_array_literals() {
    run_vm_tests(vec![
        vm_test_case!("[]", Vec::<f64>::new()),
        vm_test_case!("[1, 2, 3]", vec![1.0f64, 2.0f64, 3.0f64]),
        vm_test_case!("[1 + 2, 3 - 4, 5 * 6]", vec![3.0f64, -1.0f64, 30.0f64]),
    ]);
}

#[test]
fn test_hash_literals() {
    let mut hash1 = HashMap::new();
    hash1.insert(ObjectType::IntegerObj(1.0).hash().unwrap(), 2.0f64);
    hash1.insert(ObjectType::IntegerObj(2.0).hash().unwrap(), 3.0f64);

    let mut hash2 = HashMap::new();
    hash2.insert(ObjectType::IntegerObj(2.0).hash().unwrap(), 4.0f64);
    hash2.insert(ObjectType::IntegerObj(6.0).hash().unwrap(), 16.0f64);

    run_vm_tests(vec![
        vm_test_case!("{1: 2, 2: 3}", hash1),
        vm_test_case!("{1 + 1: 2 * 2, 3 + 3: 4 * 4}", hash2),
    ]);
}

#[test]
fn test_index_expressions() {
    run_vm_tests(vec![
        vm_test_case!("[1, 2, 3][1]", 2f64),
        vm_test_case!("[1,2,3][0 + 2]", 3f64),
        vm_test_case!("[[1,1,1]][0][0]", 1f64),
        vm_test_case!("[][0]", NULL),
        vm_test_case!("[1,2,3][99]", NULL),
        vm_test_case!("[1][-1]", NULL),
        vm_test_case!("{1: 1, 2: 2}[1]", 1f64),
        vm_test_case!("{1: 1, 2: 2}[2]", 2f64),
        vm_test_case!("{1: 1}[0]", NULL),
        vm_test_case!("{}[0]", NULL),
    ]);
}

#[test]
fn test_function_calls() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let fivePlusTen = fn() { 5 + 10 };
                    fivePlusTen();
                "#,
            15f64
        ),
        vm_test_case!(
            r#"
                    let one = fn() { 1; };
                    let two = fn() { 2; };
                    one() + two()
                "#,
            3f64
        ),
        vm_test_case!(
            r#"
                    let a = fn() { 1 };
                    let b = fn() { a() + 1 };
                    let c = fn() { b() + 1 };
                    c();
                "#,
            3f64
        ),
    ]);
}

#[test]
fn test_functions_with_return_statements() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let earlyExit = fn() { return 99; 100; };
                    earlyExit();
                "#,
            99f64
        ),
        vm_test_case!(
            r#"
                    let earlyExit = fn() { return 99; 100; };
                    earlyExit();
                "#,
            99f64
        ),
    ]);
}

#[test]
fn test_functions_without_return_values() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let noReturn = fn() { };
                    noReturn();
                "#,
            NULL
        ),
        vm_test_case!(
            r#"
                    let noReturn = fn() { };
                    let noReturnTwo = fn() { };
                    noReturn();
                    noReturnTwo();
                "#,
            NULL
        ),
    ]);
}

#[test]
fn test_first_class_functions() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let returnsOne = fn() { 1; };
                    let returnsOneReturner = fn() { returnsOne; };
                    returnsOneReturner()();
                "#,
            1f64
        ),
        vm_test_case!(
            r#"
                    let returnsOneReturner = fn() {
                        let returnsOne = fn() {
                            1;
                        };
                        returnsOne;
                    };
                    returnsOneReturner()();
                "#,
            1f64
        ),
    ]);
}

#[test]
fn test_calling_funcs_with_bindings() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let one = fn() { let one = 1; one };
                    one();
                "#,
            1f64
        ),
        vm_test_case!(
            r#"
                    let oneAndTwo = fn() { let one = 1; let two = 2; one + two };
                    oneAndTwo();
                "#,
            3f64
        ),
        vm_test_case!(
            r#"
                    let oneAndTwo = fn() { let one = 1; let two = 2; one + two };
                    let threeAndFour = fn() { let three = 3; let four = 4; three + four };
                    oneAndTwo() + threeAndFour();
                "#,
            10f64
        ),
        vm_test_case!(
            r#"
                    let firstFoobar = fn() { let foobar = 50; foobar; };
                    let secondFoobar = fn() { let foobar = 100; foobar; };
                    firstFoobar() + secondFoobar();
                "#,
            150f64
        ),
        vm_test_case!(
            r#"
                    let globalSeed = 50;
                    let minusOne = fn() { 
                        let num = 1;
                        globalSeed - num;
                    };
                    let minusTwo = fn() { 
                        let num = 2;
                        globalSeed - num;
                    };
                    minusOne() + minusTwo();
                "#,
            97f64
        ),
    ]);
}

#[test]
fn test_calling_funcs_with_args_and_bindings() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let identity = fn(a) { a; };
                    identity(4);
                "#,
            4f64
        ),
        vm_test_case!(
            r#"
                    let sum = fn(a, b) { a + b; };
                    sum(1, 2);
                "#,
            3f64
        ),
        vm_test_case!(
            r#"
                    let sum = fn(a, b) {
                        let c = a + b;
                        c;
                    };

                    sum(1, 2);
                "#,
            3f64
        ),
        vm_test_case!(
            r#"
                    let sum = fn(a, b) {
                        let c = a + b;
                        c;
                    };

                    sum(1, 2) + sum(3, 4);
                "#,
            10f64
        ),
        vm_test_case!(
            r#"
                    let sum = fn(a, b) {
                        let c = a + b;
                        c;
                    };

                    let outer = fn() {
                        sum(1, 2) + sum(3, 4);
                    };

                    outer();
                "#,
            10f64
        ),
        vm_test_case!(
            r#"
                    let globalNum = 10;

                    let sum = fn(a, b) {
                        let c = a + b;
                        c + globalNum;
                    };

                    let outer = fn() {
                        sum(1, 2) + sum(3, 4) + globalNum;
                    };

                    outer() + globalNum;
                "#,
            50f64
        ),
    ]);
}

#[test]
fn call_functions_with_wrong_arguments() {
    let tests = vec![
        vm_test_case!(
            r#"
                    fn() { 1; }(1);
                "#,
            "wrong number of arguments: want=0; got=1"
        ),
        vm_test_case!(
            r#"
                    fn(a) { a; }();
                "#,
            "wrong number of arguments: want=1; got=0"
        ),
        vm_test_case!(
            r#"
                    fn(a, b) { a + b; }(1);
                "#,
            "wrong number of arguments: want=2; got=1"
        ),
    ];

    for test in tests {
        let program = test_setup!(&test.input);
        let mut constants = Vec::new();
        let symbol_table = SymbolTable::new();
        let mut comp = Compiler::new(&mut constants, symbol_table);
        let mut globals = [const { ObjectType::NullObj }; GLOBAL_SIZE];

        comp.compile(program).unwrap();

        let mut vm = VM::new(comp, &mut globals);

        let expected_error_msg = test.expected.downcast::<&'static str>().unwrap();

        match vm.run() {
            Ok(_) => panic!("Expected vm to fail"),
            Err(msg) => assert_eq!(msg.to_string(), *expected_error_msg),
        }
    }
}

#[test]
fn test_builtin_funcs() {
    run_vm_tests(vec![
        vm_test_case!("len(\"\")", 0f64),
        vm_test_case!("len(\"four\")", 4f64),
        vm_test_case!("len(\"hello world\")", 11f64),
        vm_test_case!(
            "len(1)",
            ObjectType::ErrorObj("argument to `len` not supported, got INTEGER".into())
        ),
        vm_test_case!(
            "len(\"one\", \"two\")",
            ObjectType::ErrorObj("wrong number of arguments. got=2, want=1".into())
        ),
        vm_test_case!("len([1,2,3])", 3f64),
        vm_test_case!("len([])", 0f64),
        vm_test_case!("puts(\"hello\", \"world\")", NULL),
        vm_test_case!("first([1,2,3])", 1f64),
        vm_test_case!("first([])", NULL),
        vm_test_case!(
            "first(1)",
            ObjectType::ErrorObj("argument to `first` must be ARRAY, got INTEGER".into())
        ),
        vm_test_case!("last([1,2,3])", 3f64),
        vm_test_case!("last([])", NULL),
        vm_test_case!(
            "last(1)",
            ObjectType::ErrorObj("argument to `last` must be ARRAY, got INTEGER".into())
        ),
        vm_test_case!("rest([1,2,3])", vec![2f64, 3f64]),
        vm_test_case!("rest([])", NULL),
        vm_test_case!("push([], 1)", vec![1f64]),
        vm_test_case!(
            "push(1, 1)",
            ObjectType::ErrorObj("argument to `push` must be ARRAY, got INTEGER".into())
        ),
    ]);
}

#[test]
fn test_closures() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let newClosure = fn(a) {
                        fn() { a };
                    };
                    let closure = newClosure(99);
                    closure();
                "#,
            99.0
        ),
        vm_test_case!(
            r#"
                    let newAdder = fn(a, b) {
                        fn(c) { a + b + c };
                    };
                    let adder = newAdder(1,2);
                    adder(8);
                "#,
            11.0
        ),
        vm_test_case!(
            r#"
                    let newAdder = fn(a, b) {
                        let c = a + b;
                        fn(d) { c + d };
                    };
                    let adder = newAdder(1, 2);
                    adder(8);
                "#,
            11.0
        ),
        vm_test_case!(
            r#"
                    let newAdderOuter = fn(a, b) {
                        let c = a + b;
                        fn(d) {
                            let e = d + c;
                            fn(f) { e + f; };
                        };
                    };
                    let newAdderInner = newAdderOuter(1, 2)
                    let adder = newAdderInner(3);
                    adder(8);
                "#,
            14.0
        ),
        vm_test_case!(
            r#"
                    let a = 1;
                    let newAdderOuter = fn(b) {
                        fn(c) {
                            fn(d) { a + b + c + d };
                        };
                    };
                    let newAdderInner = newAdderOuter(2)
                    let adder = newAdderInner(3);
                    adder(8);
            "#,
            14.0
        ),
        vm_test_case!(
            r#"
                    let newClosure = fn(a, b) {
                        let one = fn() { a; };
                        let two = fn() { b; };
                        fn() { one() + two(); };
                    };
                    let closure = newClosure(9, 90);
                    closure();
                "#,
            99.0
        ),
    ]);
}

#[test]
fn test_recursive_functions() {
    run_vm_tests(vec![
        vm_test_case!(
            r#"
                    let countDown = fn(x) {
                        if (x == 0) {
                            return 0;
                        } else {
                            countDown(x - 1);
                        }
                    };
                    countDown(1);
                "#,
            0.0
        ),
        vm_test_case!(
            r#"
                    let countDown = fn(x) {
                        if (x == 0) {
                            return 0;
                        } else {
                            countDown(x - 1);
                        }
                    };
                    let wrapper = fn() {
                        countDown(1);
                    };
                    wrapper();
                "#,
            0.0
        ),
        vm_test_case!(
            r#"
                    let wrapper = fn() {
                        let countDown = fn(x) {
                            if (x == 0) {
                                return 0;
                            } else {
                                countDown(x - 1);
                            }
                        };
                        countDown(1);
                    };
                    wrapper();
                "#,
            0.0
        ),
    ]);
}

#[test]
fn test_fibonacci() {
    run_vm_tests(vec![vm_test_case!(
        r#"
            let fibonacci = fn(x) {
                if (x == 0) {
                    return 0;
                }
                if (x == 1) {
                    return 1;
                }
                return fibonacci(x - 1) + fibonacci(x - 2);
            };
            fibonacci(15);
        "#,
        610.0
    )]);
}

#[test]
fn test_or() {
    run_vm_tests(vec![
        vm_test_case!("true || true", true),
        vm_test_case!("true || false", true),
        vm_test_case!("false || false", false),
        vm_test_case!("1 || 0;", true),
        vm_test_case!("0 || 0;", false),
        vm_test_case!("0 || false;", false),
        vm_test_case!("false || \"this is a truthy sentence\";", true),
        vm_test_case!(
            r#"
                    let x = 24;
                    if (x == 24 || false) {
                        1;
                    } else {
                        48;
                    }
                "#,
            1.0
        ),
    ]);
}

#[test]
fn test_and() {
    run_vm_tests(vec![
        vm_test_case!("true && true", true),
        vm_test_case!("false && true", false),
        vm_test_case!("true && 0", false),
        vm_test_case!("45 && 1", true),
        vm_test_case!("0 && 5", false),
    ]);
}

#[test]
fn test_mutation() {
    run_vm_tests(vec![
        vm_test_case!("let a = 5; a = a + 5; a;", 10.0),
        vm_test_case!("let a = 5; let b = 6; a = b + 1; a;", 7.0),
    ]);
}

#[test]
fn test_postfix() {
    run_vm_tests(vec![
        vm_test_case!("let a = 5; a++;", 6.0),
        vm_test_case!("let a = 5; a--;", 4.0),
    ]);
}
