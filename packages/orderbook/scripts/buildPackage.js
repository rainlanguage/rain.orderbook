const fs = require('fs');
const { execSync } = require('child_process');

const run = (command) => execSync(command, { stdio: 'inherit' });

const packagePrefix = 'rain_orderbook_';
const [package, isTauriBuild = false] = process.argv.slice(2);

// generate node/web bindgens
run(
	`wasm-bindgen --target nodejs ../../target/wasm32-unknown-unknown/release/${
		packagePrefix + package
	}.wasm --out-dir ./temp/node/${package} --out-name ${package}`
);
run(
	`wasm-bindgen --target web ../../target/wasm32-unknown-unknown/release/${
		packagePrefix + package
	}.wasm --out-dir ./temp/web/${package} --out-name ${package}`
);

// encode wasm as base64 into a json for cjs and esm that can be natively imported
// in js modules in order to avoid using fetch or fs operations
const wasmCjsBytes = fs.readFileSync(`./temp/node/${package}/${package}_bg.wasm`);
fs.writeFileSync(
	`./dist/cjs/orderbook_wbg.json`,
	JSON.stringify({
		wasm: Buffer.from(wasmCjsBytes, 'binary').toString('base64')
	})
);
const wasmEsmBytes = fs.readFileSync(`./temp/web/${package}/${package}_bg.wasm`);
fs.writeFileSync(
	`./dist/esm/orderbook_wbg.json`,
	JSON.stringify({
		wasm: Buffer.from(wasmEsmBytes, 'binary').toString('base64')
	})
);

// prepare the dts
let dts = fs.readFileSync(`./temp/node/${package}/${package}.d.ts`, {
	encoding: 'utf-8'
});
dts = dts.replace(
	`/* tslint:disable */
/* eslint-disable */`,
	''
);
dts = '/* this file is auto-generated, do not modify */\n' + dts;
fs.writeFileSync(`./dist/cjs/index.d.ts`, dts);
fs.writeFileSync(`./dist/esm/index.d.ts`, dts);

// prepare cjs
let cjs = fs.readFileSync(`./temp/node/${package}/${package}.js`, {
	encoding: 'utf-8'
});
cjs = cjs.replace(
	`const path = require('path').join(__dirname, '${package}_bg.wasm');
const bytes = require('fs').readFileSync(path);`,
	`
const { Buffer } = require('buffer');
const wasmB64 = require('../cjs/orderbook_wbg.json');
const bytes = Buffer.from(wasmB64.wasm, 'base64');`
);
cjs = cjs.replace('const { TextEncoder, TextDecoder } = require(`util`);', '');
cjs = '/* this file is auto-generated, do not modify */\n' + cjs;
fs.writeFileSync(`./dist/cjs/index.js`, cjs);

// prepare esm
let esm = fs.readFileSync(`./temp/web/${package}/${package}.js`, {
	encoding: 'utf-8'
});
if (isTauriBuild) {
	esm = esm.replace(
	`export { initSync };
export default __wbg_init;`,
	`import { Buffer } from 'buffer';
import wasmB64 from '../esm/orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');
initSync(bytes);`
);
} else {
	esm = esm.replace(
	`export { initSync };
export default __wbg_init;`,
	`import { Buffer } from 'buffer';
import wasmB64 from '../esm/orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');
await __wbg_init(bytes);`
);
}
esm = '/* this file is auto-generated, do not modify */\n' + esm;
fs.writeFileSync(`./dist/esm/index.js`, esm);
