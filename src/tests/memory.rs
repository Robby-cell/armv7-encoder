use crate::assembler::assemble;

#[test]
fn push_pop_range() {
    // push {r0-r5} -> r0 to r5 is 6 registers -> 0b00111111 = 0x3F
    // cond AL = 0xE. Push base = 0x092D0000.
    // 0xE92D003F -> [0x3F, 0x00, 0x2D, 0xE9]
    let bytes = assemble("push {r0-r5}").unwrap();
    assert_eq!(bytes, vec![0x3f, 0x00, 0x2d, 0xe9]);

    // pop {r1-r3, pc} -> r1, r2, r3, pc (15).
    // mask = 0b1000_0000_0000_1110 = 0x800E
    // Pop base = 0x08BD0000 -> 0xE8BD800E
    let bytes = assemble("pop {r1-r3, pc}").unwrap();
    assert_eq!(bytes, vec![0x0e, 0x80, 0xbd, 0xe8]);
}

#[test]
fn ldm_stm() {
    let code = "ldm r0, {r1-r3}";
    let bytes = assemble(code).unwrap();
    // cond AL=0xE. base=0x08900000. Rn=0. mask=r1,r2,r3=0x000E. -> 0xE890000E
    assert_eq!(bytes, vec![0x0e, 0x00, 0x90, 0xe8]);

    let code = "stm r0, {r2, r4-r5}";
    let bytes = assemble(code).unwrap();
    // base=0x08800000. mask=r2,r4,r5=0x0034. -> 0xE8800034
    assert_eq!(bytes, vec![0x34, 0x00, 0x80, 0xe8]);
}

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
