use crate::error::AsmError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    SP,
    LR,
    PC,
}

impl Register {
    pub fn code(self) -> u32 {
        match self {
            Register::R0 => 0,
            Register::R1 => 1,
            Register::R2 => 2,
            Register::R3 => 3,
            Register::R4 => 4,
            Register::R5 => 5,
            Register::R6 => 6,
            Register::R7 => 7,
            Register::R8 => 8,
            Register::R9 => 9,
            Register::R10 => 10,
            Register::R11 => 11,
            Register::R12 => 12,
            Register::SP => 13,
            Register::LR => 14,
            Register::PC => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    EQ,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
}

impl Condition {
    pub fn code(self) -> u32 {
        match self {
            Condition::EQ => 0b0000,
            Condition::NE => 0b0001,
            Condition::CS => 0b0010,
            Condition::CC => 0b0011,
            Condition::MI => 0b0100,
            Condition::PL => 0b0101,
            Condition::VS => 0b0110,
            Condition::VC => 0b0111,
            Condition::HI => 0b1000,
            Condition::LS => 0b1001,
            Condition::GE => 0b1010,
            Condition::LT => 0b1011,
            Condition::GT => 0b1100,
            Condition::LE => 0b1101,
            Condition::AL => 0b1110,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShiftType {
    LSL,
    LSR,
    ASR,
    ROR,
    RRX,
}

impl ShiftType {
    pub fn code(self) -> u32 {
        match self {
            ShiftType::LSL => 0b00,
            ShiftType::LSR => 0b01,
            ShiftType::ASR => 0b10,
            ShiftType::ROR => 0b11,
            ShiftType::RRX => 0b11,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShifterOperand {
    Immediate(u32),
    Register(Register),
    ImmediateShift(Register, ShiftType, u32),
    RegisterShift(Register, ShiftType, Register),
    RRX(Register),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataOpcode {
    AND,
    EOR,
    SUB,
    RSB,
    ADD,
    ADC,
    SBC,
    RSC,
    TST,
    TEQ,
    CMP,
    CMN,
    ORR,
    MOV,
    BIC,
    MVN,
}

impl DataOpcode {
    pub fn code(self) -> u32 {
        match self {
            DataOpcode::AND => 0b0000,
            DataOpcode::EOR => 0b0001,
            DataOpcode::SUB => 0b0010,
            DataOpcode::RSB => 0b0011,
            DataOpcode::ADD => 0b0100,
            DataOpcode::ADC => 0b0101,
            DataOpcode::SBC => 0b0110,
            DataOpcode::RSC => 0b0111,
            DataOpcode::TST => 0b1000,
            DataOpcode::TEQ => 0b1001,
            DataOpcode::CMP => 0b1010,
            DataOpcode::CMN => 0b1011,
            DataOpcode::ORR => 0b1100,
            DataOpcode::MOV => 0b1101,
            DataOpcode::BIC => 0b1110,
            DataOpcode::MVN => 0b1111,
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    DataProcessing {
        cond: Condition,
        s: bool,
        opcode: DataOpcode,
        rd: Register,
        rn: Option<Register>,
        operand2: ShifterOperand,
    },
    LoadStore {
        cond: Condition,
        load: bool,
        byte: bool,
        rd: Register,
        addressing: AddressingMode,
    },
    Push {
        cond: Condition,
        reg_list: Vec<Register>,
    },
    Pop {
        cond: Condition,
        reg_list: Vec<Register>,
    },
    Multiply {
        cond: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        rm: Register,
    },
    Branch {
        cond: Condition,
        link: bool,
        offset: i32,
    },
    BranchExchange {
        cond: Condition,
        rm: Register,
    },
    Svc {
        cond: Condition,
        imm: u32,
    },
    RawWord(u32),
}

#[derive(Debug, Clone)]
pub enum AddressingMode {
    OffsetImmediate(Register, i32),
    OffsetRegister(Register, Register),
    OffsetScaled(Register, Register, ShiftType, u32),
}

fn encode_arm_immediate(value: u32) -> Option<(u8, u8)> {
    if value < 256 {
        return Some((value as u8, 0));
    }
    for rot in 1..=15 {
        let rotated = value.rotate_left(rot * 2);
        if rotated < 256 {
            return Some((rotated as u8, rot as u8));
        }
    }
    None
}

pub fn encode_instruction(instr: &Instruction) -> Result<u32, AsmError> {
    match instr {
        Instruction::DataProcessing {
            cond,
            s,
            opcode,
            rd,
            rn,
            operand2,
        } => {
            let cond_code = cond.code() << 28;
            let s_bit = if *s { 1 << 20 } else { 0 };
            let op_bits = opcode.code() << 21;
            let rn_code = rn.map(|r| r.code()).unwrap_or(0) << 16;
            let rd_code = rd.code() << 12;

            let (i_bit, op2_bits) = encode_shifter_operand(operand2, *rd, rn_code)?;

            Ok(cond_code | op_bits | s_bit | rn_code | rd_code | i_bit | op2_bits)
        }
        Instruction::LoadStore {
            cond,
            load,
            byte,
            rd,
            addressing,
        } => {
            let cond_code = cond.code() << 28;
            let l_bit = if *load { 1 << 20 } else { 0 };
            let b_bit = if *byte { 1 << 22 } else { 0 };
            let rd_code = rd.code() << 12;

            match addressing {
                AddressingMode::OffsetImmediate(rn, offset) => {
                    let rn_code = rn.code() << 16;
                    if *offset < -4095 || *offset > 4095 {
                        return Err(AsmError::ImmediateOutOfRange {
                            line: 0,
                            value: *offset as u32,
                        });
                    }
                    let u_bit = if *offset >= 0 { 1 << 23 } else { 0 };
                    let abs_offset = offset.unsigned_abs() & 0xFFF;
                    Ok(cond_code
                        | 0x05000000
                        | u_bit
                        | b_bit
                        | l_bit
                        | rn_code
                        | rd_code
                        | abs_offset)
                }
                AddressingMode::OffsetRegister(rn, rm) => {
                    let rn_code = rn.code() << 16;
                    let u_bit = 1 << 23;
                    Ok(cond_code
                        | 0x07000000
                        | u_bit
                        | b_bit
                        | l_bit
                        | rn_code
                        | rd_code
                        | rm.code())
                }
                AddressingMode::OffsetScaled(rn, rm, shift, imm) => {
                    let rn_code = rn.code() << 16;
                    let u_bit = 1 << 23;
                    let shift_imm = (*imm & 0x1F) << 7;
                    let shift_code = shift.code() << 5;
                    Ok(cond_code
                        | 0x07000000
                        | u_bit
                        | shift_imm
                        | shift_code
                        | b_bit
                        | l_bit
                        | rn_code
                        | rd_code
                        | rm.code())
                }
            }
        }
        Instruction::Push { cond, reg_list } => {
            let cond_code = cond.code() << 28;
            // Optimize single register push to STR Rd,[SP, #-4]!
            if reg_list.len() == 1 {
                let rd_code = reg_list[0].code() << 12;
                Ok(cond_code | 0x052D0004 | rd_code)
            } else {
                let mut mask = 0;
                for r in reg_list {
                    mask |= 1 << r.code();
                }
                Ok(cond_code | 0x092D0000 | mask)
            }
        }
        Instruction::Pop { cond, reg_list } => {
            let cond_code = cond.code() << 28;
            // Optimize single register pop to LDR Rd,[SP], #4
            if reg_list.len() == 1 {
                let rd_code = reg_list[0].code() << 12;
                Ok(cond_code | 0x049D0004 | rd_code)
            } else {
                let mut mask = 0;
                for r in reg_list {
                    mask |= 1 << r.code();
                }
                Ok(cond_code | 0x08BD0000 | mask)
            }
        }
        Instruction::Multiply {
            cond,
            s,
            rd,
            rn,
            rm,
        } => {
            let cond_code = cond.code() << 28;
            let s_bit = if *s { 1 << 20 } else { 0 };
            Ok(cond_code | s_bit | (rd.code() << 16) | (rm.code() << 8) | 0x90 | rn.code())
        }
        Instruction::Branch { cond, link, offset } => {
            let cond_code = cond.code() << 28;
            let l_bit = if *link { 1 << 24 } else { 0 };
            let imm24 = (*offset as u32) & 0x00FF_FFFF;
            Ok(cond_code | 0x0A000000 | l_bit | imm24)
        }
        Instruction::BranchExchange { cond, rm } => {
            let cond_code = cond.code() << 28;
            Ok(cond_code | 0x12FFF10 | rm.code())
        }
        Instruction::Svc { cond, imm } => {
            let cond_code = cond.code() << 28;
            Ok(cond_code | 0x0F000000 | (*imm & 0x00FFFFFF))
        }
        Instruction::RawWord(val) => Ok(*val),
    }
}

fn encode_shifter_operand(
    so: &ShifterOperand,
    _rd: Register,
    _rn_bits: u32,
) -> Result<(u32, u32), AsmError> {
    match so {
        ShifterOperand::Immediate(imm) => {
            let (imm8, rot) = encode_arm_immediate(*imm).ok_or(AsmError::ImmediateOutOfRange {
                line: 0,
                value: *imm,
            })?;
            Ok((1 << 25, (rot as u32) << 8 | imm8 as u32))
        }
        ShifterOperand::Register(rm) => Ok((0, rm.code())),
        ShifterOperand::ImmediateShift(rm, shift, amount) => {
            if *amount > 31 && *shift != ShiftType::RRX {
                return Err(AsmError::InvalidShift {
                    line: 0,
                    message: "shift amount 0-31".into(),
                });
            }
            let shift_code = shift.code();
            let shift_imm = if *shift == ShiftType::RRX {
                0
            } else {
                *amount & 0x1F
            };
            Ok((0, ((shift_imm << 7) | (shift_code << 5)) | rm.code()))
        }
        ShifterOperand::RegisterShift(rm, shift, rs) => {
            if *rs == Register::PC {
                return Err(AsmError::InvalidShift {
                    line: 0,
                    message: "PC not allowed as shift register".into(),
                });
            }
            let shift_code = shift.code();
            Ok((0, (rs.code() << 8) | (shift_code << 5) | 0x10 | rm.code()))
        }
        ShifterOperand::RRX(rm) => Ok((0, (0b11 << 5) | rm.code())),
    }
}
