const fs = require("fs");
const { execSync } = require("child_process");

// create root esm.js and cjs.js files with their .d.ts
fs.writeFileSync(
  "./cjs.js",
  '"use strict";\n\nmodule.exports = require("./dist/cjs/index");\n'
);
fs.writeFileSync("./cjs.d.ts", 'export * from "./dist/cjs/index";\n');
fs.writeFileSync("./esm.js", 'export * from "./dist/esm/index";\n');
fs.writeFileSync("./esm.d.ts", 'export * from "./dist/esm/index";\n');

// build for wasm32 target
execSync("npm run build-wasm");

// build specified packages and include them in final index file
// list of packages to build can be extended by adding new package
// names to the list below
const packages = ["wasm_async_compile_wrapper", "js_api"];

// create esm index
fs.mkdirSync("./dist/esm", { recursive: true });
const esmIndex = ["import * as wasmWrapper from './wasm_async_compile_wrapper/index';"]
packages.slice(1).forEach((pkgName) => {
  esmIndex.push(`export * from './${pkgName}/index';`)
})
fs.writeFileSync("./dist//esm/index.js", `${esmIndex.join("\n")}\n`);
fs.writeFileSync("./dist/esm/index.d.ts", `${esmIndex.join("\n")}\n`);

for (const pkg of packages) {
  execSync(`node ./scripts/buildCjs ${pkg}`);
  execSync(`node ./scripts/buildEsm ${pkg}`);
}

// rm temp folder
execSync("npm run rm-temp");

// check bindings for possible errors
execSync("npm run check");
