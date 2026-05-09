# ARMv7 Assembler in Rust

A pure-Rust library that translates ARMv7 (A32) assembly text into machine code.  
**No native dependencies** – compiles natively and to WebAssembly via `wasm-pack`.

<!-- [![Crates.io](https://img.shields.io/crates/v/armv7-encoder?style=flat-square)](https://crates.io/crates/armv7-encoder) -->
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## Features

- **Full data‑processing instructions**  
  MOV, MVN, ADD, SUB, RSB, ADC, SBC, RSC, AND, ORR, EOR, BIC  
  CMP, CMN, TST, TEQ (with automatic `S` flag)

- **Flexible shifter operands**  
  Immediate (`#123`, `#0xFF`), register, shifted register by immediate or register  
  (LSL, LSR, ASR, ROR, RRX)

- **Load / Store**  
  LDR, STR, LDRB, STRB – immediate, register, and scaled register offsets  
  PC‑relative `LDR Rd, label` (assembles to `LDR Rd, [PC, #offset]`)  
  Pseudo‑instruction `LDR Rd, =immediate` (literal pool)

- **Multiply**  
  MUL

- **Stack operations**  
  PUSH, POP (optimised for single register)

- **Branch & Exchange**  
  B, BL (condition codes supported), BX

- **System / Hint instructions**  
  NOP, BKPT, WFI, WFE, YIELD, SEV

- **Condition codes & `S` flag**  
  e.g. `addeq`, `movs`, `bleq`

- **Directives**  
  `.word`, `.float`, `.align`, `.ascii`, `.asciz`, `.global`, `.text`, `.data`

- **Configurable endianness**  
  `Endian::Little` (default) or `Endian::Big`

- **Custom start address**  
  Pass `start_address` in `AssemblerOptions`

- **Symbol resolver trait**  
  Built‑in resolvers: `NoSymbolResolver`, `HashMapSymbolResolver`, `FnSymbolResolver`  
  Easy to implement your own via the `SymbolResolver` trait

- **Parser features**  
  `;` separates multiple statements on one line  
  `@` starts a line comment  
  Labels with `:`  
  Robust error reporting with line/column numbers

- **Pure Rust** – compiles to `wasm32-unknown-unknown` for WebAssembly

---

## Quick Start

Add the library to your `Cargo.toml`:

```toml
[dependencies]
armv7-encoder = { git = "https://github.com/Robby-cell/armv7-encoder.git", tag = "0.3.3" }
```

Then assemble some code:

```rust
use armv7_encoder::prelude::*;

fn main() {
    let source = r#"
        .text
        start:  mov r0, #42
                add r0, r0, #8
                cmp r0, #50
                bne start
                bx  lr
    "#;

    let machine_code = assemble(source).unwrap();
    for byte in machine_code {
        print!("{:02X} ", byte);
    }
}
```

---

## Assembler Options

You can control the output and symbol resolution via `AssemblerOptions`:

```rust
let options = AssemblerOptions {
    start_address: 0x8000,
    endian: Endian::Big,
    symbol_resolver: Box::new(my_resolver),
};
let bytes = assemble_with_options(source, options).unwrap();
```

- `start_address` – base address for labels and PC‑relative calculations.
- `endian` – `Endian::Little` (default) or `Endian::Big`.
- `symbol_resolver` – implements `SymbolResolver` trait (see below).

---

## Symbol Resolvers

The library provides several built‑in resolvers, and you can implement the `SymbolResolver` trait yourself.

### NoSymbolResolver (default)
Doesn’t resolve any external symbol.

### HashMapSymbolResolver
Use the `symbols!` macro for quick construction:

```rust
use armv7_encoder::resolver::HashMapSymbolResolver;
use armv7_encoder::symbols;

let resolver = symbols!(
    ("puts",   0x1234),
    ("printf", 0x5678),
);
```

Manual insertion:

```rust
let mut resolver = HashMapSymbolResolver::new();
resolver.insert("my_func", 0x12345678);
```

### FnSymbolResolver
Wraps any closure:

```rust
let resolver = FnSymbolResolver::new(|name: &str| {
    if name == "external_call" { Some(0x2000) } else { None }
});
```

### Custom Resolver
Implement `SymbolResolver` for your own type:

```rust
struct MyResolver;
impl SymbolResolver for MyResolver {
    fn resolve(&self, name: &str) -> Option<u32> {
        // … lookup logic
    }
}
```

---

## Supported Instructions & Syntax

| Category             | Mnemonics                                                       |
|----------------------|-----------------------------------------------------------------|
| Data processing      | MOV, MVN, ADD, SUB, RSB, ADC, SBC, RSC, AND, ORR, EOR, BIC      |
| Comparisons          | CMP, CMN, TST, TEQ                                              |
| Load/Store           | LDR, STR, LDRB, STRB                                            |
| Multiply             | MUL                                                             |
| Stack                | PUSH, POP                                                       |
| Branch               | B, BL, BX                                                       |
| Supervisor           | SVC                                                             |
| System               | SVC, NOP, BKPT, WFI, WFE, YIELD, SEV                            |

### Operand forms

- **Immediate**: `#123`, `#0xFF`, optional `#`
- **Register**: `r0`, `sp`, `lr`, `pc`, `a1`–`a4`, `v1`–`v8`, `sb`, `sl`, `fp`, `ip`
- **Shifts**: `r1, lsl #2`, `r2, ror r3`, `r4, rrx`
- **Memory**: `[r0]`, `[r1, #4]`, `[r2, r3]`, `[r4, r5, lsl #2]`
- **PC‑relative load**: `ldr r0, label` (encodes as `ldr r0, [pc, #offset]`)
- **Pseudo load**: `ldr r0, =0x12345678` (generates a literal pool)

---

## Directives

| Directive     | Example                     |
|---------------|-----------------------------|
| `.text`       | switch to code section      |
| `.data`       | switch to data section      |
| `.word`       | `.word 0xdeadbeef`          |
| `.float`      | `.float 3.14`               |
| `.align`      | `.align 2`  (align to 4 bytes) |
| `.ascii`      | `.ascii "hello"`            |
| `.asciz`      | `.asciz "hello"`  (null terminated) |
| `.global`     | `.global main`              |
| `.label`      | `.label`                    |

---

## Error Handling

All errors implement `Display + Error`. Example match:

```rust
match assemble(source) {
    Err(AsmError::ParseError { line, col, message }) => …,
    Err(AsmError::UndefinedLabel { line, label }) => …,
    Err(AsmError::ImmediateOutOfRange { line, value }) => …,
    …
}
```

---

## WebAssembly Usage

Build with `wasm-pack`:

```bash
wasm-pack build --target web --features wasm
```

Then use in JavaScript:

```javascript
import {Endian, Encoder} from './pkg/armv7_encoder.js'

const e = new Encoder(0, Endian.Little, (s) => {
    switch (s) {
        case "foo": return 0x1234
        default: return null
    }
})
const bytes = e.assemble("ldr r0, =foo; add r0, r0, #42; svc 0")
console.log(bytes)
```

---

## Testing

Run the full test suite:

```bash
cargo test
```

Extensive unit tests cover instruction encoding, directives, symbol resolution, error cases, and endianness.

---

## License

This project is licensed under the MIT license. See [LICENSE](LICENSE) for details.

---

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss what you would like to change.
