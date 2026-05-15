use crate::assembler::assemble;

#[test]
fn byte_directive() {
    let code = ".byte 0x12\n.byte 0x34\n.align 2\n.word 0xdeadbeef";
    let bytes = assemble(code).unwrap().bytes;

    // 0x12, 0x34, then 2 bytes padding to align to 4
    // followed by 0xDEADBEEF (little endian)
    assert_eq!(bytes, [0x12, 0x34, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn padding_arithmetic() {
    let code = r#"
    _start:
        mov r0, #0x10
        .space 0x10 - (. - _start)
        .word 0xDEADBEEF
    "#;
    let bytes = assemble(code).unwrap().bytes;
    // 1 instruction = 4 bytes.
    // space = 0x10 - (4 - 0) = 16 - 4 = 12 bytes of padding.
    // Total bytes before word = 16.
    // Word = 4 bytes.
    // Total bytes = 20.
    assert_eq!(bytes.len(), 20);
    assert_eq!(&bytes[16..20], &[0xEF, 0xBE, 0xAD, 0xDE]);
}

#[test]
fn memory_spacing_and_shorts() {
    let code = r#"
        .byte 0x01
        .space 2, 0xFF
        .short 0x1234
        .int 0xDEADBEEF
    "#;
    let bytes = assemble(code).unwrap().bytes;

    // Total size: 1 byte + 2 bytes space + 2 bytes short + 4 bytes int = 9 bytes
    assert_eq!(bytes.len(), 9);

    // 1 byte
    assert_eq!(bytes[0], 0x01);
    // .space 2 bytes padded with 0xFF
    assert_eq!(&bytes[1..3], &[0xFF, 0xFF]);
    // .short 0x1234 (little endian)
    assert_eq!(&bytes[3..5], &[0x34, 0x12]);
    // .int / .long / .word 0xDEADBEEF (little endian)
    assert_eq!(&bytes[5..9], &[0xEF, 0xBE, 0xAD, 0xDE]);
}

#[test]
fn word_directive() {
    let code = ".word 0xdeadbeef";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes, [0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn word_label() {
    let code = "start: .word start";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes, [0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn mixed_data_and_code() {
    let code = ".word 0xdeadbeef\nmov r0, r0\n.word 0xcafebabe";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes.len(), 12);
    assert_eq!(&bytes[0..4], &[0xef, 0xbe, 0xad, 0xde]);
    assert_eq!(&bytes[4..8], &[0x00, 0x00, 0xa0, 0xe1]);
    assert_eq!(&bytes[8..12], &[0xbe, 0xba, 0xfe, 0xca]);
}
