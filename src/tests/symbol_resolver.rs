use crate::assembler::{AssemblerOptions, assemble, assemble_with_options};
use crate::error::AsmError;
use crate::resolver::{FnSymbolResolver, HashMapSymbolResolver};
use crate::symbols;

// Fn‑based resolver tests

#[test]
fn external_symbol_branch() {
    let code = "b external_func";
    let options = AssemblerOptions {
        symbol_resolver: Box::new(FnSymbolResolver::new(|name: &str| {
            if name == "external_func" {
                Some(0x1000)
            } else {
                None
            }
        })),
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, [0xfe, 0x03, 0x00, 0xea]);
}

#[test]
fn external_symbol_word() {
    let code = ".word ext_data";
    let options = AssemblerOptions {
        symbol_resolver: Box::new(FnSymbolResolver::new(|name: &str| {
            if name == "ext_data" {
                Some(0xDEADBEEF)
            } else {
                None
            }
        })),
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, [0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn external_symbol_bl() {
    let code = "bl external_call";
    let options = AssemblerOptions {
        symbol_resolver: Box::new(FnSymbolResolver::new(|name: &str| {
            if name == "external_call" {
                Some(0x2000)
            } else {
                None
            }
        })),
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, [0xfe, 0x07, 0x00, 0xeb]);
}

#[test]
fn local_label_overrides_resolver() {
    let code = "b internal\ninternal: mov r0, r0";
    let options = AssemblerOptions {
        symbol_resolver: Box::new(FnSymbolResolver::new(|name: &str| {
            if name == "internal" {
                Some(0x9999)
            } else {
                None
            }
        })),
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes[0..4], [0xff, 0xff, 0xff, 0xea]);
}

#[test]
fn resolver_returns_none_causes_error() {
    let code = "b missing";
    let options = AssemblerOptions {
        symbol_resolver: Box::new(FnSymbolResolver::new(|_: &str| None)),
        ..AssemblerOptions::default()
    };
    let err = assemble_with_options(code, options).unwrap_err();
    assert!(matches!(err, AsmError::UndefinedLabel { .. }));
}

#[test]
fn resolver_not_called_for_defined_label() {
    let code = "defined: b defined";
    let options = AssemblerOptions {
        symbol_resolver: Box::new(FnSymbolResolver::new(|_name: &str| -> Option<u32> {
            panic!("resolver should not be called for local label");
        })),
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, [0xfe, 0xff, 0xff, 0xea]);
}

// HashMap‑based resolver tests

#[test]
fn hashmap_resolver_works() {
    let resolver = symbols!(("puts", 0x1000), ("printf", 0x2000));

    let options = AssemblerOptions {
        symbol_resolver: Box::new(resolver),
        ..AssemblerOptions::default()
    };
    let code = "bl puts";
    let bytes = assemble_with_options(code, options).unwrap();
    // PC = 0+8 = 8, target = 0x1000, offset = (0x1000-8)/4 = 0x3FE
    assert_eq!(bytes, [0xfe, 0x03, 0x00, 0xeb]);
}

#[test]
fn hashmap_missing_symbol_errors() {
    let resolver = symbols!();

    let options = AssemblerOptions {
        symbol_resolver: Box::new(resolver),
        ..AssemblerOptions::default()
    };
    let err = assemble_with_options("b unknown", options).unwrap_err();
    assert!(matches!(err, AsmError::UndefinedLabel { .. }));
}

// No‑op resolver

#[test]
fn noop_resolver_errors() {
    // Default uses NoSymbolResolver, so we can just use default options
    let code = "b missing";
    let err = assemble(code).unwrap_err();
    assert!(matches!(err, AsmError::UndefinedLabel { .. }));
}
