use crate::assembler::assemble;

#[test]
fn test_hint_instructions() {
    let code = "wfi\nwfe\nyield\nsev\nnop";
    let bytes = assemble(code).unwrap();
    // WFI: 0xE320F003
    assert_eq!(&bytes[0..4], &[0x03, 0xf0, 0x20, 0xe3]);
    // WFE: 0xE320F002
    assert_eq!(&bytes[4..8], &[0x02, 0xf0, 0x20, 0xe3]);
    // YIELD: 0xE320F001
    assert_eq!(&bytes[8..12], &[0x01, 0xf0, 0x20, 0xe3]);
    // SEV: 0xE320F004
    assert_eq!(&bytes[12..16], &[0x04, 0xf0, 0x20, 0xe3]);
    // NOP: 0xE320F000
    assert_eq!(&bytes[16..20], &[0x00, 0xf0, 0x20, 0xe3]);
}

#[test]
fn test_conditional_hints() {
    let code = "wfieq\nsevne";
    let bytes = assemble(code).unwrap();
    // WFIEQ: 0x0320F003
    assert_eq!(&bytes[0..4], &[0x03, 0xf0, 0x20, 0x03]);
    // SEVNE: 0x1320F004
    assert_eq!(&bytes[4..8], &[0x04, 0xf0, 0x20, 0x13]);
}

#[test]
fn test_bkpt() {
    let code = "bkpt #0x1234\nbkpt 42";
    let bytes = assemble(code).unwrap();
    // bkpt 0x1234: 0xE1212374
    assert_eq!(&bytes[0..4], &[0x74, 0x23, 0x21, 0xe1]);
    // bkpt 42 (0x2A): 0xE120027A
    assert_eq!(&bytes[4..8], &[0x7a, 0x02, 0x20, 0xe1]);
}
