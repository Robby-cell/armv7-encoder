use crate::encoder::{
    AddressingMode, Condition, DataOpcode, ExtraLoadStoreOp, Register, ShiftType, ShifterOperand,
};
use crate::error::AsmError;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while, take_while1},
    character::complete::{char, digit1, hex_digit1, space0},
    combinator::{map, map_res, opt, recognize, value},
    multi::separated_list1,
    sequence::{preceded, terminated},
};

fn sp(input: &str) -> IResult<&str, &str> {
    space0(input)
}

fn register(input: &str) -> IResult<&str, Register> {
    let (input, name) = take_while1(|c: char| c.is_alphanumeric()).parse(input)?;
    let lower = name.to_lowercase();
    let reg = match lower.as_str() {
        "r0" | "a1" => Register::R0,
        "r1" | "a2" => Register::R1,
        "r2" | "a3" => Register::R2,
        "r3" | "a4" => Register::R3,
        "r4" | "v1" => Register::R4,
        "r5" | "v2" => Register::R5,
        "r6" | "v3" => Register::R6,
        "r7" | "v4" => Register::R7,
        "r8" | "v5" => Register::R8,
        "r9" | "v6" | "sb" => Register::R9,
        "r10" | "v7" | "sl" => Register::R10,
        "r11" | "v8" | "fp" => Register::R11,
        "r12" | "ip" => Register::R12,
        "r13" | "sp" => Register::Sp,
        "r14" | "lr" => Register::Lr,
        "r15" | "pc" => Register::Pc,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((input, reg))
}

fn register_item(input: &str) -> IResult<&str, Vec<Register>> {
    let (input, reg1) = register(input)?;
    let (input, has_dash) = opt(preceded((sp, char('-'), sp), register)).parse(input)?;

    if let Some(reg2) = has_dash {
        let start = reg1.code();
        let end = reg2.code();
        if start > end {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
        let mut regs = Vec::new();
        for c in start..=end {
            // Unwrapping is safe because `Register::code()` values are guaranteed to be 0..=15
            regs.push(Register::from_code(c).unwrap());
        }
        Ok((input, regs))
    } else {
        Ok((input, vec![reg1]))
    }
}

fn register_list(input: &str) -> IResult<&str, Vec<Register>> {
    let (input, _) = char('{').parse(input)?;
    let (input, _) = sp(input)?;
    let (input, items) = separated_list1((sp, char(','), sp), register_item).parse(input)?;
    let (input, _) = sp(input)?;
    let (input, _) = char('}').parse(input)?;

    // Flatten multiple potentially ranged vectors into a single list
    Ok((input, items.into_iter().flatten().collect()))
}

fn string_literal(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"').parse(input)?;
    let mut out = String::new();
    let mut chars = input.chars();
    let mut rest;

    while let Some(c) = chars.next() {
        rest = chars.as_str();
        if c == '"' {
            return Ok((rest, out));
        }
        if c == '\\' {
            if let Some(nc) = chars.next() {
                match nc {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\\' => out.push('\\'),
                    '"' => out.push('"'),
                    '0' => out.push('\0'),
                    _ => {
                        out.push('\\');
                        out.push(nc);
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn immediate(input: &str) -> IResult<&str, u32> {
    preceded(
        opt(char('#')),
        preceded(
            space0,
            map(
                (
                    opt(char('-')),
                    alt((
                        preceded(
                            tag("0x"),
                            map_res(hex_digit1, |h: &str| u32::from_str_radix(h, 16)),
                        ),
                        preceded(
                            tag("0X"),
                            map_res(hex_digit1, |h: &str| u32::from_str_radix(h, 16)),
                        ),
                        map_res(digit1, |d: &str| d.parse::<u32>()),
                    )),
                ),
                |(minus, val)| {
                    if minus.is_some() {
                        (-(val as i32)) as u32
                    } else {
                        val
                    }
                },
            ),
        ),
    )
    .parse(input)
}

fn float_literal(input: &str) -> IResult<&str, f32> {
    let (input, float_str) = recognize((
        opt(alt((char('+'), char('-')))),
        digit1,
        opt((char('.'), digit1)),
        opt((
            alt((char('e'), char('E'))),
            opt(alt((char('+'), char('-')))),
            digit1,
        )),
    ))
    .parse(input)?;

    let val = float_str.parse::<f32>().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
    })?;
    Ok((input, val))
}

fn label_name(input: &str) -> IResult<&str, String> {
    map(
        recognize((
            take_while1(|c: char| c.is_alphabetic() || c == '_' || c == '.'),
            take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '.'),
        )),
        |s: &str| s.to_string(),
    )
    .parse(input)
}

fn condition_parser(input: &str) -> IResult<&str, Condition> {
    alt((
        value(Condition::Eq, tag_no_case("eq")),
        value(Condition::Ne, tag_no_case("ne")),
        value(Condition::Cs, tag_no_case("cs")),
        value(Condition::Cc, tag_no_case("cc")),
        value(Condition::Mi, tag_no_case("mi")),
        value(Condition::Pl, tag_no_case("pl")),
        value(Condition::Vs, tag_no_case("vs")),
        value(Condition::Vc, tag_no_case("vc")),
        value(Condition::Hi, tag_no_case("hi")),
        value(Condition::Ls, tag_no_case("ls")),
        value(Condition::Ge, tag_no_case("ge")),
        value(Condition::Lt, tag_no_case("lt")),
        value(Condition::Gt, tag_no_case("gt")),
        value(Condition::Le, tag_no_case("le")),
        value(Condition::Al, tag_no_case("al")),
    ))
    .parse(input)
}

fn shift_type(input: &str) -> IResult<&str, ShiftType> {
    alt((
        value(ShiftType::Lsl, tag_no_case("lsl")),
        value(ShiftType::Lsr, tag_no_case("lsr")),
        value(ShiftType::Asr, tag_no_case("asr")),
        value(ShiftType::Ror, tag_no_case("ror")),
        value(ShiftType::Rrx, tag_no_case("rrx")),
    ))
    .parse(input)
}

enum ShiftAmount {
    Immediate(u32),
    Register(Register),
}

fn shift_amount(input: &str) -> IResult<&str, ShiftAmount> {
    alt((
        map(immediate, ShiftAmount::Immediate),
        map(register, ShiftAmount::Register),
    ))
    .parse(input)
}

fn shifter_operand(input: &str) -> IResult<&str, ShifterOperand> {
    if let Ok((rest, val)) = immediate(input) {
        return Ok((rest, ShifterOperand::Immediate(val)));
    }

    let (input, rm) = register(input)?;
    let (input, _) = sp(input)?;
    let (input, shift) = opt(preceded(
        (char(','), sp),
        alt((
            map(tag_no_case("rrx"), |_| (ShiftType::Rrx, None)),
            map((shift_type, sp, shift_amount), |(st, _, amt)| {
                (st, Some(amt))
            }),
        )),
    ))
    .parse(input)?;

    match shift {
        None => Ok((input, ShifterOperand::Register(rm))),
        Some((ShiftType::Rrx, _)) => Ok((input, ShifterOperand::Rrx(rm))),
        Some((st, Some(ShiftAmount::Immediate(imm)))) => {
            if imm > 31 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Digit,
                )));
            }
            Ok((input, ShifterOperand::ImmediateShift(rm, st, imm)))
        }
        Some((st, Some(ShiftAmount::Register(rs)))) => {
            if rs == Register::Pc {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )));
            }
            Ok((input, ShifterOperand::RegisterShift(rm, st, rs)))
        }
        _ => unreachable!(),
    }
}

