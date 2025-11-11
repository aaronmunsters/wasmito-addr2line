use std::collections::HashSet;

use anyhow::Result;
use wasmito_addr2line::{Module, instruction::Instruction};

const WAT: &str = r#"
(module
  (import "even" "even" (func $even (param i32) (result i32)))
  (export "odd" (func $odd))
  (func $odd (param $0 i32) (result i32)
    local.get $0
    i32.eqz
    if
    i32.const 0
    return
    end
    local.get $0
    i32.const 1
    i32.sub
    call $even))
"#;

#[test]
fn fast_addresses_work() -> Result<()> {
    let mapped_module = Module::from_wat(None, WAT)?;
    let mapping = mapped_module.mappings()?;
    let target = &mapping[5];

    assert_eq!(target.address, 57);
    assert_eq!(target.range_size, 1);

    assert_eq!(target.location.file.as_ref().unwrap(), "./<input>.wat");
    assert_eq!(target.location.line.unwrap(), 11);
    assert_eq!(target.location.column.unwrap(), 5);

    assert_eq!(target.location.file, Some("./<input>.wat".into()));
    assert_eq!(target.location.line, Some(11));
    assert_eq!(target.location.column, Some(5));

    Ok(())
}

#[test]
fn fast_files_work() -> Result<()> {
    let mapped_module = Module::from_wat(None, WAT)?;
    let files = mapped_module.files()?;
    assert_eq!(files.iter().collect::<Vec<_>>(), vec!["./<input>.wat"]);
    Ok(())
}

#[test]
fn single_address_works() -> Result<()> {
    let mapped_module = Module::from_wat(None, WAT)?;
    let address_too_low = 0;
    let address_too_high = u64::MAX;
    assert!(mapped_module.addr2line(address_too_low).is_err());
    assert!(mapped_module.addr2line(address_too_high).is_err());

    let location = mapped_module.addr2line(57)?;
    assert_eq!(location.file.unwrap(), "./<input>.wat".to_string());
    assert_eq!(location.line.unwrap(), 12);
    assert_eq!(location.column.unwrap(), 5);
    Ok(())
}

#[test]
fn test_from_c_works() -> Result<()> {
    let module = include_bytes!("./example_from_c.wasm");
    let module = Module::new(module.into());
    let files = module.files()?;

    let expected_files = [
        "/emsdk/emscripten/system/lib/libc/crt1.c",
        "/emsdk/emscripten/system/lib/libc/musl/src/errno/__errno_location.c",
        "/xxxxx/xxxxxxxxxxxxxxxxxxx/xxxxxxxx/xxxxxxxxxxxxxxxx/xxxxxxxxxxxxxxxxx/path/to/source/code/lib.c",
        "/emsdk/emscripten/system/lib/libc/musl/src/exit/_Exit.c",
        "/emsdk/emscripten/system/lib/libc/musl/src/exit/exit.c",
    ].iter().map(std::string::ToString::to_string).collect::<HashSet<_>>();
    assert_eq!(files, expected_files);

    assert!(
        module
            .mappings_including_instruction_offsets()?
            .iter()
            .any(|mapping| {
                mapping
                    .instructions
                    .iter()
                    .any(|instr| matches!(instr.instr, Instruction::Local(_)))
            })
    );
    Ok(())
}
