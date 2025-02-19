/* eslint-disable @typescript-eslint/no-var-requires */
const fs = require('node:fs');
const { execSync } = require('node:child_process');

// create dirs
fs.mkdirSync('./temp', { recursive: true });
fs.mkdirSync('./src/lib/types', { recursive: true });

// generate bindings
execSync(
  'wasm-bindgen --target nodejs ./src-tauri/target/wasm32-unknown-unknown/release/tauri_app.wasm --out-dir ./temp --out-name tauriBindings',
);

// prepare and move to src/lib/types as we only need the generated typings
let ts = fs.readFileSync(`./temp/tauriBindings.d.ts`, {
  encoding: 'utf-8',
});
ts = ts.replace(
  `/* tslint:disable */
/* eslint-disable */`,
  '',
);
ts = '/* this file is auto-generated, do not modify */\n' + ts;
fs.writeFileSync('./src/lib/types/tauriBindings.ts', ts);

execSync('npm run rm-temp');
