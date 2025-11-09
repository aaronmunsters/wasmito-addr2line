use anyhow::Result;
use wasmito_addr2line::Module;

#[test]
fn fast_addresses_work() -> Result<()> {
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

    let mapped_module = Module::from_wat(None, WAT)?;
    let mapping = mapped_module.mappings()?;
    let target = &mapping[5];

    assert_eq!(target.address, 56);
    assert_eq!(target.range_size, 1);

    assert_eq!(target.location.file, Some("./<input>.wat".into()));
    assert_eq!(target.location.line, Some(11));
    assert_eq!(target.location.column, Some(11));

    Ok(())
}