fn memory(input: &str) -> IResult<&str, AddressingMode> {
    let (input, _) = char('[').parse(input)?;
    let (input, _) = sp(input)?;
    let (input, rn) = register(input)?;
    let (input, _) = sp(input)?;
    let (input, addr_opt) = opt(preceded(
        (char(','), sp),
        alt((
            map(immediate, move |imm| {
                AddressingMode::OffsetImmediate(rn, imm as i32)
            }),
            map(
                (
                    register,
                    opt(preceded((char(','), sp), (shift_type, sp, immediate))),
                ),
                move |(rm, shift_opt)| {
                    if let Some((shift, _, imm)) = shift_opt {
                        AddressingMode::OffsetScaled(rn, rm, shift, imm)
                    } else {
                        AddressingMode::OffsetRegister(rn, rm)
                    }
                },
            ),
        )),
    ))
    .parse(input)?;
    let (input, _) = sp(input)?;
    let (input, _) = char(']').parse(input)?;
    Ok((
        input,
        addr_opt.unwrap_or(AddressingMode::OffsetImmediate(rn, 0)),
    ))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mnemonic {
    DataProcessing(DataOpcode),
    Shift(ShiftType),
    Ldr,
    Str,
    Ldrb,
    Strb,
    LoadStoreExtra(ExtraLoadStoreOp),
    Push,
    Pop,
    Mul,
    Mla,
    Mls,
    Sdiv,
    Udiv,
    B,
    Bl,
    Bx,
    Svc,
    Nop,
    Bkpt,
    Wfi,
    Wfe,
    Yield,
    Sev,
    Global,
    Text,
    Data,
    Align,
    Ascii,
    Asciz,
    Word,
    LabelOnly,
    It,
    Float,
    Movw,
    Movt,
    Ldm,
    Stm,
    Sxtb,
    Uxtb,
    Sxth,
    Uxth,
    Rev,
    Rev16,
    Revsh,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Reg(Register),

    // TODO: Unused field
    #[allow(unused)]
    RegWriteback(Register),
    Imm(u32),
    Shifter(ShifterOperand),
    Label(String),
    Memory(AddressingMode),
    RegList(Vec<Register>),
    PseudoLoadLabel(String),
    PseudoLoadImm(u32),
    StringBytes(Vec<u8>),

    // TODO: Unused field
    #[allow(unused)]
    Cond(Condition),

    Float(f32),
}

#[derive(Debug, Clone)]
pub struct MnemonicInfo {
    pub op: Mnemonic,
    pub condition: Condition,
    pub set_flags: bool,
}

fn try_condition(s: &str) -> Option<Condition> {
    condition_parser(s)
        .ok()
        .filter(|(rest, _)| rest.is_empty())
        .map(|(_, c)| c)
}

fn parse_mnemonic_with_modifiers(input: &str) -> IResult<&str, MnemonicInfo> {
    let (remaining, token) = take_while1(|c: char| c.is_alphanumeric()).parse(input)?;
    let token_lower = token.to_lowercase();

    if let Some(rest) = token_lower.strip_prefix("it")
        && (rest.is_empty() || rest.chars().all(|c| c == 't' || c == 'e'))
        && rest.len() <= 3
    {
        return Ok((
            remaining,
            MnemonicInfo {
                op: Mnemonic::It,
                condition: Condition::Al,
                set_flags: false,
            },
        ));
    }

    let bases = [
        ("ldrb", Mnemonic::Ldrb),
        ("strb", Mnemonic::Strb),
        ("ldrh", Mnemonic::LoadStoreExtra(ExtraLoadStoreOp::Ldrh)),
        ("strh", Mnemonic::LoadStoreExtra(ExtraLoadStoreOp::Strh)),
        ("ldrsb", Mnemonic::LoadStoreExtra(ExtraLoadStoreOp::Ldrsb)),
        ("ldrsh", Mnemonic::LoadStoreExtra(ExtraLoadStoreOp::Ldrsh)),
        ("ldrd", Mnemonic::LoadStoreExtra(ExtraLoadStoreOp::Ldrd)),
        ("strd", Mnemonic::LoadStoreExtra(ExtraLoadStoreOp::Strd)),
        ("push", Mnemonic::Push),
        ("pop", Mnemonic::Pop),
        ("mul", Mnemonic::Mul),
        ("mla", Mnemonic::Mla),
        ("mls", Mnemonic::Mls),
        ("sdiv", Mnemonic::Sdiv),
        ("udiv", Mnemonic::Udiv),
        ("and", Mnemonic::DataProcessing(DataOpcode::And)),
        ("eor", Mnemonic::DataProcessing(DataOpcode::Eor)),
        ("sub", Mnemonic::DataProcessing(DataOpcode::Sub)),
        ("rsb", Mnemonic::DataProcessing(DataOpcode::Rsb)),
        ("add", Mnemonic::DataProcessing(DataOpcode::Add)),
        ("adc", Mnemonic::DataProcessing(DataOpcode::Adc)),
        ("sbc", Mnemonic::DataProcessing(DataOpcode::Sbc)),
        ("rsc", Mnemonic::DataProcessing(DataOpcode::Rsc)),
        ("tst", Mnemonic::DataProcessing(DataOpcode::Tst)),
        ("teq", Mnemonic::DataProcessing(DataOpcode::Teq)),
        ("cmp", Mnemonic::DataProcessing(DataOpcode::Cmp)),
        ("cmn", Mnemonic::DataProcessing(DataOpcode::Cmn)),
        ("orr", Mnemonic::DataProcessing(DataOpcode::Orr)),
        ("mov", Mnemonic::DataProcessing(DataOpcode::Mov)),
        ("bic", Mnemonic::DataProcessing(DataOpcode::Bic)),
        ("mvn", Mnemonic::DataProcessing(DataOpcode::Mvn)),
        ("ldr", Mnemonic::Ldr),
        ("str", Mnemonic::Str),
        ("bl", Mnemonic::Bl),
        ("bx", Mnemonic::Bx),
        ("b", Mnemonic::B),
        ("svc", Mnemonic::Svc),
        ("swi", Mnemonic::Svc),
        ("nop", Mnemonic::Nop),
        ("bkpt", Mnemonic::Bkpt),
        ("wfi", Mnemonic::Wfi),
        ("wfe", Mnemonic::Wfe),
        ("yield", Mnemonic::Yield),
        ("sev", Mnemonic::Sev),
        ("movw", Mnemonic::Movw),
        ("movt", Mnemonic::Movt),
        ("ldmia", Mnemonic::Ldm),
        ("ldm", Mnemonic::Ldm),
        ("stmia", Mnemonic::Stm),
        ("stm", Mnemonic::Stm),
        ("sxtb", Mnemonic::Sxtb),
        ("uxtb", Mnemonic::Uxtb),
        ("sxth", Mnemonic::Sxth),
        ("uxth", Mnemonic::Uxth),
        ("revsh", Mnemonic::Revsh),
        ("rev16", Mnemonic::Rev16),
        ("rev", Mnemonic::Rev),
        ("lsl", Mnemonic::Shift(ShiftType::Lsl)),
        ("lsr", Mnemonic::Shift(ShiftType::Lsr)),
        ("asr", Mnemonic::Shift(ShiftType::Asr)),
        ("ror", Mnemonic::Shift(ShiftType::Ror)),
        ("rrx", Mnemonic::Shift(ShiftType::Rrx)),
    ];

    for (name, op) in bases.iter() {
        if let Some(rest) = token_lower.strip_prefix(name) {
            let mut s = rest;
            let mut set_flags = false;
            let mut cond = Condition::Al;

            if let Some(r) = s.strip_suffix('s') {
                set_flags = true;
                s = r;
            } else if let Some(r) = s.strip_prefix('s') {
                set_flags = true;
                s = r;
            }

            if !s.is_empty() {
                if let Some(c) = try_condition(s) {
                    cond = c;
                } else {
                    continue;
                }
            }

            if matches!(
                op,
                Mnemonic::DataProcessing(
                    DataOpcode::Cmp | DataOpcode::Cmn | DataOpcode::Tst | DataOpcode::Teq
                )
            ) {
                set_flags = true;
            }

            return Ok((
                remaining,
                MnemonicInfo {
                    op: op.clone(),
                    condition: cond,
                    set_flags,
                },
            ));
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn parse_data_proc_operands(input: &str, opcode: DataOpcode) -> IResult<&str, Vec<Operand>> {
    match opcode {
        DataOpcode::Mov | DataOpcode::Mvn => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, op2) = shifter_operand(input)?;
            Ok((input, vec![Operand::Reg(rd), Operand::Shifter(op2)]))
        }
        DataOpcode::Cmp | DataOpcode::Cmn | DataOpcode::Tst | DataOpcode::Teq => {
            let (input, rn) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, op2) = shifter_operand(input)?;
            Ok((input, vec![Operand::Reg(rn), Operand::Shifter(op2)]))
        }
        _ => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, rn) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, op2) = shifter_operand(input)?;
            Ok((
                input,
                vec![Operand::Reg(rd), Operand::Reg(rn), Operand::Shifter(op2)],
            ))
        }
    }
}

