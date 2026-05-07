pub mod assembler;
mod encoder;
pub mod error;
mod parser;
pub mod resolver;

pub mod prelude;

// Only include the WASM bindings when the feature is active
#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(test)]
mod tests;
