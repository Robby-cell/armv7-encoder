use crate::assembler::{AssemblerOptions, assemble_with_options};
use crate::prelude::Endian;
use crate::resolver::FnSymbolResolver;

#[test]
fn led_blink_example() {
    let source = r#"
_start: MOV R2, #0
loop:
    LDR R0, =led0 ; BL turn_on
    LDR R0, =led0 ; BL turn_off
    ADD R2, R2, #1 ; CMP R2, #0x3 ; BNE loop
    MOV R7, #1
    MOV R0, #0
    SVC 0
turn_on:
    PUSH {LR}
    MOV R1, #0x400
    STR R1, [R0]
    MOV R1, #0x20
    STR R1, [R0, #0x14]
    POP {PC}
turn_off:
    MOV R1, #0x400
    STR R1, [R0]
    MOV R1, #0x00
    STR R1, [R0, #0x14]
    BX LR
"#;

    let resolver = FnSymbolResolver::new(|name: &str| {
        if name == "led0" {
            Some(0x40000000)
        } else {
            None
        }
    });

    let options = AssemblerOptions {
        start_address: 0,
        endian: Endian::Little,
        symbol_resolver: Box::new(resolver),
    };

    let bytes = assemble_with_options(source, options).unwrap().bytes;

    let expected = [
        0, 32, 160, 227, // MOV R2, #0
        76, 0, 159, 229, // LDR R0, [PC, #76]  – first literal pool entry
        7, 0, 0, 235, // BL turn_on
        72, 0, 159, 229, // LDR R0, [PC, #72]  – second literal pool entry
        11, 0, 0, 235, // BL turn_off
        1, 32, 130, 226, // ADD R2, R2, #1
        3, 0, 82, 227, // CMP R2, #3
        248, 255, 255, 26, // BNE loop
        1, 112, 160, 227, // MOV R7, #1
        0, 0, 160, 227, // MOV R0, #0
        0, 0, 0, 239, // SVC 0
        // turn_on
        // NOTE: This is with GNU optimization applied
        4, 224, 45, 229, // PUSH {LR}
        1, 27, 160, 227, // MOV R1, #0x400
        0, 16, 128, 229, // STR R1, [R0]
        32, 16, 160, 227, // MOV R1, #0x20
        20, 16, 128, 229, // STR R1, [R0, #0x14]
        // NOTE: This also receives GNU optimization
        4, 240, 157, 228, // POP {PC}
        // turn_off
        1, 27, 160, 227, // MOV R1, #0x400
        0, 16, 128, 229, // STR R1, [R0]
        0, 16, 160, 227, // MOV R1, #0x00
        20, 16, 128, 229, // STR R1, [R0, #0x14]
        30, 255, 47, 225, // BX LR
        // literal pool
        0, 0, 0, 64, // .word 0x40000000 (led0)
        0, 0, 0, 64, // .word 0x40000000 (led0)
    ];

    assert_eq!(bytes, expected);
}
