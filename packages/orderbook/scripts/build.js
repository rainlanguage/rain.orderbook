const fs = require("fs");
const { execSync } = require("child_process");

const [buildType = ''] = process.argv.slice(2);

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
fs.writeFileSync("./dist//esm/index.js", "export * from './wasm_async_compile_wrapper/index';\nexport * from './js_api/index';\n");
fs.writeFileSync("./dist/esm/index.d.ts", "export * from './wasm_async_compile_wrapper/index';\nexport * from './js_api/index';\n");
fs.writeFileSync("./dist//cjs/index.js", "'use strict';\nmodule.exports = {\n...require('./wasm_async_compile_wrapper/index'),\n...require('./js_api/index')\n};");
fs.writeFileSync("./dist/cjs/index.d.ts", "export * from './wasm_async_compile_wrapper/index';\nexport * from './js_api/index';\n");

// build for wasm32 target
execSync("npm run build-wasm");

// build specified packages and include them in final index file
// list of packages to build can be extended by adding new package
// names to the list below
const packages = ["js_api", "wasm_async_compile_wrapper"];
for (const pkg of packages) {
  execSync(`node ./scripts/buildPackage ${pkg} ${buildType}`);
}

// rm temp folder
execSync("npm run rm-temp");

// check bindings for possible errors
// execSync("npm run check");
