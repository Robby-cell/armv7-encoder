use crate::assembler::assemble;

#[test]
fn byte_directive() {
    let code = ".byte 0x12\n.byte 0x34\n.align 2\n.word 0xdeadbeef";
    let bytes = assemble(code).unwrap();

    // 0x12, 0x34, then 2 bytes padding to align to 4
    // followed by 0xDEADBEEF (little endian)
    assert_eq!(bytes, [0x12, 0x34, 0x00, 0x00, 0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn word_directive() {
    let code = ".word 0xdeadbeef";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, [0xef, 0xbe, 0xad, 0xde]);
}

#[test]
fn word_label() {
    let code = "start: .word start";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, [0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn mixed_data_and_code() {
    let code = ".word 0xdeadbeef\nmov r0, r0\n.word 0xcafebabe";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes.len(), 12);
    assert_eq!(&bytes[0..4], &[0xef, 0xbe, 0xad, 0xde]);
    assert_eq!(&bytes[4..8], &[0x00, 0x00, 0xa0, 0xe1]);
    assert_eq!(&bytes[8..12], &[0xbe, 0xba, 0xfe, 0xca]);
}
