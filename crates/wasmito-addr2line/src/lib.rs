use std::path::Path;

use addr2line::Location as Addr2LineLocation;
use wasm_tools::addr2line::Addr2lineModules;
use wat::GenerateDwarf;
use wat::Parser;

pub mod error;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Mapping {
    pub address: u64,
    pub range_size: u64,
    pub location: Location,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Location {
    /// The file name.
    pub file: Option<String>,
    /// The line number.
    pub line: Option<u32>,
    /// The column number.
    ///
    /// A value of `Some(0)` indicates the left edge.
    pub column: Option<u32>,
}

impl From<Addr2LineLocation<'_>> for Location {
    fn from(value: Addr2LineLocation<'_>) -> Self {
        Self {
            file: value.file.map(ToString::to_string),
            line: value.line,
            column: value.column,
        }
    }
}

/// Macro to append the current file, line and column to a `&'static str`
/// Example: "src/lib.rs:167:58"
macro_rules! location {
    () => {
        concat!(file!(), ":", line!(), ":", column!())
    };
}

pub(crate) struct CodeSectionInformation {
    pub(crate) start_offset: usize,
    pub(crate) size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module(Vec<u8>);

impl Module {
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        let Self(bytes) = self;
        bytes
    }

    /// # Errors
    /// In the case parsing fails, cf. <Error> on retrieving the error info.
    pub fn from_wat(path: Option<&Path>, wat: &str) -> Result<Self, error::WatParseError> {
        // Configure new parser with Dwarf support
        let mut parser = Parser::new();
        parser.generate_dwarf(GenerateDwarf::Full);

        // Parse the module, yield early if parsing fails
        let wat_module = parser
            .parse_str(path, wat)
            .map_err(|e| error::WatParseError(format!("{e:?}")))?;

        Ok(Self(wat_module))
    }

    /// # Errors
    /// In the case parsing fails, cf. <Error> on retrieving the error info.
    pub fn addr2line(&self, byte_address: u64) -> Result<Location, error::Error> {
        let Self(module) = self;
        let mut addr2line_modules = Addr2lineModules::parse(module)
            .map_err(|reason| error::Error::Wasmparser(reason.to_string()))?;

        let code_section_relative = false;
        let (ctx, text_relative_address) = addr2line_modules
            .context(byte_address, code_section_relative)
            .map_err(|reason| error::Error::ContextCreation1(reason.to_string()))?
            .ok_or_else(|| error::Error::ContextCreation2(Box::from(location!())))?;

        // Use text_relative_address here, not byte_offset!
        let outcome = ctx
            .find_location(text_relative_address)
            .map_err(|reason| error::Error::FindTextOffset1(reason.to_string()))?
            .ok_or_else(|| error::Error::FindTextOffset2(Box::from(location!())))?;

        Ok(outcome.into())
    }

    /// # Errors
    /// In the case parsing fails, cf. <Error> on retrieving the error info.
    pub fn mappings(&self) -> Result<Vec<Mapping>, error::Error> {
        let Self(module) = self;
        let mut addr2line_modules = Addr2lineModules::parse(module)
            .map_err(|reason| error::Error::Wasmparser(reason.to_string()))?;

        let code_section_relative = false;
        let CodeSectionInformation {
            start_offset: code_section_start_offset,
            size: code_section_size,
        } = self.determine_code_section_size()?;
        let (ctx, text_relative_address) = addr2line_modules
            .context(code_section_start_offset as u64, code_section_relative)
            .map_err(|reason| error::Error::ContextCreation1(reason.to_string()))?
            .ok_or_else(|| error::Error::ContextCreation2(Box::from(location!())))?;

        let mut mappings = vec![];

        for (address, range_size, location) in ctx
            .find_location_range(text_relative_address, code_section_size.into())
            .map_err(|reason| error::Error::FindTextOffset1(reason.to_string()))?
        {
            let location: Location = location.into();
            let mapping = Mapping {
                address: code_section_start_offset as u64 + address,
                range_size,
                location,
            };
            mappings.push(mapping);
        }

        Ok(mappings)
    }

    fn determine_code_section_size(&self) -> Result<CodeSectionInformation, error::Error> {
        let Self(module) = self;

        // Parse the module to find valid code offsets
        let parser = wasmparser::Parser::new(0);

        for payload in parser.parse_all(module) {
            let payload = match payload {
                Ok(payload) => payload,
                Err(reason) => return Err(error::Error::Wasmparser(reason.to_string())),
            };

            if let wasmparser::Payload::CodeSectionStart { size, range, .. } = payload {
                let info = CodeSectionInformation {
                    start_offset: range.start,
                    size,
                };
                return Ok(info);
            }
        }

        Err(error::Error::NoCodeSection)
    }
}
