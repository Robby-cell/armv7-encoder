use crate::{assembler::assemble, error::AsmError};

#[test]
fn undefined_label_error() {
    let code = "b nonexistent";
    let err = assemble(code).unwrap_err();
    assert!(matches!(err, AsmError::UndefinedLabel { .. }));
}

#[test]
fn invalid_immediate_error() {
    let code = "mov r0, #0x101";
    let err = assemble(code).unwrap_err();
    assert!(matches!(err, AsmError::ImmediateOutOfRange { .. }));
}

#[test]
fn syntax_error_missing_comma() {
    let code = "mov r0 r1";
    let err = assemble(code).unwrap_err();
    assert!(matches!(err, AsmError::ParseError { .. }));
}

#[test]
fn duplicate_label_error() {
    let code = "start: mov r0, #1\nstart: mov r1, #2";
    let err = assemble(code).unwrap_err();
    assert!(matches!(err, AsmError::ParseError { .. }));
}
