use crate::assembler::{Encoder, assemble};

#[test]
fn default_entry_point() {
    let code = "mov r0, #0\nmov r1, #1";
    let result = assemble(code).unwrap();

    // If no _start or main label is present, it defaults to the start_address (0)
    assert_eq!(result.entry_point, 0);
}

#[test]
fn detects_start_entry_point() {
    let code = r#"
        nop
        nop
    _start:
        mov r0, #1
    "#;
    let result = assemble(code).unwrap();

    // Two NOPs take up 8 bytes, so _start is at 0x8
    assert_eq!(result.entry_point, 8);
}

#[test]
fn detects_main_entry_point_with_offset() {
    let code = r#"
        nop
    main:
        mov r0, #1
    "#;

    let result = Encoder::new().start_address(0x8000).assemble(code).unwrap();

    // 0x8000 base + 4 bytes for the first NOP
    assert_eq!(result.entry_point, 0x8004);
}

#[test]
fn tracks_label_offsets_correctly() {
    let code = r#"
        .text
    first:
        nop
    second:
        .space 10, 0
    third:
        .word 0x42
    "#;

    let result = assemble(code).unwrap();

    // 'first' is exactly at the start (0)
    assert_eq!(result.labels.get("first"), Some(&0));

    // 'second' is after the 4-byte NOP
    assert_eq!(result.labels.get("second"), Some(&4));

    // 'third' is after the 4-byte NOP and the 10-byte space padding (4 + 10 = 14)
    assert_eq!(result.labels.get("third"), Some(&14));
}

#[test]
fn accurate_instruction_counts() {
    let code = r#"
        .global main
        .text
    main:
        mov r0, r0
        nop
        .data
    data_label:
        .word 0xDEADBEEF
        .byte 0xFF
        .align 2
    "#;

    let result = assemble(code).unwrap();

    // The instruction counter should count emitted instructions & data chunks,
    // but IGNORE pure labels (main, data_label) and meta-directives (.global, .text, .data).
    //
    // Emitted tokens:
    // 1. mov
    // 2. nop
    // 3. .word
    // 4. .byte
    // 5. .align

    assert_eq!(result.instruction_count, 5);
}

#[test]
fn literal_pool_metadata_tracking() {
    let code = r#"
        ldr r0, =0xDEADBEEF
        ldr r1, =my_label
    my_label:
        mov r0, #0
    "#;

    let result = assemble(code).unwrap();

    // Emitted structure:
    // 0: ldr r0, [pc, offset]
    // 4: ldr r1, [pc, offset]
    // 8: mov r0, #0 (my_label)
    // 12: .word 0xDEADBEEF (__pool_0)
    // 16: .word my_label   (__pool_1)

    assert_eq!(result.labels.get("my_label"), Some(&8));

    // Verify that the assembler injected the internal pool labels exactly where they belong
    assert_eq!(result.labels.get("__pool_0"), Some(&12));
    assert_eq!(result.labels.get("__pool_1"), Some(&16));

    // 3 user instructions + 2 generated pool words
    assert_eq!(result.instruction_count, 5);
}