fn parse_operands_for_mnemonic<'a>(
    input: &'a str,
    info: &MnemonicInfo,
) -> IResult<&'a str, Vec<Operand>> {
    match &info.op {
        Mnemonic::DataProcessing(op) => parse_data_proc_operands(input, *op),
        Mnemonic::Shift(st) => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;

            if *st == ShiftType::Rrx {
                let (input, rm) = opt(preceded((sp, char(','), sp), register)).parse(input)?;
                if let Some(rm) = rm {
                    Ok((input, vec![Operand::Reg(rd), Operand::Reg(rm)]))
                } else {
                    Ok((input, vec![Operand::Reg(rd), Operand::Reg(rd)]))
                }
            } else {
                let (input, op2) = alt((map(register, Operand::Reg), map(immediate, Operand::Imm)))
                    .parse(input)?;
                let (input, op3) = opt(preceded(
                    (sp, char(','), sp),
                    alt((map(register, Operand::Reg), map(immediate, Operand::Imm))),
                ))
                .parse(input)?;

                if let Some(op3) = op3 {
                    Ok((input, vec![Operand::Reg(rd), op2, op3]))
                } else {
                    Ok((input, vec![Operand::Reg(rd), Operand::Reg(rd), op2]))
                }
            }
        }
        Mnemonic::It => {
            let (input, cond) = condition_parser(input)?;
            Ok((input, vec![Operand::Cond(cond)]))
        }
        Mnemonic::Ldr => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, op2) = alt((
                map(memory, Operand::Memory),
                preceded(
                    char('='),
                    alt((
                        map(immediate, Operand::PseudoLoadImm),
                        map(label_name, Operand::PseudoLoadLabel),
                    )),
                ),
                map(label_name, Operand::Label),
            ))
            .parse(input)?;
            Ok((input, vec![Operand::Reg(rd), op2]))
        }
        Mnemonic::Str | Mnemonic::Ldrb | Mnemonic::Strb | Mnemonic::LoadStoreExtra(_) => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, op2) = alt((
                map(memory, Operand::Memory),
                map(label_name, Operand::Label),
            ))
            .parse(input)?;
            Ok((input, vec![Operand::Reg(rd), op2]))
        }
        Mnemonic::Push | Mnemonic::Pop => {
            let (input, regs) = register_list(input)?;
            Ok((input, vec![Operand::RegList(regs)]))
        }
        Mnemonic::Mul | Mnemonic::Sdiv | Mnemonic::Udiv => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, rn) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, rm) = register(input)?;
            Ok((
                input,
                vec![Operand::Reg(rd), Operand::Reg(rn), Operand::Reg(rm)],
            ))
        }
        Mnemonic::Mla | Mnemonic::Mls => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, rn) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, rm) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, ra) = register(input)?;
            Ok((
                input,
                vec![
                    Operand::Reg(rd),
                    Operand::Reg(rn),
                    Operand::Reg(rm),
                    Operand::Reg(ra),
                ],
            ))
        }
        Mnemonic::B | Mnemonic::Bl => {
            let (input, label) = label_name(input)?;
            Ok((input, vec![Operand::Label(label)]))
        }
        Mnemonic::Bx => {
            let (input, rm) = register(input)?;
            Ok((input, vec![Operand::Reg(rm)]))
        }
        Mnemonic::Svc | Mnemonic::Bkpt => {
            let (input, imm) = immediate(input)?;
            Ok((input, vec![Operand::Imm(imm)]))
        }
        Mnemonic::Nop | Mnemonic::Wfi | Mnemonic::Wfe | Mnemonic::Yield | Mnemonic::Sev => {
            Ok((input, vec![]))
        }
        Mnemonic::Movw | Mnemonic::Movt => {
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, imm) = immediate(input)?;
            Ok((input, vec![Operand::Reg(rd), Operand::Imm(imm)]))
        }
        Mnemonic::Ldm | Mnemonic::Stm => {
            let (input, rn) = register(input)?;
            let (input, wb) = opt(char('!')).parse(input)?; // Explicitly capture the optional writeback flag
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, regs) = register_list(input)?;

            let rn_operand = if wb.is_some() {
                Operand::RegWriteback(rn)
            } else {
                Operand::Reg(rn)
            };

            Ok((input, vec![rn_operand, Operand::RegList(regs)]))
        }
        Mnemonic::Sxtb
        | Mnemonic::Uxtb
        | Mnemonic::Sxth
        | Mnemonic::Uxth
        | Mnemonic::Rev
        | Mnemonic::Rev16
        | Mnemonic::Revsh => {
            // Rd, Rm format
            let (input, rd) = register(input)?;
            let (input, _) = (sp, char(','), sp).parse(input)?;
            let (input, rm) = register(input)?;
            Ok((input, vec![Operand::Reg(rd), Operand::Reg(rm)]))
        }
        _ => unreachable!(),
    }
}

