use crate::encoder::*;
use crate::error::AsmError;
use crate::parser::{Expr, Mnemonic, Operand, Statement, parse_statement};
use crate::resolver::SymbolResolver;
use std::collections::HashMap;

/// Endianness selection with distinct discriminants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    /// Big‑endian mode – numeric value.
    Big = 1 << 30,

    /// Little‑endian mode – numeric value 0.
    Little = 0,
}

#[derive(Debug, Clone)]
pub struct AssembleResult {
    pub bytes: Vec<u8>,
    pub entry_point: u64,
    /// Maps a label/function name to its exact physical byte address (IP)
    pub labels: HashMap<String, u64>,
    /// Total number of physical instructions and data directives emitted
    pub instruction_count: usize,
}

struct AssemblerOptions<'a> {
    start_address: u32,
    endian: Endian,
    symbol_resolver: Option<&'a mut dyn SymbolResolver>,
}

pub struct Encoder<'a> {
    options: AssemblerOptions<'a>,
}

impl<'a> Default for Encoder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Encoder<'a> {
    pub fn new() -> Self {
        Self {
            options: AssemblerOptions {
                start_address: 0,
                endian: Endian::Little,
                symbol_resolver: None,
            },
        }
    }

    pub fn start_address(mut self, addr: u32) -> Self {
        self.options.start_address = addr;
        self
    }

    pub fn endian(mut self, endian: Endian) -> Self {
        self.options.endian = endian;
        self
    }

    pub fn with_resolver(mut self, resolver: &'a mut dyn SymbolResolver) -> Self {
        self.options.symbol_resolver = Some(resolver);
        self
    }

