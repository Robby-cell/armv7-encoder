//! JS bindings exposed only when the `wasm` feature is enabled.

use js_sys::Function;
use wasm_bindgen::prelude::*;

use crate::assembler::{AssemblerOptions, Encoder as CoreEncoder, Endian as AV7Endian, assemble};
use crate::resolver::{NoSymbolResolver, SymbolResolver};

/// Endianness options natively exposed to WASM.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Endian {
    Little = 0,
    Big = 1 << 30,
}

impl From<Endian> for AV7Endian {
    fn from(e: Endian) -> Self {
        match e {
            Endian::Little => AV7Endian::Little,
            Endian::Big => AV7Endian::Big,
        }
    }
}

/// A bridge to allow JS Functions to resolve assembly symbols on the fly.
struct JsSymbolResolver {
    js_func: Option<Function>,
}

impl SymbolResolver for JsSymbolResolver {
    fn resolve(&self, name: &str) -> Option<u32> {
        if let Some(func) = &self.js_func {
            let arg = JsValue::from_str(name);
            if let Ok(res) = func.call1(&JsValue::NULL, &arg) {
                // Return None if the JS function explicitly returns null/undefined
                if !res.is_null() && !res.is_undefined() {
                    return res.as_f64().map(|v| v as u32);
                }
            }
        }
        None
    }
}

/// An object-oriented Assembler Encoder exposed to JavaScript.
#[wasm_bindgen]
pub struct Encoder {
    start_address: u32,
    endian: Endian,
    resolver_func: Option<Function>,
}

#[wasm_bindgen]
impl Encoder {
    /// Create a new Encoder.
    ///
    /// The `resolver_func` parameter expects a JavaScript callback taking a string
    /// and returning a number (or null/undefined if unknown).
    /// Example: `(name) => name === "led0" ? 0x40000000 : null`
    #[wasm_bindgen(constructor)]
    pub fn new(
        start_address: Option<u32>,
        endian: Option<Endian>,
        resolver_func: Option<Function>,
    ) -> Self {
        Self {
            start_address: start_address.unwrap_or(0),
            endian: endian.unwrap_or(Endian::Little),
            resolver_func,
        }
    }

    /// Assemble the provided ARMv7 assembly source using this object's internal state.
    /// Returns a JavaScript `Uint8Array`.
    #[wasm_bindgen]
    pub fn assemble(&self, source: &str) -> Result<Vec<u8>, JsError> {
        let resolver = JsSymbolResolver {
            js_func: self.resolver_func.clone(),
        };

        let options = AssemblerOptions {
            start_address: self.start_address,
            endian: self.endian.into(),
            symbol_resolver: Box::new(resolver),
        };

        let core_encoder = CoreEncoder::new(options);
        core_encoder
            .assemble(source)
            .map_err(|e| JsError::new(&e.to_string()))
    }
}

/// Standalone function: Assemble ARMv7 (A32) assembly text into a byte array (Little‑Endian).
#[wasm_bindgen]
pub fn assemble_armv7(source: &str) -> Result<Vec<u8>, JsError> {
    assemble(source).map_err(|e| JsError::new(&e.to_string()))
}

/// Standalone function: Assemble ARMv7 (A32) assembly text into a byte array (Big‑Endian).
#[wasm_bindgen]
pub fn assemble_armv7_big_endian(source: &str) -> Result<Vec<u8>, JsError> {
    let options = AssemblerOptions {
        start_address: 0,
        endian: AV7Endian::Big,
        symbol_resolver: Box::new(NoSymbolResolver),
    };

    let core_encoder = CoreEncoder::new(options);
    core_encoder
        .assemble(source)
        .map_err(|e| JsError::new(&e.to_string()))
}
