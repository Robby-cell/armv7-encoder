use crate::assembler::assemble;

#[test]
fn branch_forward() {
    let code = "b target\nmov r0, #0\ntarget: mov r1, #0";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes[0..4], [0x00, 0x00, 0x00, 0xea]);
}

#[test]
fn label_with_colon() {
    let code = "start: mov r0, #5\nb start";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes[4..8], [0xfd, 0xff, 0xff, 0xea]);
}

#[test]
fn bl_label() {
    let code = "bl target\nmov r0, #0\ntarget: mov r1, #0";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes[0..4], [0x00, 0x00, 0x00, 0xeb]);
}

#[test]
fn bx_instruction() {
    let code = "bx lr";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes, vec![0x1e, 0xff, 0x2f, 0xe1]);
}

#[test]
fn forward_branch_offset() {
    let code = "b forward\nnop\nnop\nforward: mov r0, #0";
    let bytes = assemble(code).unwrap().bytes;
    assert_eq!(bytes[0..4], [0x01, 0x00, 0x00, 0xea]);
}