    pub fn assemble(&self, source: &str) -> Result<AssembleResult, AsmError> {
        let mut statements = Vec::new();
        for (line_idx, raw_line) in source.lines().enumerate() {
            let line_num = line_idx + 1;
            let trimmed = process_line(raw_line);
            if trimmed.is_empty() {
                continue;
            }
            for part in trimmed.split(';') {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                let mut stmt = parse_statement(part, line_num)?;
                stmt.line = line_num;
                statements.push(stmt);
            }
        }

        let mut pool_entries = Vec::new();
        let mut pool_id = 0;

        for stmt in &mut statements {
            if matches!(stmt.mnemonic, Mnemonic::Ldr)
                && let Some(Operand::PseudoLoadExpr(expr)) = stmt.operands.get(1).cloned()
            {
                let pool_lbl = format!("__pool_{}", pool_id);
                pool_id += 1;
                pool_entries.push(Statement {
                    label: Some(pool_lbl.clone()),
                    mnemonic: Mnemonic::Word,
                    condition: Condition::Al,
                    s_flag: false,
                    operands: vec![Operand::Expr(expr)],
                    line: stmt.line,
                });
                stmt.operands[1] = Operand::Label(pool_lbl);
            }
        }

        statements.extend(pool_entries);

        let mut current_addr = self.options.start_address;
        let mut label_map: HashMap<String, u32> = HashMap::new();

        for stmt in &mut statements {
            if let Some(ref label) = stmt.label {
                if label_map.contains_key(label) {
                    return Err(AsmError::ParseError {
                        line: stmt.line,
                        col: 0,
                        message: format!("duplicate label '{}'", label),
                    });
                }
                label_map.insert(label.clone(), current_addr);
            }
            let size = match stmt.mnemonic {
                Mnemonic::LabelOnly
                | Mnemonic::Global
                | Mnemonic::Text
                | Mnemonic::Data
                | Mnemonic::It => 0,
                Mnemonic::Byte => 1,
                Mnemonic::Short => 2,
                Mnemonic::Space => {
                    if let Operand::Expr(expr) = &stmt.operands[0] {
                        match eval_expr(expr, &label_map, &self.options, current_addr) {
                            Ok(size) => {
                                if size < 0 {
                                    return Err(AsmError::ParseError {
                                        line: stmt.line,
                                        col: 0,
                                        message: "negative space size".into(),
                                    });
                                }
                                size as u32
                            }
                            Err(e) => {
                                return Err(AsmError::ParseError {
                                    line: stmt.line,
                                    col: 0,
                                    message: e,
                                });
                            }
                        }
                    } else {
                        0
                    }
                }
                Mnemonic::Float => 4,
                Mnemonic::Align => {
                    if let Operand::Expr(expr) = &stmt.operands[0] {
                        match eval_expr(expr, &label_map, &self.options, current_addr) {
                            Ok(val) => {
                                let align_bytes = 1 << val;
                                if current_addr.is_multiple_of(align_bytes) {
                                    0
                                } else {
                                    align_bytes - (current_addr % align_bytes)
                                }
                            }
                            Err(e) => {
                                return Err(AsmError::ParseError {
                                    line: stmt.line,
                                    col: 0,
                                    message: e,
                                });
                            }
                        }
                    } else {
                        0
                    }
                }
                Mnemonic::Ascii => {
                    if let Operand::StringBytes(ref b) = stmt.operands[0] {
                        b.len() as u32
                    } else {
                        0
                    }
                }
                Mnemonic::Asciz => {
                    if let Operand::StringBytes(ref b) = stmt.operands[0] {
                        (b.len() + 1) as u32
                    } else {
                        0
                    }
                }
                _ => 4,
            };
            current_addr += size;
        }

        let mut bytes = Vec::new();
        let mut instruction_count = 0;
        current_addr = self.options.start_address;

        for stmt in &statements {
            match &stmt.mnemonic {
                Mnemonic::LabelOnly
                | Mnemonic::Global
                | Mnemonic::Text
                | Mnemonic::Data
                | Mnemonic::It => {
                    continue;
                }
                Mnemonic::Byte => {
                    if let Operand::Expr(expr) = &stmt.operands[0] {
                        let val = eval_expr(expr, &label_map, &self.options, current_addr)
                            .unwrap_or(0) as u8;
                        bytes.push(val);
                        current_addr += 1;
                        instruction_count += 1;
                    }
                    continue;
                }
                Mnemonic::Short => {
                    if let Operand::Expr(expr) = &stmt.operands[0] {
                        let val = eval_expr(expr, &label_map, &self.options, current_addr)
                            .unwrap_or(0) as u16;
                        match self.options.endian {
                            Endian::Big => bytes.extend_from_slice(&val.to_be_bytes()),
                            Endian::Little => bytes.extend_from_slice(&val.to_le_bytes()),
                        }
                        current_addr += 2;
                        instruction_count += 1;
                    }
                    continue;
                }
                Mnemonic::Space => {
                    if let (Operand::Expr(size_expr), Operand::Expr(fill_expr)) =
                        (&stmt.operands[0], &stmt.operands[1])
                    {
                        let size = eval_expr(size_expr, &label_map, &self.options, current_addr)
                            .unwrap_or(0) as usize;
                        let fill = eval_expr(fill_expr, &label_map, &self.options, current_addr)
                            .unwrap_or(0) as u8;
                        bytes.resize(bytes.len() + size, fill);
                        current_addr += size as u32;
                        instruction_count += 1;
                    }
                    continue;
                }
                Mnemonic::Align => {
                    if let Operand::Expr(expr) = &stmt.operands[0] {
                        let val =
                            eval_expr(expr, &label_map, &self.options, current_addr).unwrap_or(0);
                        let align_bytes = 1 << val;
                        let pad = if current_addr.is_multiple_of(align_bytes) {
                            0
                        } else {
                            align_bytes - (current_addr % align_bytes)
                        };
                        bytes.resize(bytes.len() + pad as usize, 0);
                        current_addr += pad;
                        instruction_count += 1;
                    }
                    continue;
                }
                Mnemonic::Float => {
                    if let Operand::Float(val) = stmt.operands[0] {
                        let word = val.to_bits();
                        match self.options.endian {
                            Endian::Big => bytes.extend_from_slice(&word.to_be_bytes()),
                            Endian::Little => bytes.extend_from_slice(&word.to_le_bytes()),
                        }
                        current_addr += 4;
                        instruction_count += 1;
                    }
                    continue;
                }
                Mnemonic::Ascii => {
                    if let Operand::StringBytes(ref b) = stmt.operands[0] {
                        bytes.extend_from_slice(b);
                        current_addr += b.len() as u32;
                        instruction_count += 1;
                    }
                    continue;
                }
                Mnemonic::Asciz => {
                    if let Operand::StringBytes(ref b) = stmt.operands[0] {
                        bytes.extend_from_slice(b);
                        bytes.push(0);
                        current_addr += (b.len() + 1) as u32;
                        instruction_count += 1;
                    }
                    continue;
                }
                _ => {}
            }

            let instr = translate_statement(stmt, &label_map, &self.options, current_addr)?;
            let word = encode_instruction(&instr).map_err(|e| {
                if let AsmError::ImmediateOutOfRange { .. } = e {
                    AsmError::ImmediateOutOfRange {
                        line: stmt.line,
                        value: 0,
                    }
                } else {
                    e
                }
            })?;

            match self.options.endian {
                Endian::Big => bytes.extend_from_slice(&word.to_be_bytes()),
                Endian::Little => bytes.extend_from_slice(&word.to_le_bytes()),
            }
            current_addr += 4;
            instruction_count += 1;
        }

        // Smartly detect typical entry points to populate the result
        let entry_point = label_map
            .get("_start")
            .or_else(|| label_map.get("main"))
            .copied()
            .unwrap_or(self.options.start_address) as u64;

        let labels = label_map.into_iter().map(|(k, v)| (k, v as u64)).collect();

        Ok(AssembleResult {
            bytes,
            entry_point,
            labels,
            instruction_count,
        })
    }
}

