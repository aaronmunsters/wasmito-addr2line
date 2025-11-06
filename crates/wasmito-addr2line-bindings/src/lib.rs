use std::path::PathBuf;

use wasm_bindgen::prelude::*;
use wasmito_addr2line::Location as CoreLocation;
use wasmito_addr2line::Mapping as CoreMapping;
use wasmito_addr2line::Module as CoreModule;

#[wasm_bindgen]
pub struct Mapping(CoreMapping);

#[wasm_bindgen]
impl Mapping {
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn address(&self) -> u64 {
        let Self(mapping) = self;
        mapping.address
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn range_size(&self) -> u64 {
        let Self(mapping) = self;
        mapping.range_size
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn file(&self) -> Option<String> {
        let Self(mapping) = self;
        mapping.location.file.clone()
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn line(&self) -> Option<u32> {
        let Self(mapping) = self;
        mapping.location.line
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn column(&self) -> Option<u32> {
        let Self(mapping) = self;
        mapping.location.column
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Location(CoreLocation);

#[wasm_bindgen]
impl Location {
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn file(&self) -> Option<String> {
        let Self(location) = self;
        location.file.clone()
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn line(&self) -> Option<u32> {
        let Self(location) = self;
        location.line
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn column(&self) -> Option<u32> {
        let Self(location) = self;
        location.column
    }
}

#[wasm_bindgen]
#[derive(Debug, thiserror::Error)]
#[error("ParseError({0})")]
pub struct ParseError(wasmito_addr2line::error::WatParseError);

#[wasm_bindgen]
impl ParseError {
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn context(&self) -> String {
        let Self(reason) = self;
        format!("{reason:?}")
    }
}

#[wasm_bindgen]
#[derive(Debug, thiserror::Error)]
#[error("ParseError({0})")]
pub struct Addr2lineError(wasmito_addr2line::error::Error);

#[wasm_bindgen]
impl Addr2lineError {
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn context(&self) -> String {
        let Self(reason) = self;
        format!("{reason:?}")
    }
}

#[wasm_bindgen]
pub struct Module(CoreModule);

#[wasm_bindgen]
impl Module {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(CoreModule::new(bytes))
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let Self(module) = self;
        module.bytes().to_vec()
    }

    /// # Errors
    /// In the case parsing fails, cf. <Error> on retrieving the error info.
    #[wasm_bindgen]
    #[allow(clippy::needless_pass_by_value)] // reason: wasm_bindgen + lifetimes
    pub fn from_wat(path: Option<String>, wat: &str) -> Result<Self, ParseError> {
        let module = CoreModule::from_wat(path.as_deref().map(PathBuf::from).as_deref(), wat)
            .map_err(ParseError)?;
        Ok(Self(module))
    }

    /// # Errors
    /// In the case mapping fails, cf. <Error> on retrieving the error info.
    #[wasm_bindgen]
    pub fn addr2line(&self, byte_offset: u64) -> Result<Location, Addr2lineError> {
        let Self(module) = self;
        module
            .addr2line(byte_offset)
            .map(Location)
            .map_err(Addr2lineError)
    }

    /// # Errors
    /// In the case mapping fails, cf. <Error> on retrieving the error info.
    #[wasm_bindgen]
    pub fn addr2line_mappings(&self) -> Result<Vec<Mapping>, Addr2lineError> {
        let Self(module) = self;
        let mappings = module.mappings().map_err(Addr2lineError)?;
        Ok(mappings.into_iter().map(Mapping).collect())
    }
}

// Registration for `getrandom` crate to compile to wasm32-unknown-unknown
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(
    dest: *mut u8,
    len: usize,
) -> Result<(), getrandom::Error> {
    // Create a slice from the raw pointer and fill it with zeros
    let slice = unsafe { core::slice::from_raw_parts_mut(dest, len) };
    slice.fill(0);
    Ok(())
}
