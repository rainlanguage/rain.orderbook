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
const packages = ["js_api"];

for (const pkg of packages) {
  // build for cjs and esm
  execSync(`node ./scripts/buildCjs ${pkg}`);
  execSync(`node ./scripts/buildEsm ${pkg}`);

  // check wasm size
  execSync(`node ./scripts/sizeCheck ${pkg}`);
}

// rm temp folder
execSync("npm run rm-temp");

// check bindings for possible errors
execSync("npm run check");