pub fn assemble(source: &str) -> Result<AssembleResult, AsmError> {
    Encoder::new().assemble(source)
}

fn process_line(line: &str) -> String {
    if let Some(pos) = line.find('@') {
        line[..pos].to_string()
    } else {
        line.to_string()
    }
}

fn eval_expr(
    expr: &Expr,
    labels: &HashMap<String, u32>,
    options: &AssemblerOptions,
    current_addr: u32,
) -> Result<i32, String> {
    match expr {
        Expr::Number(n) => Ok(*n),
        Expr::CurrentAddress => Ok(current_addr as i32),
        Expr::Symbol(s) => {
            if let Some(&v) = labels.get(s) {
                Ok(v as i32)
            } else if let Some(ref resolver) = options.symbol_resolver
                && let Some(v) = resolver.resolve(s)
            {
                Ok(v as i32)
            } else {
                Err(format!("undefined label '{}' in expression", s))
            }
        }
        Expr::Add(l, r) => Ok(eval_expr(l, labels, options, current_addr)?
            + eval_expr(r, labels, options, current_addr)?),
        Expr::Sub(l, r) => Ok(eval_expr(l, labels, options, current_addr)?
            - eval_expr(r, labels, options, current_addr)?),
    }
}

fn translate_statement(
    stmt: &Statement,
    labels: &HashMap<String, u32>,
    options: &AssemblerOptions,
    addr: u32,
) -> Result<Instruction, AsmError> {
    let cond = stmt.condition;
    match &stmt.mnemonic {
        Mnemonic::LabelOnly
        | Mnemonic::Global
        | Mnemonic::Text
        | Mnemonic::Data
        | Mnemonic::Align
        | Mnemonic::Ascii
        | Mnemonic::Asciz
        | Mnemonic::It
        | Mnemonic::Byte
        | Mnemonic::Short
        | Mnemonic::Space
        | Mnemonic::Float => unreachable!(),

        Mnemonic::DataProcessing(opcode) => {
            let (rd, rn, op2) = match stmt.operands.as_slice() {
                [Operand::Reg(rd), op2] if matches!(opcode, DataOpcode::Mov | DataOpcode::Mvn) => {
                    (*rd, None, op2_to_shifter(op2)?)
                }
                [Operand::Reg(rn), op2]
                    if matches!(
                        opcode,
                        DataOpcode::Cmp | DataOpcode::Cmn | DataOpcode::Tst | DataOpcode::Teq
                    ) =>
                {
                    (Register::R0, Some(*rn), op2_to_shifter(op2)?)
                }
                [Operand::Reg(rd), Operand::Reg(rn), op2] => (*rd, Some(*rn), op2_to_shifter(op2)?),
                _ => {
                    return Err(AsmError::ParseError {
                        line: stmt.line,
                        col: 0,
                        message: "invalid operands".into(),
                    });
                }
            };
            Ok(Instruction::DataProcessing {
                cond,
                s: stmt.s_flag,
                opcode: *opcode,
                rd,
                rn,
                operand2: op2,
            })
        }
        Mnemonic::Shift(st) => {
            if *st == ShiftType::Rrx {
                if let [Operand::Reg(rd), Operand::Reg(rm)] = &stmt.operands[..] {
                    Ok(Instruction::DataProcessing {
                        cond,
                        s: stmt.s_flag,
                        opcode: DataOpcode::Mov,
                        rd: *rd,
                        rn: None,
                        operand2: ShifterOperand::Rrx(*rm),
                    })
                } else {
                    Err(AsmError::ParseError {
                        line: stmt.line,
                        col: 0,
                        message: "RRX expects Rd, Rm".into(),
                    })
                }
            } else {
                match &stmt.operands[..] {
                    [Operand::Reg(rd), Operand::Reg(rm), Operand::Reg(rs)] => {
                        Ok(Instruction::DataProcessing {
                            cond,
                            s: stmt.s_flag,
                            opcode: DataOpcode::Mov,
                            rd: *rd,
                            rn: None,
                            operand2: ShifterOperand::RegisterShift(*rm, *st, *rs),
                        })
                    }
                    [Operand::Reg(rd), Operand::Reg(rm), Operand::Imm(imm)] => {
                        Ok(Instruction::DataProcessing {
                            cond,
                            s: stmt.s_flag,
                            opcode: DataOpcode::Mov,
                            rd: *rd,
                            rn: None,
                            operand2: ShifterOperand::ImmediateShift(*rm, *st, *imm),
                        })
                    }
                    _ => Err(AsmError::ParseError {
                        line: stmt.line,
                        col: 0,
                        message: "Shift expects Rd, Rm, Rs or Rd, Rm, #imm".into(),
                    }),
                }
            }
        }
        Mnemonic::Ldr | Mnemonic::Str => {
            let load = matches!(stmt.mnemonic, Mnemonic::Ldr);
            match &stmt.operands[..] {
                [Operand::Reg(rd), Operand::Memory(addr_mode)] => Ok(Instruction::LoadStore {
                    cond,
                    load,
                    byte: false,
                    rd: *rd,
                    addressing: addr_mode.clone(),
                }),
                [Operand::Reg(rd), Operand::Label(label)] => {
                    let target = resolve_label(label, labels, options, stmt.line)?;
                    Ok(Instruction::LoadStore {
                        cond,
                        load,
                        byte: false,
                        rd: *rd,
                        addressing: AddressingMode::OffsetImmediate(
                            Register::Pc,
                            target as i32 - (addr as i32 + 8),
                        ),
                    })
                }
                _ => Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "LDR/STR require Rd, [memory] or Rd, label".into(),
                }),
            }
        }
        Mnemonic::Ldrb | Mnemonic::Strb => {
            let load = matches!(stmt.mnemonic, Mnemonic::Ldrb);
            match &stmt.operands[..] {
                [Operand::Reg(rd), Operand::Memory(addr_mode)] => Ok(Instruction::LoadStore {
                    cond,
                    load,
                    byte: true,
                    rd: *rd,
                    addressing: addr_mode.clone(),
                }),
                [Operand::Reg(rd), Operand::Label(label)] => {
                    let target = resolve_label(label, labels, options, stmt.line)?;
                    Ok(Instruction::LoadStore {
                        cond,
                        load,
                        byte: true,
                        rd: *rd,
                        addressing: AddressingMode::OffsetImmediate(
                            Register::Pc,
                            target as i32 - (addr as i32 + 8),
                        ),
                    })
                }
                _ => Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "LDRB/STRB require Rd,[memory] or Rd, label".into(),
                }),
            }
        }
        Mnemonic::LoadStoreExtra(op) => match &stmt.operands[..] {
            [Operand::Reg(rd), Operand::Memory(addr_mode)] => Ok(Instruction::LoadStoreExtra {
                cond,
                op: *op,
                rd: *rd,
                addressing: addr_mode.clone(),
            }),
            _ => Err(AsmError::ParseError {
                line: stmt.line,
                col: 0,
                message: "Extra Load/Store requires Rd,[memory]".into(),
            }),
        },
        Mnemonic::Push => {
            if let [Operand::RegList(regs)] = &stmt.operands[..] {
                Ok(Instruction::Push {
                    cond,
                    reg_list: regs.clone(),
                })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "PUSH requires register list".into(),
                })
            }
        }
        Mnemonic::Pop => {
            if let [Operand::RegList(regs)] = &stmt.operands[..] {
                Ok(Instruction::Pop {
                    cond,
                    reg_list: regs.clone(),
                })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "POP requires register list".into(),
                })
            }
        }
        Mnemonic::Mul => {
            if let [Operand::Reg(rd), Operand::Reg(rn), Operand::Reg(rm)] = &stmt.operands[..] {
                Ok(Instruction::Multiply {
                    cond,
                    s: stmt.s_flag,
                    rd: *rd,
                    rn: *rn,
                    rm: *rm,
                })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "MUL requires Rd, Rn, Rm".into(),
                })
            }
        }
        Mnemonic::Mla => {
            if let [
                Operand::Reg(rd),
                Operand::Reg(rn),
                Operand::Reg(rm),
                Operand::Reg(ra),
            ] = &stmt.operands[..]
            {
                Ok(Instruction::MultiplyAccumulate {
                    cond,
                    s: stmt.s_flag,
                    rd: *rd,
                    rn: *rn,
                    rm: *rm,
                    ra: *ra,
                })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "MLA requires Rd, Rn, Rm, Ra".into(),
                })
            }
        }
        Mnemonic::Mls => {
            if let [
                Operand::Reg(rd),
                Operand::Reg(rn),
                Operand::Reg(rm),
                Operand::Reg(ra),
            ] = &stmt.operands[..]
            {
                Ok(Instruction::MultiplySubtract {
                    cond,
                    rd: *rd,
                    rn: *rn,
                    rm: *rm,
                    ra: *ra,
                })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "MLS requires Rd, Rn, Rm, Ra".into(),
                })
            }
        }
        Mnemonic::Sdiv => {
            if let [Operand::Reg(rd), Operand::Reg(rn), Operand::Reg(rm)] = &stmt.operands[..] {
                Ok(Instruction::Divide {
                    cond,
                    unsigned: false,
                    rd: *rd,
                    rn: *rn,
                    rm: *rm,
                })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "SDIV requires Rd, Rn, Rm".into(),
                })
            }
        }
        Mnemonic::Udiv => {
            if let [Operand::Reg(rd), Operand::Reg(rn), Operand::Reg(rm)] = &stmt.operands[..] {
                Ok(Instruction::Divide {
                    cond,
                    unsigned: true,
                    rd: *rd,
                    rn: *rn,
                    rm: *rm,
                })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "UDIV requires Rd, Rn, Rm".into(),
                })
            }
        }
        Mnemonic::B | Mnemonic::Bl => {
            let link = matches!(stmt.mnemonic, Mnemonic::Bl);
            if let Operand::Label(label) = &stmt.operands[0] {
                let target = resolve_label(label, labels, options, stmt.line)?;
                let pc = addr + 8;
                let offset = (target as i32 - pc as i32) / 4;
                Ok(Instruction::Branch { cond, link, offset })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "branch expects label".into(),
                })
            }
        }
        Mnemonic::Bx => {
            if let Operand::Reg(rm) = &stmt.operands[0] {
                Ok(Instruction::BranchExchange { cond, rm: *rm })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "BX expects register".into(),
                })
            }
        }
        Mnemonic::Svc => {
            if let [Operand::Imm(imm)] = &stmt.operands[..] {
                Ok(Instruction::Svc { cond, imm: *imm })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "SVC requires immediate".into(),
                })
            }
        }
        Mnemonic::Nop => Ok(Instruction::Hint { cond, hint: 0 }),
        Mnemonic::Yield => Ok(Instruction::Hint { cond, hint: 1 }),
        Mnemonic::Wfe => Ok(Instruction::Hint { cond, hint: 2 }),
        Mnemonic::Wfi => Ok(Instruction::Hint { cond, hint: 3 }),
        Mnemonic::Sev => Ok(Instruction::Hint { cond, hint: 4 }),
        Mnemonic::Bkpt => {
            if let [Operand::Imm(imm)] = &stmt.operands[..] {
                if *imm > 0xFFFF {
                    return Err(AsmError::ImmediateOutOfRange {
                        line: stmt.line,
                        value: *imm,
                    });
                }
                Ok(Instruction::Bkpt { imm: *imm as u16 })
            } else {
                Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "BKPT requires immediate".into(),
                })
            }
        }
        Mnemonic::Word => {
            let val = match &stmt.operands[0] {
                Operand::Expr(expr) => {
                    eval_expr(expr, labels, options, addr).map_err(|e| AsmError::ParseError {
                        line: stmt.line,
                        col: 0,
                        message: e,
                    })? as u32
                }
                _ => {
                    return Err(AsmError::ParseError {
                        line: stmt.line,
                        col: 0,
                        message: ".word expects expression".into(),
                    });
                }
            };
            Ok(Instruction::RawWord(val))
        }
        Mnemonic::Movw => match &stmt.operands[..] {
            [Operand::Reg(rd), Operand::Imm(imm)] => {
                if *imm > 0xFFFF {
                    return Err(AsmError::ImmediateOutOfRange {
                        line: stmt.line,
                        value: *imm,
                    });
                }
                Ok(Instruction::Movw {
                    cond,
                    rd: *rd,
                    imm16: *imm as u16,
                })
            }
            _ => Err(AsmError::ParseError {
                line: stmt.line,
                col: 0,
                message: "MOVW expects Rd, #imm16".into(),
            }),
        },
        Mnemonic::Movt => match &stmt.operands[..] {
            [Operand::Reg(rd), Operand::Imm(imm)] => {
                if *imm > 0xFFFF {
                    return Err(AsmError::ImmediateOutOfRange {
                        line: stmt.line,
                        value: *imm,
                    });
                }
                Ok(Instruction::Movt {
                    cond,
                    rd: *rd,
                    imm16: *imm as u16,
                })
            }
            _ => Err(AsmError::ParseError {
                line: stmt.line,
                col: 0,
                message: "MOVT expects Rd, #imm16".into(),
            }),
        },
        Mnemonic::Ldm | Mnemonic::Stm => {
            let load = matches!(stmt.mnemonic, Mnemonic::Ldm);
            match &stmt.operands[..] {
                [Operand::Reg(rn), Operand::RegList(regs)] => Ok(Instruction::LdmStm {
                    cond,
                    load,
                    rn: *rn,
                    reg_list: regs.clone(),
                    writeback: false,
                }),
                [Operand::RegWriteback(rn), Operand::RegList(regs)] => Ok(Instruction::LdmStm {
                    cond,
                    load,
                    rn: *rn,
                    reg_list: regs.clone(),
                    writeback: true,
                }),
                _ => Err(AsmError::ParseError {
                    line: stmt.line,
                    col: 0,
                    message: "LDM/STM expect Rn or Rn!, {reglist}".into(),
                }),
            }
        }
        Mnemonic::Sxtb => build_extend(cond, ExtendOp::Sxtb, stmt),
        Mnemonic::Uxtb => build_extend(cond, ExtendOp::Uxtb, stmt),
        Mnemonic::Sxth => build_extend(cond, ExtendOp::Sxth, stmt),
        Mnemonic::Uxth => build_extend(cond, ExtendOp::Uxth, stmt),
        Mnemonic::Rev => build_reverse(cond, ReverseOp::Rev, stmt),
        Mnemonic::Rev16 => build_reverse(cond, ReverseOp::Rev16, stmt),
        Mnemonic::Revsh => build_reverse(cond, ReverseOp::RevSH, stmt),
    }
}

