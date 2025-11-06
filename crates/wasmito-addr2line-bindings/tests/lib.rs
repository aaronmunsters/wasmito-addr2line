use anyhow::Result;
use wasmito_addr2line_bindings::Module;

#[test]
fn test_move_semantics() -> Result<()> {
    let module = Module::from_wat(
        None,
        r#"
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
        "#,
    )?;

    let location = module.addr2line(50)?;
    assert_eq!(location.column(), Some(17));
    assert_eq!(location.file(), Some("./<input>.wat".into()));
    assert_eq!(location.line(), Some(7));

    let bytes = module.bytes();
    assert_eq!(bytes.len(), 397);

    Ok(())
}
