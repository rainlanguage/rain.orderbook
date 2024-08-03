const fs = require("fs");
const { execSync } = require("child_process");

const args = process.argv.slice(2);

if (args[0] === "check") {
  // exit early if already built, this for installing as npm dependency
  if (fs.existsSync("./dist")) {
    return;
  }
}

// rm dist if present
execSync("npm run rm-dist");
execSync("npm run rm-temp");

// create root esm.js and cjs.js files with their .d.ts
fs.writeFileSync(
  "./cjs.js",
  '"use strict";\n\nmodule.exports = require("./dist/cjs/index");\n'
);
fs.writeFileSync("./cjs.d.ts", 'export * from "./dist/cjs/index";\n');
fs.writeFileSync("./esm.js", 'export * from "./dist/esm/index";\n');
fs.writeFileSync("./esm.d.ts", 'export * from "./dist/esm/index";\n');

// create dist dir
fs.mkdirSync("./dist/cjs", { recursive: true });
fs.mkdirSync("./dist/esm", { recursive: true });

// build for wasm32 target
execSync(
  "nix develop -c cargo build --target wasm32-unknown-unknown --lib -r --workspace --exclude rain_orderbook_cli --exclude rain-orderbook-env"
);

// build specified packages and include them in final index file
const packagePrefix = "rain_orderbook_";
const packages = ["common"];
for (const package of packages) {
  // generate bindgen wasm
  execSync(
    `nix develop -c wasm-bindgen --target nodejs ./target/wasm32-unknown-unknown/release/${
      packagePrefix + package
    }.wasm --out-dir ./temp/node/${package} --out-name ${package}`
  );
  execSync(
    `nix develop -c wasm-bindgen --target web ./target/wasm32-unknown-unknown/release/${
      packagePrefix + package
    }.wasm --out-dir ./temp/web/${package} --out-name ${package}`
  );

  // encode wasm as base64 into a json that can be natively imported
  // in js modules in order to avoid using fetch or fs operations
  const wasmBytes = fs.readFileSync(
    `./temp/node/${package}/${package}_bg.wasm`
  );
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
  fs.writeFileSync(`./dist/cjs/${package}.d.ts`, dts);
  fs.writeFileSync(`./dist/esm/${package}.d.ts`, dts);

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
  cjs = cjs.replace(
    "const { TextDecoder, TextEncoder } = require(`util`);",
    ""
  );
  cjs = "/* this file is auto-generated, do not modify */\n" + cjs;
  fs.writeFileSync(`./dist/cjs/${package}.js`, cjs);

  // prepare esm
  let esm = fs.readFileSync(`./temp/web/${package}/${package}.js`, {
    encoding: "utf-8",
  });
  const index = esm.indexOf(
    "function __wbg_init_memory(imports, maybe_memory)"
  );
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
}

// create index files exporting all packages that was built in previous step
fs.writeFileSync(
  "./dist/cjs/index.js",
  `"use strict";
${packages.map((v) => `const ${v} = require("./${v}");`).join("\n")}

module.exports = {
    ${packages.map((v) => `...${v}`).join(",\n")}
};
`
);
fs.writeFileSync(
  "./dist/cjs/index.d.ts",
  packages.map((v) => `export * from "./${v}";`).join("\n")
);
fs.writeFileSync(
  "./dist/esm/index.js",
  packages.map((v) => `export * from "./${v}";`).join("\n")
);
fs.writeFileSync(
  "./dist/esm/index.d.ts",
  packages.map((v) => `export * from "./${v}";`).join("\n")
);

// rm temp folder
execSync("npm run lint-bindings");

// rm temp folder
execSync("nix develop -c npm run rm-temp");
