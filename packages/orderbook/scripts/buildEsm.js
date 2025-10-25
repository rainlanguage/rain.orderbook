const fs = require('fs');
const { execSync } = require('child_process');

const packagePrefix = 'rain_orderbook_';
const [pkg] = process.argv.slice(2);

// generate web bindgens for esm output
fs.mkdirSync(`./dist/esm`, { recursive: true });
execSync(
    `wasm-bindgen --target web ../../target/wasm32-unknown-unknown/release-wasm/${
        packagePrefix + pkg
    }.wasm --out-dir ./temp/web/${pkg} --out-name ${pkg}`
);

// encode wasm as base64 into a json for esm that can be natively imported
// in js modules in order to avoid using fetch or fs operations
const wasmEsmBytes = fs.readFileSync(`./temp/web/${pkg}/${pkg}_bg.wasm`);
fs.writeFileSync(
    `./dist/esm/orderbook_wbg.json`,
    JSON.stringify({
        wasm: Buffer.from(wasmEsmBytes, 'binary').toString('base64')
    })
);

// prepare the dts
let dts = fs.readFileSync(`./temp/web/${pkg}/${pkg}.d.ts`, {
    encoding: 'utf-8'
});
dts = dts.replace(
    `/* tslint:disable */
/* eslint-disable */`,
    ''
);
dts = '/* this file is auto-generated, do not modify */\n' + dts;

// after using opt-level on wasm build, WasmEncodedResult and WasmEncodedError
// are duplicated in the dts so we need to dedupe them
const dups = [
    "export type WasmEncodedResult<T> = { value: T; error: undefined } | { value: undefined; error: WasmEncodedError };",
    `export interface WasmEncodedError {
    msg: string;
    readableMsg: string;
}`
];
for (const dup of dups) {
    const index = dts.indexOf(dup);
    if (index === -1) continue;
    dts = dts.replaceAll(dup, "");
    const start = dts.slice(0, index);
    const end = dts.slice(index);
    console.log(start.length, end.length)
    dts = start + dup + end;
}
fs.writeFileSync(`./dist/esm/index.d.ts`, dts);

// prepare esm .js
let esm = fs.readFileSync(`./temp/web/${pkg}/${pkg}.js`, {
    encoding: 'utf-8'
});
esm = esm.replace(`export { initSync };
export default __wbg_init;`,
    `import { Buffer } from 'buffer';
import wasmB64 from './orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');\n
initSync(bytes);`
    );
esm = '/* this file is auto-generated, do not modify */\n' + esm;
fs.writeFileSync(`./dist/esm/index.js`, esm);
