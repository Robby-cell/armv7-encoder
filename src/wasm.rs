//! JS bindings exposed only when the `wasm` feature is enabled.

use wasm_bindgen::prelude::*;

use crate::assembler::assemble;

/// Assemble ARMv7 (A32) assembly text into a byte array (little‑endian).
/// Returns a JavaScript `Uint8Array` on success, or throws an error string on failure.
#[wasm_bindgen]
pub fn assemble_armv7(source: &str) -> Result<Vec<u8>, JsError> {
    assemble(source).map_err(|e| JsError::new(&e.to_string()))
}
