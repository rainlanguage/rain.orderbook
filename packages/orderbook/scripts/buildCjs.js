const fs = require('fs');
const { execSync } = require('child_process');

const packagePrefix = 'rain_orderbook_';
const [pkg] = process.argv.slice(2);

// for cjs we only need to build "js_api" since we dont have limitations on wasm init
if (pkg !== "js_api") return;

// generate node bindgens for cjs output
fs.mkdirSync(`./dist/cjs`, { recursive: true });
execSync(
    `wasm-bindgen --target nodejs ../../target/wasm32-unknown-unknown/release/${
        packagePrefix + pkg
    }.wasm --out-dir ./temp/node/${pkg} --out-name ${pkg}`
);

// encode wasm as base64 into a json for cjs that can be natively imported
// in js modules in order to avoid using fetch or fs operations
const wasmCjsBytes = fs.readFileSync(`./temp/node/${pkg}/${pkg}_bg.wasm`);
fs.writeFileSync(
    "./dist/cjs/orderbook_wbg.json",
    JSON.stringify({
        wasm: Buffer.from(wasmCjsBytes, 'binary').toString('base64')
    })
);

// prepare the dts
let dts = fs.readFileSync(`./temp/node/${pkg}/${pkg}.d.ts`, {
    encoding: 'utf-8'
});
dts = dts.replace(
    `/* tslint:disable */
/* eslint-disable */`,
    ''
);
dts = '/* this file is auto-generated, do not modify */\n' + dts;
fs.writeFileSync(`./dist/cjs/index.d.ts`, dts);

// prepare cjs
let cjs = fs.readFileSync(`./temp/node/${pkg}/${pkg}.js`, {
    encoding: 'utf-8'
});
cjs = cjs.replace(
    `const path = require('path').join(__dirname, '${pkg}_bg.wasm');
const bytes = require('fs').readFileSync(path);`,
    `
const { Buffer } = require('buffer');
const wasmB64 = require('./orderbook_wbg.json');
const bytes = Buffer.from(wasmB64.wasm, 'base64');`
);
cjs = cjs.replace('const { TextEncoder, TextDecoder } = require(`util`);', '');
cjs = '/* this file is auto-generated, do not modify */\n' + cjs;
fs.writeFileSync(`./dist/cjs/index.js`, cjs);
