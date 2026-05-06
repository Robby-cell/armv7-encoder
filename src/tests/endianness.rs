use crate::assembler::{AssemblerOptions, Endian, assemble_with_options};

#[test]
fn little_endian_mov() {
    let code = "mov r0, #1";
    let options = AssemblerOptions {
        endian: Endian::Little,
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, vec![0x01, 0x00, 0xa0, 0xe3]);
}

#[test]
fn big_endian_mov() {
    let code = "mov r0, #1";
    let options = AssemblerOptions {
        endian: Endian::Big,
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, vec![0xe3, 0xa0, 0x00, 0x01]);
}

#[test]
fn little_endian_word() {
    let code = ".word 0x12345678";
    let options = AssemblerOptions {
        endian: Endian::Little,
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, vec![0x78, 0x56, 0x34, 0x12]);
}

#[test]
fn big_endian_word() {
    let code = ".word 0x12345678";
    let options = AssemblerOptions {
        endian: Endian::Big,
        ..AssemblerOptions::default()
    };
    let bytes = assemble_with_options(code, options).unwrap();
    assert_eq!(bytes, vec![0x12, 0x34, 0x56, 0x78]);
}