#[derive(Debug)]
pub struct Statement {
    pub label: Option<String>,
    pub mnemonic: Mnemonic,
    pub condition: Condition,
    pub s_flag: bool,
    pub operands: Vec<Operand>,
    pub line: usize,
}

pub fn parse_statement(input: &str, line: usize) -> Result<Statement, AsmError> {
    let original = input;
    match parse_statement_inner(input) {
        Ok((_, mut stmt)) => {
            stmt.line = line;
            Ok(stmt)
        }
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            let remaining = e.input;
            let col = original.len() - remaining.len();
            let snippet = &remaining[..remaining.len().min(20)];
            Err(AsmError::ParseError {
                line,
                col,
                message: format!("unexpected '{}'", snippet),
            })
        }
        Err(nom::Err::Incomplete(_)) => Err(AsmError::ParseError {
            line,
            col: 0,
            message: "incomplete input".to_string(),
        }),
    }
}

fn parse_statement_inner(input: &str) -> IResult<&str, Statement> {
    let (input, _) = sp(input)?;
    let (input, label) = opt(terminated(label_name, (sp, char(':'), sp))).parse(input)?;
    let (input, _) = sp(input)?;

    if input.is_empty() {
        return Ok((
            input,
            Statement {
                label,
                mnemonic: Mnemonic::LabelOnly,
                condition: Condition::Al,
                s_flag: false,
                operands: vec![],
                line: 0,
            },
        ));
    }

    if let Ok((after_dot, _)) = char::<&str, nom::error::Error<&str>>('.').parse(input) {
        let (rest, dir_name) = take_while1(|c: char| c.is_alphabetic() || c == '_')(after_dot)?;
        let (rest, _) = sp(rest)?;
        match dir_name {
            "global" | "globl" => {
                let (rest, lbl) = label_name(rest)?;
                return Ok((
                    rest,
                    Statement {
                        label,
                        mnemonic: Mnemonic::Global,
                        condition: Condition::Al,
                        s_flag: false,
                        operands: vec![Operand::Label(lbl)],
                        line: 0,
                    },
                ));
            }
            "text" | "data" => {
                let mnem = if dir_name == "text" {
                    Mnemonic::Text
                } else {
                    Mnemonic::Data
                };
                return Ok((
                    rest,
                    Statement {
                        label,
                        mnemonic: mnem,
                        condition: Condition::Al,
                        s_flag: false,
                        operands: vec![],
                        line: 0,
                    },
                ));
            }
            "align" => {
                let (rest, val) = immediate(rest)?;
                return Ok((
                    rest,
                    Statement {
                        label,
                        mnemonic: Mnemonic::Align,
                        condition: Condition::Al,
                        s_flag: false,
                        operands: vec![Operand::Imm(val)],
                        line: 0,
                    },
                ));
            }
            "ascii" | "asciz" => {
                let (rest, string_val) = string_literal(rest)?;
                let mnem = if dir_name == "ascii" {
                    Mnemonic::Ascii
                } else {
                    Mnemonic::Asciz
                };
                return Ok((
                    rest,
                    Statement {
                        label,
                        mnemonic: mnem,
                        condition: Condition::Al,
                        s_flag: false,
                        operands: vec![Operand::StringBytes(string_val.into_bytes())],
                        line: 0,
                    },
                ));
            }
            "float" => {
                let (rest, val) = float_literal(rest)?;
                return Ok((
                    rest,
                    Statement {
                        label,
                        mnemonic: Mnemonic::Float,
                        condition: Condition::Al,
                        s_flag: false,
                        operands: vec![Operand::Float(val)],
                        line: 0,
                    },
                ));
            }
            "word" | "long" => {
                let (rest, op) = alt((
                    map(immediate, Operand::Imm),
                    map(label_name, Operand::Label),
                ))
                .parse(rest)?;
                return Ok((
                    rest,
                    Statement {
                        label,
                        mnemonic: Mnemonic::Word,
                        condition: Condition::Al,
                        s_flag: false,
                        operands: vec![op],
                        line: 0,
                    },
                ));
            }
            _ => {
                return Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )));
            }
        }
    }

    let (input, info) = parse_mnemonic_with_modifiers(input)?;
    let (input, _) = sp(input)?;
    let (input, operands) = parse_operands_for_mnemonic(input, &info)?;
    let (input, _) = sp(input)?;

    Ok((
        input,
        Statement {
            label,
            mnemonic: info.op,
            condition: info.condition,
            s_flag: info.set_flags,
            operands,
            line: 0,
        },
    ))
}
