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
    Sp,
    Lr,
    Pc,
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
            Register::Sp => 13,
            Register::Lr => 14,
            Register::Pc => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Condition {
    Eq,
    Ne,
    Cs,
    Cc,
    Mi,
    Pl,
    Vs,
    Vc,
    Hi,
    Ls,
    Ge,
    Lt,
    Gt,
    Le,
    Al,
}

impl Condition {
    pub fn code(self) -> u32 {
        match self {
            Condition::Eq => 0b0000,
            Condition::Ne => 0b0001,
            Condition::Cs => 0b0010,
            Condition::Cc => 0b0011,
            Condition::Mi => 0b0100,
            Condition::Pl => 0b0101,
            Condition::Vs => 0b0110,
            Condition::Vc => 0b0111,
            Condition::Hi => 0b1000,
            Condition::Ls => 0b1001,
            Condition::Ge => 0b1010,
            Condition::Lt => 0b1011,
            Condition::Gt => 0b1100,
            Condition::Le => 0b1101,
            Condition::Al => 0b1110,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
    Rrx,
}

impl ShiftType {
    pub fn code(self) -> u32 {
        match self {
            ShiftType::Lsl => 0b00,
            ShiftType::Lsr => 0b01,
            ShiftType::Asr => 0b10,
            ShiftType::Ror => 0b11,
            ShiftType::Rrx => 0b11,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ShifterOperand {
    Immediate(u32),
    Register(Register),
    ImmediateShift(Register, ShiftType, u32),
    RegisterShift(Register, ShiftType, Register),
    Rrx(Register),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataOpcode {
    And,
    Eor,
    Sub,
    Rsb,
    Add,
    Adc,
    Sbc,
    Rsc,
    Tst,
    Teq,
    Cmp,
    Cmn,
    Orr,
    Mov,
    Bic,
    Mvn,
}

impl DataOpcode {
    pub fn code(self) -> u32 {
        match self {
            DataOpcode::And => 0b0000,
            DataOpcode::Eor => 0b0001,
            DataOpcode::Sub => 0b0010,
            DataOpcode::Rsb => 0b0011,
            DataOpcode::Add => 0b0100,
            DataOpcode::Adc => 0b0101,
            DataOpcode::Sbc => 0b0110,
            DataOpcode::Rsc => 0b0111,
            DataOpcode::Tst => 0b1000,
            DataOpcode::Teq => 0b1001,
            DataOpcode::Cmp => 0b1010,
            DataOpcode::Cmn => 0b1011,
            DataOpcode::Orr => 0b1100,
            DataOpcode::Mov => 0b1101,
            DataOpcode::Bic => 0b1110,
            DataOpcode::Mvn => 0b1111,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtraLoadStoreOp {
    Strh,
    Ldrh,
    Ldrsb,
    Ldrsh,
    Strd,
    Ldrd,
}

impl ExtraLoadStoreOp {
    fn l_s_h(self) -> (u32, u32, u32) {
        match self {
            Self::Strh => (0, 0, 1),
            Self::Ldrh => (1, 0, 1),
            Self::Ldrd => (0, 1, 0),
            Self::Ldrsb => (1, 1, 0),
            Self::Strd => (0, 1, 1),
            Self::Ldrsh => (1, 1, 1),
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
    LoadStoreExtra {
        cond: Condition,
        op: ExtraLoadStoreOp,
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
    MultiplyAccumulate {
        cond: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        rm: Register,
        ra: Register,
    },
    MultiplySubtract {
        cond: Condition,
        rd: Register,
        rn: Register,
        rm: Register,
        ra: Register,
    },
    Divide {
        cond: Condition,
        unsigned: bool,
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
    Hint {
        cond: Condition,
        hint: u8,
    },
    Bkpt {
        imm: u16,
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
        Instruction::LoadStoreExtra {
            cond,
            op,
            rd,
            addressing,
        } => {
            let cond_code = cond.code() << 28;
            let rd_code = rd.code() << 12;
            let (l, s, h) = op.l_s_h();
            let l_bit = l << 20;
            let op_bits = (s << 6) | (h << 5) | (1 << 7) | (1 << 4);

            match addressing {
                AddressingMode::OffsetImmediate(rn, offset) => {
                    let rn_code = rn.code() << 16;
                    if *offset < -255 || *offset > 255 {
                        return Err(AsmError::ImmediateOutOfRange {
                            line: 0,
                            value: offset.unsigned_abs(),
                        });
                    }
                    let u_bit = if *offset >= 0 { 1 << 23 } else { 0 };
                    let abs_off = offset.unsigned_abs();
                    let imm4h = (abs_off >> 4) & 0xF;
                    let imm4l = abs_off & 0xF;

                    Ok(cond_code
                        | (1 << 24)
                        | u_bit
                        | (1 << 22)
                        | l_bit
                        | rn_code
                        | rd_code
                        | (imm4h << 8)
                        | op_bits
                        | imm4l)
                }
                AddressingMode::OffsetRegister(rn, rm) => {
                    let rn_code = rn.code() << 16;
                    let u_bit = 1 << 23;
                    Ok(cond_code
                        | (1 << 24)
                        | u_bit
                        | l_bit
                        | rn_code
                        | rd_code
                        | op_bits
                        | rm.code())
                }
                AddressingMode::OffsetScaled(..) => Err(AsmError::ParseError {
                    line: 0,
                    col: 0,
                    message: "Scaled offset not supported for extra load/store".into(),
                }),
            }
        }
        Instruction::Push { cond, reg_list } => {
            let cond_code = cond.code() << 28;
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
        Instruction::MultiplyAccumulate {
            cond,
            s,
            rd,
            rn,
            rm,
            ra,
        } => {
            let cond_code = cond.code() << 28;
            let s_bit = if *s { 1 << 20 } else { 0 };
            Ok(cond_code
                | 0x00200090
                | s_bit
                | (rd.code() << 16)
                | (ra.code() << 12)
                | (rm.code() << 8)
                | rn.code())
        }
        Instruction::MultiplySubtract {
            cond,
            rd,
            rn,
            rm,
            ra,
        } => {
            let cond_code = cond.code() << 28;
            Ok(cond_code
                | 0x00600090
                | (rd.code() << 16)
                | (ra.code() << 12)
                | (rm.code() << 8)
                | rn.code())
        }
        Instruction::Divide {
            cond,
            unsigned,
            rd,
            rn,
            rm,
        } => {
            let cond_code = cond.code() << 28;
            let base = if *unsigned { 0x0730F010 } else { 0x0710F010 };
            Ok(cond_code | base | (rd.code() << 16) | (rm.code() << 8) | rn.code())
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
        Instruction::Hint { cond, hint } => {
            let cond_code = cond.code() << 28;
            Ok(cond_code | 0x0320F000 | (*hint as u32))
        }
        Instruction::Bkpt { imm } => {
            let imm32 = *imm as u32;
            let imm12 = (imm32 >> 4) & 0xFFF;
            let imm4 = imm32 & 0xF;
            Ok(0xE1200070 | (imm12 << 8) | imm4)
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
            if *amount > 31 && *shift != ShiftType::Rrx {
                return Err(AsmError::InvalidShift {
                    line: 0,
                    message: "shift amount 0-31".into(),
                });
            }
            let shift_code = shift.code();
            let shift_imm = if *shift == ShiftType::Rrx {
                0
            } else {
                *amount & 0x1F
            };
            Ok((0, ((shift_imm << 7) | (shift_code << 5)) | rm.code()))
        }
        ShifterOperand::RegisterShift(rm, shift, rs) => {
            if *rs == Register::Pc {
                return Err(AsmError::InvalidShift {
                    line: 0,
                    message: "PC not allowed as shift register".into(),
                });
            }
            let shift_code = shift.code();
            Ok((0, (rs.code() << 8) | (shift_code << 5) | 0x10 | rm.code()))
        }
        ShifterOperand::Rrx(rm) => Ok((0, (0b11 << 5) | rm.code())),
    }
}
