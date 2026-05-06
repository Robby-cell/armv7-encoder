use core::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AsmError {
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },
    ImmediateOutOfRange {
        line: usize,
        value: u32,
    },
    UndefinedLabel {
        line: usize,
        label: String,
    },
    InvalidRegister {
        line: usize,
        name: String,
    },
    UnknownMnemonic {
        line: usize,
        mnemonic: String,
    },
    InvalidShift {
        line: usize,
        message: String,
    },
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsmError::ParseError { line, col, message } => {
                write!(f, "line {}:{}: {}", line, col, message)
            }
            AsmError::ImmediateOutOfRange { line, value } => write!(
                f,
                "line {}: immediate 0x{:X} cannot be encoded",
                line, value
            ),
            AsmError::UndefinedLabel { line, label } => {
                write!(f, "line {}: undefined label '{}'", line, label)
            }
            AsmError::InvalidRegister { line, name } => {
                write!(f, "line {}: invalid register '{}'", line, name)
            }
            AsmError::UnknownMnemonic { line, mnemonic } => {
                write!(f, "line {}: unknown mnemonic '{}'", line, mnemonic)
            }
            AsmError::InvalidShift { line, message } => {
                write!(f, "line {}: invalid shift: {}", line, message)
            }
        }
    }
}

impl core::error::Error for AsmError {}
