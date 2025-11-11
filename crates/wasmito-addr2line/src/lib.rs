use std::collections::HashSet as Set;
use std::ops::Range;
use std::path::Path;

use addr2line::Location as Addr2LineLocation;
use wasm_tools::addr2line::Addr2lineModules;
use wasmparser::BinaryReaderError;
use wat::GenerateDwarf;
use wat::Parser;

pub mod error;
pub mod instruction;

use instruction::{BodyInstruction, Instruction, ValType};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Mapping {
    pub address: u64,
    pub range_size: u64,
    pub location: Location,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PositionedInstruction {
    pub address: usize,
    pub instr: Instruction,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MappingWithInstructions {
    pub address_range: Range<u64>,
    pub instructions: Vec<PositionedInstruction>,
    pub location: Location,
}

impl MappingWithInstructions {
    fn new(mapping: Mapping) -> Self {
        let Mapping {
            address,
            range_size,
            location,
        } = mapping;
        Self {
            address_range: address..(address + range_size),
            instructions: vec![],
            location,
        }
    }
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

pub(crate) enum CodeSectionInformationOutcome {
    NoCodeSection,
    Some(CodeSectionInformation),
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
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
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
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
    pub fn mappings(&self) -> Result<Vec<Mapping>, error::Error> {
        let Self(module) = self;
        let mut addr2line_modules = Addr2lineModules::parse(module)
            .map_err(|reason| error::Error::Wasmparser(reason.to_string()))?;

        let CodeSectionInformation {
            start_offset: code_section_start_offset,
            size: code_section_size,
        } = match self.determine_code_section_size()? {
            CodeSectionInformationOutcome::NoCodeSection => return Ok(vec![]),
            CodeSectionInformationOutcome::Some(code_section_information) => {
                code_section_information
            }
        };

        let code_section_start_offset: u64 = code_section_start_offset
            .try_into()
            .map_err(error::Error::Cast)?;
        let (ctx, text_relative_address) = addr2line_modules
            .context(code_section_start_offset, false)
            .map_err(|reason| error::Error::ContextCreation1(reason.to_string()))?
            .ok_or_else(|| error::Error::ContextCreation2(Box::from(location!())))?;

        let mut mappings = vec![];

        for (address, range_size, location) in ctx
            .find_location_range(text_relative_address, code_section_size.into())
            .map_err(|reason| error::Error::FindTextOffset1(reason.to_string()))?
        {
            let location: Location = location.into();
            let mapping = Mapping {
                // FIXME: why is the `+ 1` required for the instruction offsets to match debugging info?
                address: code_section_start_offset + address + 1,
                range_size,
                location,
            };
            mappings.push(mapping);
        }

        Ok(mappings)
    }

    /// Retrieves the source files that were used during compilation.
    ///
    /// # Errors
    /// In the case parsing fails, cf. <Error> on retrieving the error info.
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
    pub fn files(&self) -> Result<Set<String>, error::Error> {
        let mappings = self.mappings()?;
        let files = mappings
            .into_iter()
            .filter_map(|mapping| mapping.location.file)
            .collect();
        Ok(files)
    }

    /// # Errors
    /// In the case parsing fails, cf. <Error> on retrieving the error info.
    ///
    /// # Note
    /// Cache successive calls to this method, its result does not change.
    pub fn mappings_including_instruction_offsets(
        &self,
    ) -> Result<Vec<MappingWithInstructions>, error::Error> {
        let mappings: Vec<_> = self
            .mappings()?
            .into_iter()
            .map(MappingWithInstructions::new)
            .collect();

        self.compute_instruction_offsets(mappings)
            .map_err(error::Error::Binary)
    }

    fn compute_instruction_offsets(
        &self,
        mut mappings: Vec<MappingWithInstructions>,
    ) -> Result<Vec<MappingWithInstructions>, BinaryReaderError> {
        // Parse the module to find valid code offsets
        let Self(module) = self;
        let parser = wasmparser::Parser::default();

        for payload in parser.parse_all(module) {
            let payload = payload?;
            if let wasmparser::Payload::CodeSectionEntry(ref function_body) = payload {
                let mut function_locals_reader = function_body.get_locals_reader()?;
                for _ in 0..function_locals_reader.get_count() {
                    let address = function_locals_reader.original_position() as _;
                    let (count, local_type) = function_locals_reader.read()?;
                    for mapping in &mut mappings {
                        if mapping.address_range.contains(&(address as u64)) {
                            let ty = ValType::from(local_type);
                            let instr = Instruction::new_local(count, ty);
                            mapping
                                .instructions
                                .push(PositionedInstruction { address, instr });
                        }
                    }
                }

                for operator_offset in function_body
                    .get_operators_reader()?
                    .into_iter_with_offsets()
                {
                    let (operator, address) = operator_offset?;
                    for mapping in &mut mappings {
                        if mapping.address_range.contains(&(address as u64)) {
                            let instr = Instruction::new_body(BodyInstruction::from(&operator));
                            let instr = PositionedInstruction { address, instr };
                            mapping.instructions.push(instr);
                        }
                    }
                }
            }
        }

        Ok(mappings)
    }

    fn determine_code_section_size(&self) -> Result<CodeSectionInformationOutcome, error::Error> {
        let Self(module) = self;

        // Parse the module to find valid code offsets
        let parser = wasmparser::Parser::new(0);

        for payload in parser.parse_all(module) {
            let payload = payload.map_err(|reason| error::Error::Wasmparser(reason.to_string()))?;

            if let wasmparser::Payload::CodeSectionStart { size, range, .. } = payload {
                let info = CodeSectionInformation {
                    start_offset: range.start,
                    size,
                };
                return Ok(CodeSectionInformationOutcome::Some(info));
            }
        }

        Ok(CodeSectionInformationOutcome::NoCodeSection)
    }
}
