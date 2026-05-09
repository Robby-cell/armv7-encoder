use crate::assembler::assemble;

#[test]
fn test_divide() {
    let code = "sdiv r0, r1, r2\nudiv r0, r1, r2";
    let bytes = assemble(code).unwrap();
    // SDIV R0, R1, R2 -> 0xE710F211
    assert_eq!(bytes[0..4], [0x11, 0xf2, 0x10, 0xe7]);
    // UDIV R0, R1, R2 -> 0xE730F211
    assert_eq!(bytes[4..8], [0x11, 0xf2, 0x30, 0xe7]);
}

#[test]
fn test_multiply_accumulate() {
    let code = "mla r0, r1, r2, r3\nmls r0, r1, r2, r3";
    let bytes = assemble(code).unwrap();
    // MLA R0, R1, R2, R3 -> 0xE0203291
    assert_eq!(bytes[0..4], [0x91, 0x32, 0x20, 0xe0]);
    // MLS R0, R1, R2, R3 -> 0xE0603291
    assert_eq!(bytes[4..8], [0x91, 0x32, 0x60, 0xe0]);
}
