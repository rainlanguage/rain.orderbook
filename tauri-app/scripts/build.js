import { execSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

// create dirs
mkdirSync('./temp', { recursive: true });
mkdirSync('./src/lib/types', { recursive: true });

// generate bindings
execSync(
  'wasm-bindgen --target nodejs ./src-tauri/target/wasm32-unknown-unknown/release/tauri_app.wasm --out-dir ./temp --out-name tauriBindings',
);

// prepare and move to src/lib/types as we only need the ts typings
let ts = readFileSync(`./temp/tauriBindings.d.ts`, {
  encoding: 'utf-8',
});
ts = ts.replace(
  `/* tslint:disable */
/* eslint-disable */`,
  '',
);
ts = '/* this file is auto-generated, do not modify */\n' + ts;
writeFileSync('./src/lib/types/tauriBindings.ts', ts);

// remove temp dir
execSync('npm run rm-temp');
