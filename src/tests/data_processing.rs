use crate::assembler::assemble;

#[test]
fn mov_immediate() {
    let code = "mov r0, #0x56";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x56, 0x00, 0xa0, 0xe3]);
}

#[test]
fn add_register() {
    let code = "add r1, r2, r3";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x03, 0x10, 0x82, 0xe0]);
}

#[test]
fn sub_with_shift() {
    let code = "sub r4, r5, r6, lsl #2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x06, 0x41, 0x45, 0xe0]);
}

#[test]
fn cmp_register_shift() {
    let code = "cmp r0, r1, ror r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes.len(), 4);
}

#[test]
fn multiple_statements_semicolon() {
    let code = "mov r0, #1 ; mov r1, #2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes.len(), 8);
    assert_eq!(&bytes[0..4], &[0x01, 0x00, 0xa0, 0xe3]);
    assert_eq!(&bytes[4..8], &[0x02, 0x10, 0xa0, 0xe3]);
}

#[test]
fn comment_at() {
    let code = "mov r0, #3 @ load 3 into r0";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x03, 0x00, 0xa0, 0xe3]);
}

#[test]
fn mov_register() {
    let code = "mov r0, r1";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x01, 0x00, 0xa0, 0xe1]);
}

#[test]
fn mvn_immediate() {
    let code = "mvn r0, #0xFF";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0xff, 0x00, 0xe0, 0xe3]);
}

#[test]
fn add_immediate() {
    let code = "add r0, r1, #42";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x2a, 0x00, 0x81, 0xe2]);
}

#[test]
fn sub_immediate() {
    let code = "sub r0, r1, #1";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x01, 0x00, 0x41, 0xe2]);
}

#[test]
fn rsb_immediate() {
    let code = "rsb r0, r1, #5";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x05, 0x00, 0x61, 0xe2]);
}

#[test]
fn adc_register() {
    let code = "adc r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0xa1, 0xe0]);
}

#[test]
fn sbc_register() {
    let code = "sbc r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0xc1, 0xe0]);
}

#[test]
fn rsc_register() {
    let code = "rsc r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0xe1, 0xe0]);
}

#[test]
fn and_immediate() {
    let code = "and r0, r1, #0xF0";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0xf0, 0x00, 0x01, 0xe2]);
}

#[test]
fn orr_register() {
    let code = "orr r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0x81, 0xe1]);
}

#[test]
fn eor_register() {
    let code = "eor r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0x21, 0xe0]);
}

#[test]
fn bic_register() {
    let code = "bic r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0xc1, 0xe1]);
}

#[test]
fn cmp_immediate() {
    let code = "cmp r0, #0";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x00, 0x00, 0x50, 0xe3]);
}

#[test]
fn cmn_immediate() {
    let code = "cmn r0, #5";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x05, 0x00, 0x70, 0xe3]);
}

#[test]
fn tst_immediate() {
    let code = "tst r0, #0xFF";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0xff, 0x00, 0x10, 0xe3]);
}

#[test]
fn teq_immediate() {
    let code = "teq r0, #0xFF";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0xff, 0x00, 0x30, 0xe3]);
}

// S flag tests
#[test]
fn movs_flag() {
    let code = "movs r0, #1";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x01, 0x00, 0xb0, 0xe3]);
}

#[test]
fn adds_flag() {
    let code = "adds r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0x91, 0xe0]);
}

#[test]
fn subs_flag() {
    let code = "subs r0, r1, #1";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x01, 0x00, 0x51, 0xe2]);
}

// Condition code tests
#[test]
fn mov_eq() {
    let code = "moveq r0, #1";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x01, 0x00, 0xa0, 0x03]);
}

#[test]
fn add_ne() {
    let code = "addne r0, r1, r2";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x02, 0x00, 0x81, 0x10]);
}

#[test]
fn sub_gt() {
    let code = "subgt r0, r1, #1";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x01, 0x00, 0x41, 0xc2]);
}

// Shift type tests
#[test]
fn lsl_shift() {
    let code = "add r0, r1, r2, lsl #3";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x82, 0x01, 0x81, 0xe0]);
}

#[test]
fn lsr_shift() {
    let code = "add r0, r1, r2, lsr #4";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x22, 0x02, 0x81, 0xe0]);
}

#[test]
fn asr_shift() {
    let code = "add r0, r1, r2, asr #5";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0xc2, 0x02, 0x81, 0xe0]);
}

#[test]
fn ror_shift() {
    let code = "add r0, r1, r2, ror #6";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x62, 0x03, 0x81, 0xe0]);
}

#[test]
fn rrx_shift() {
    let code = "add r0, r1, r2, rrx";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x62, 0x00, 0x81, 0xe0]);
}

#[test]
fn shift_register() {
    let code = "add r0, r1, r2, lsl r3";
    let bytes = assemble(code).unwrap();
    assert_eq!(bytes, vec![0x12, 0x03, 0x81, 0xe0]);
}
