// replace "./pgk/..." with "wasmito-addr2line" after
// installing through `npm install github:aaronmunsters/wasmito-addr2line#pkg`
import { Module } from "./pkg/wasmito-addr2line.js";

const path = undefined;
const module = Module.from_wat(
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
  console.assert(module.bytes[index] === wasm_magic_bytes[index]);
}

const mappings = module.addr2line_mappings();

console.assert(mappings[5].address === BigInt(56));
console.assert(mappings[5].range_size === BigInt(1));
console.assert(mappings[5].file === "./<input>.wat");
console.assert(mappings[5].line === 11);
console.assert(mappings[5].column === 5);
