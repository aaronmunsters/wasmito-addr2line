# Wasmito Addr2Line

Enables the use of functionality from `wasm-tools`, such as addr2line from JavaScript directly rather than going through a shell invocation.

## Example usage

```bash
# Using NPM
npm install github:aaronmunsters/wasmito-addr2line#pkg
```

After which you can call the module and use its functionality.
For example:

```js
import { Module, StripConfig } from "wasmito-addr2line";

const path = undefined; // can be "/path/to/your/module.wasm", used for generating DWARF
const wasm_module = Module.from_wat(path, `(module ...)`);
const mappings = wasm_module.addr2line_mappings();
// use mappings to inspect what source code maps to what range of bytecodes

const stripped = new StripConfig(true, []).strip(wasm_module.bytes);
// strip binaries from their custom sections to get a minimal binary
```

See [example.mjs](./example.mjs) for example assertions you can make use of.
In fact, you can `bash ./package.sh` and then `node example.mjs` to run that example file directly!
