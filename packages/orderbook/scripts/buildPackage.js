const fs = require("fs");
const { execSync } = require("child_process");

const packagePrefix = "rain_orderbook_";
const [package] = process.argv.slice(2);

// generate node/web bindgens
execSync(
  `wasm-bindgen --target nodejs ../../target/wasm32-unknown-unknown/release/${
    packagePrefix + package
  }.wasm --out-dir ./temp/node/${package} --out-name ${package}`
);
execSync(
  `wasm-bindgen --target web ../../target/wasm32-unknown-unknown/release/${
    packagePrefix + package
  }.wasm --out-dir ./temp/web/${package} --out-name ${package}`
);

// encode wasm as base64 into a json that can be natively imported
// in js modules in order to avoid using fetch or fs operations
const wasmBytes = fs.readFileSync(`./temp/node/${package}/${package}_bg.wasm`);
fs.writeFileSync(
  `./dist/${package}.json`,
  JSON.stringify({
    wasm: Buffer.from(wasmBytes, "binary").toString("base64"),
  })
);

// prepare the dts
let dts = fs.readFileSync(`./temp/node/${package}/${package}.d.ts`, {
  encoding: "utf-8",
});
dts = dts.replace(
  `/* tslint:disable */
/* eslint-disable */`,
  ""
);
dts = "/* this file is auto-generated, do not modify */\n" + dts;
fs.writeFileSync(`./dist/types/${package}.d.ts`, dts);

// prepare cjs
let cjs = fs.readFileSync(`./temp/node/${package}/${package}.js`, {
  encoding: "utf-8",
});
cjs = cjs.replace(
  `const path = require('path').join(__dirname, '${package}_bg.wasm');
const bytes = require('fs').readFileSync(path);`,
  `
const { Buffer } = require('buffer');
const wasmB64 = require('../${package}.json');
const bytes = Buffer.from(wasmB64.wasm, 'base64');`
);
cjs = cjs.replace("const { TextEncoder, TextDecoder } = require(`util`);", "");
cjs = "/* this file is auto-generated, do not modify */\n" + cjs;
fs.writeFileSync(`./dist/cjs/${package}.js`, cjs);

// prepare esm
let esm = fs.readFileSync(`./temp/web/${package}/${package}.js`, {
  encoding: "utf-8",
});
const index = esm.indexOf("function __wbg_init_memory(imports, memory)");
esm = esm.slice(0, index);
esm =
  "let imports = {};\n" +
  esm +
  `
imports = __wbg_get_imports();

import { Buffer } from 'buffer';
import wasmB64 from '../${package}.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;`;
esm = esm.replaceAll("imports.wbg", "imports.__wbindgen_placeholder__");
esm = "/* this file is auto-generated, do not modify */\n" + esm;
fs.writeFileSync(`./dist/esm/${package}.js`, esm);
