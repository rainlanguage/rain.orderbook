const fs = require("fs");
const { execSync } = require("child_process");

// create root esm.js and cjs.js files with their .d.ts
fs.writeFileSync(
  "./cjs.js",
  '"use strict";\n\nmodule.exports = require("./dist/cjs/index");\n'
);
fs.writeFileSync("./cjs.d.ts", 'export * from "./dist/types/index";\n');
fs.writeFileSync("./esm.js", 'export * from "./dist/esm/index";\n');
fs.writeFileSync("./esm.d.ts", 'export * from "./dist/types/index";\n');

// create dist dir
fs.mkdirSync("./dist/cjs", { recursive: true });
fs.mkdirSync("./dist/esm", { recursive: true });
fs.mkdirSync("./dist/types", { recursive: true });

// build for wasm32 target
execSync("npm run build-wasm");

// build specified packages and include them in final index file
// list of packages to build can be extended by adding new package
// names to the list below
const packages = ["common", "quote", "js_api"];
for (const package of packages) {
  execSync(`node ./scripts/buildPackage ${package}`);
}

// create index file that exports all packages that were built in previous step
fs.writeFileSync(
  "./dist/cjs/index.js",
  `"use strict";
${packages.map((v) => `const ${v} = require("./${v}");`).join("\n")}

module.exports = {
    ${packages.map((v) => `${v}`).join(",\n    ")}
};
`
);
fs.writeFileSync(
  "./dist/types/index.d.ts",
  packages.map((v) => `export * as ${v} from "./${v}";`).join("\n")
);
fs.writeFileSync(
  "./dist/esm/index.js",
  packages.map((v) => `export * as ${v} from "./${v}";`).join("\n")
);

// rm temp folder
execSync("npm run rm-temp");
