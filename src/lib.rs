pub mod assembler;
mod encoder;
pub mod error;
mod parser;
pub mod resolver;

pub mod prelude {
    pub use crate::assembler::{AssemblerOptions, Endian, assemble, assemble_with_options};
    pub use crate::error::AsmError;
}

#[cfg(test)]
mod tests;
