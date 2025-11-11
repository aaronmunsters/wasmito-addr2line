use std::{fs::File, io::Write, process::Command};

use wasmito_addr2line::instruction::{BodyInstruction, Instruction};
use wasmito_addr2line::{Module, PositionedInstruction};

use anyhow::{Result, bail};
use arbitrary::Unstructured;
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use wasm_smith::{Config as WasmSmithConfig, Module as WasmSmithModule};

const MAX_SEED: u64 = 1000;
const MAX_PRGS: usize = 2_usize.pow(14);

#[test]
fn test_with_smith() {
    let total_mappings_count = (0..MAX_SEED)
        .into_par_iter()
        .fold(
            || 0_usize,
            |acc, seed| {
                let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
                let mut random_sequence = [0_u8; MAX_PRGS];
                rng.fill(&mut random_sequence);
                let mut random = Unstructured::new(&random_sequence);
                let module = WasmSmithModule::new(WasmSmithConfig::default(), &mut random).unwrap();
                let module_bytes = module.to_bytes();
                let mappings_count = round_trip_wasm_to_wat(module_bytes).unwrap();
                acc + mappings_count
            },
        )
        .sum::<usize>();
    println!("{total_mappings_count}");
    assert!(total_mappings_count > 10_000);
}

fn round_trip_wasm_to_wat(module_bytes: Vec<u8>) -> Result<usize> {
    let wat = Module::new(module_bytes).to_wat()?;
    let module = Module::from_wat(None, wat.as_str())?;
    let wat = module.to_wat()?;
    let mappings = module.mappings_including_instruction_offsets()?;

    let mappings_len = mappings.len();

    for mapping in mappings {
        for instruction in mapping.instructions {
            let PositionedInstruction {
                address: offset,
                instr,
            } = instruction;
            let Instruction::Body(instr) = instr else {
                continue;
            };

            let wat_instruction = instr.to_wat_instr();
            let line_key = format!("(;@{offset:x}");

            let wat_line = wat.split('\n').find(|line| line.contains(&line_key));

            let Some(wat_line) = wat_line else {
                bail!("{line_key:7} {{{wat_instruction:20}}} ({instr:20?}) -> {wat_line:?}");
            };

            if let BodyInstruction::End | BodyInstruction::LocalGet = instr {
                continue;
            }

            assert!(wat_line.contains(wat_instruction), "{wat}");
        }
    }

    Ok(mappings_len)
}

trait ToWat {
    fn to_wat(&self) -> Result<String>;
}

impl ToWat for Module {
    fn to_wat(&self) -> Result<String> {
        use tempfile::tempdir;
        let temp_dir = tempdir()?;
        let temp_file_path = temp_dir.path().join("temp.wasm");
        let mut temp_file = File::create(&temp_file_path)?;
        let temp_file_path_str = temp_file_path.as_os_str().display().to_string();
        temp_file.write_all(self.bytes())?;
        let result = Command::new("wasm-tools")
            .args(["print", &temp_file_path_str, "--print-offsets"])
            .output()?
            .stdout;
        String::from_utf8(result).map_err(Into::into)
    }
}
