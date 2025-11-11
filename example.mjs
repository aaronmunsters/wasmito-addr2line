// replace "./pgk/..." with "wasmito-tools" after
// installing through `npm install github:aaronmunsters/wasmito-tools#pkg`
import { Module, StripConfig } from "./pkg/wasmito-tools.js";

const path = undefined;
const wasm_module = Module.from_wat(
  path,
  `
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
`,
);

const wasm_magic_bytes = [0x00, 0x61, 0x73, 0x6D];
for (let index; index < 4; index++) {
  console.assert(wasm_module.bytes[index] === wasm_magic_bytes[index]);
}

const mappings = wasm_module.addr2line_mappings();

console.assert(mappings[5].address === BigInt(57));
console.assert(mappings[5].range_size === BigInt(1));
console.assert(mappings[5].file === "./<input>.wat");
console.assert(mappings[5].line === 11);
console.assert(mappings[5].column === 5);

const module_including_dwarf = wasm_module;
const stripped = new StripConfig(true, []).strip(module_including_dwarf.bytes);
console.assert(stripped.length < module_including_dwarf.bytes.length);
