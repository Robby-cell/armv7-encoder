use crate::assembler::assemble;

#[test]
fn ldr_immediate_offset() {
    let code = "ldr r0, [r1, #4]";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x04, 0x00, 0x91, 0xe5]);
}

#[test]
fn extra_load_store() {
    let code = "ldrh r0, [r1, #2]\nstrd r2, [r3, r4]";
    let bytes = assemble(code).unwrap();

    // ldrh r0, [r1, #2] -> 0xE1D100B2
    assert_eq!(&bytes[0..4], &[0xb2, 0x00, 0xd1, 0xe1]);

    // strd r2,[r3, r4] -> 0xE18320F4 (U=1, P=1, Register offset)
    assert_eq!(&bytes[4..8], &[0xf4, 0x20, 0x83, 0xe1]);
}

#[test]
fn str_immediate_offset() {
    let code = "str r0, [r1, #8]";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x08, 0x00, 0x81, 0xe5]);
}

#[test]
fn ldrb_register_offset() {
    let code = "ldrb r0, [r1, r2]";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0xd1, 0xe7]);
}

#[test]
fn strb_scaled_offset() {
    let code = "strb r0,[r1, r2, lsl #2]";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x01, 0xc1, 0xe7]);
}

#[test]
fn ldr_pc_relative() {
    let code = "ldr r0, [pc, #8]";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x08, 0x00, 0x9f, 0xe5]);
}

#[test]
fn push_pop() {
    let code = "push {r0, r1, lr}\npop {r0, r1, pc}";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes.len(), 8);
    assert_eq!(&bytes[0..4], &[0x03, 0x40, 0x2d, 0xe9]);
    assert_eq!(&bytes[4..8], &[0x03, 0x80, 0xbd, 0xe8]);
}
