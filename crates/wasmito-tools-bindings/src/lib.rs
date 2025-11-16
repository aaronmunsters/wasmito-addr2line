use std::path::PathBuf;

use wasm_bindgen::prelude::*;

use wasmito_addr2line::Location as CoreLocation;
use wasmito_addr2line::Mapping as CoreMapping;
use wasmito_addr2line::MappingWithInstructions as CoreMappingWithInstructions;
use wasmito_addr2line::Module as CoreModule;
use wasmito_addr2line::PositionedInstruction as CorePositionedInstruction;

use wasmito_addr2line::instruction::Instruction;
use wasmito_strip::Config as CoreStripConfig;

#[wasm_bindgen]
#[derive(derive_more::Deref)]
pub struct Mapping(CoreMapping);

#[wasm_bindgen]
impl Mapping {
    #[must_use]
    pub fn address(&self) -> u64 {
        self.address
    }

    #[must_use]
    pub fn range_size(&self) -> u64 {
        self.range_size
    }

    #[must_use]
    pub fn file(&self) -> Option<String> {
        self.location.file.clone()
    }

    #[must_use]
    pub fn line(&self) -> Option<u32> {
        self.location.line
    }

    #[must_use]
    pub fn column(&self) -> Option<u32> {
        self.location.column
    }
}

#[wasm_bindgen]
#[derive(derive_more::Deref)]
pub struct MappingIncludingOffset(CoreMappingWithInstructions);

#[wasm_bindgen]
impl MappingIncludingOffset {
    #[must_use]
    pub fn address(&self) -> u64 {
        self.address_range.start
    }

    #[must_use]
    pub fn range_size(&self) -> u64 {
        self.address_range.end - self.address_range.start
    }

    #[must_use]
    pub fn file(&self) -> Option<String> {
        self.location.file.clone()
    }

    #[must_use]
    pub fn line(&self) -> Option<u32> {
        self.location.line
    }

    #[must_use]
    pub fn column(&self) -> Option<u32> {
        self.location.column
    }

    #[must_use]
    pub fn instructions(&self) -> Vec<PositionedInstruction> {
        self.instructions
            .iter()
            .map(|instruction| PositionedInstruction(instruction.clone()))
            .collect()
    }
}

#[wasm_bindgen]
#[derive(derive_more::Deref)]
pub struct PositionedInstruction(CorePositionedInstruction);

#[wasm_bindgen]
impl PositionedInstruction {
    #[must_use]
    pub fn address(&self) -> usize {
        self.address
    }

    #[must_use]
    pub fn instr(&self) -> String {
        match &self.instr {
            Instruction::Body(instr) => instr.to_wat_instr().to_string(),
            Instruction::Local(local) => format!("{local:?}"),
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Hash, PartialEq, Eq, derive_more::Deref)]
pub struct Location(CoreLocation);

#[wasm_bindgen]
impl Location {
    #[must_use]
    pub fn file(&self) -> Option<String> {
        self.file.clone()
    }

    #[must_use]
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    #[must_use]
    pub fn column(&self) -> Option<u32> {
        self.column
    }
}

#[wasm_bindgen]
#[derive(Debug, thiserror::Error)]
#[error("ParseError({0})")]
pub struct ParseError(wasmito_addr2line::error::WatParseError);

#[wasm_bindgen]
impl ParseError {
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
    #[must_use]
    pub fn context(&self) -> String {
        let Self(reason) = self;
        format!("{reason:?}")
    }
}

#[wasm_bindgen]
#[derive(derive_more::Deref)]
pub struct Module(CoreModule);

#[wasm_bindgen]
impl Module {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(CoreModule::new(bytes))
    }

    #[must_use]
    pub fn bytes(&self) -> Vec<u8> {
        let Self(module) = self;
        module.bytes().to_vec()
    }

    /// # Errors
    /// In the case parsing fails, cf. [`ParseError`] on retrieving the error info.
    #[allow(clippy::needless_pass_by_value)] // reason: wasm_bindgen + lifetimes
    pub fn from_wat(path: Option<String>, wat: &str) -> Result<Self, ParseError> {
        let module = CoreModule::from_wat(path.as_deref().map(PathBuf::from).as_deref(), wat)
            .map_err(ParseError)?;
        Ok(Self(module))
    }

    /// # Errors
    /// In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
    pub fn addr2line(&self, byte_offset: u64) -> Result<Location, Addr2lineError> {
        let Self(module) = self;
        module
            .addr2line(byte_offset)
            .map(Location)
            .map_err(Addr2lineError)
    }

    /// # Errors
    /// In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
    pub fn addr2line_mappings(&self) -> Result<Vec<Mapping>, Addr2lineError> {
        let Self(module) = self;
        let mappings = module.mappings().map_err(Addr2lineError)?;
        Ok(mappings.into_iter().map(Mapping).collect())
    }

    /// # Errors
    /// In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
    pub fn files(&self) -> Result<Vec<String>, Addr2lineError> {
        let Self(module) = self;
        module
            .files()
            .map(|files| files.into_iter().collect())
            .map_err(Addr2lineError)
    }

    /// # Errors
    /// In the case mapping fails, cf. [`Addr2lineError`] on retrieving the error info.
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
    pub fn addr2line_mappings_with_offsets(
        &self,
    ) -> Result<Vec<MappingIncludingOffset>, Addr2lineError> {
        let Self(module) = self;
        let mappings = module
            .mappings_including_instruction_offsets()
            .map_err(Addr2lineError)?;
        Ok(mappings.into_iter().map(MappingIncludingOffset).collect())
    }
}

#[wasm_bindgen]
#[derive(Debug, thiserror::Error)]
#[error("StripError({0})")]
pub struct StripError(wasmito_strip::error::Error);

#[wasm_bindgen]
impl StripError {
    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn context(&self) -> String {
        let Self(reason) = self;
        format!("{reason:?}")
    }
}

#[wasm_bindgen]
pub struct StripConfig(CoreStripConfig);

#[wasm_bindgen]
impl StripConfig {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(all: bool, to_delete: Vec<String>) -> Self {
        Self(CoreStripConfig::new(all, to_delete))
    }

    /// # Errors
    /// In the case parsing fails, cf. [`StripError`] on retrieving the error info.
    pub fn strip(&self, module: Vec<u8>) -> Result<Vec<u8>, StripError> {
        let Self(config) = self;
        config.strip(module).map_err(StripError)
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