fn build_extend(cond: Condition, op: ExtendOp, stmt: &Statement) -> Result<Instruction, AsmError> {
    match &stmt.operands[..] {
        [Operand::Reg(rd), Operand::Reg(rm)] => Ok(Instruction::Extend {
            cond,
            op,
            rd: *rd,
            rm: *rm,
        }),
        _ => Err(AsmError::ParseError {
            line: stmt.line,
            col: 0,
            message: "extend expects Rd, Rm".into(),
        }),
    }
}

fn build_reverse(
    cond: Condition,
    op: ReverseOp,
    stmt: &Statement,
) -> Result<Instruction, AsmError> {
    match &stmt.operands[..] {
        [Operand::Reg(rd), Operand::Reg(rm)] => Ok(Instruction::Reverse {
            cond,
            op,
            rd: *rd,
            rm: *rm,
        }),
        _ => Err(AsmError::ParseError {
            line: stmt.line,
            col: 0,
            message: "reverse expects Rd, Rm".into(),
        }),
    }
}

fn op2_to_shifter(op: &Operand) -> Result<ShifterOperand, AsmError> {
    match op {
        Operand::Reg(r) => Ok(ShifterOperand::Register(*r)),
        Operand::Imm(i) => Ok(ShifterOperand::Immediate(*i)),
        Operand::Shifter(s) => Ok(s.clone()),
        _ => Err(AsmError::ParseError {
            line: 0,
            col: 0,
            message: "invalid operand type".into(),
        }),
    }
}

fn resolve_label(
    name: &str,
    local: &HashMap<String, u32>,
    options: &AssemblerOptions,
    line: usize,
) -> Result<u32, AsmError> {
    if let Some(&addr) = local.get(name) {
        return Ok(addr);
    }
    if let Some(ref resolver) = options.symbol_resolver
        && let Some(addr) = resolver.resolve(name)
    {
        return Ok(addr);
    }
    Err(AsmError::UndefinedLabel {
        line,
        label: name.to_string(),
    })
}
