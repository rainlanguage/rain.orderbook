const fs = require('fs');
const { execSync } = require('child_process');

const packagePrefix = 'rain_orderbook_';
const [pkg, buildType = ''] = process.argv.slice(2);

// generate node/web bindgens
fs.mkdirSync(`./dist/cjs/${pkg}`, { recursive: true });
fs.mkdirSync(`./dist/esm/${pkg}`, { recursive: true });
execSync(
	`wasm-bindgen --target nodejs ../../target/wasm32-unknown-unknown/release/${
		packagePrefix + pkg
	}.wasm --out-dir ./temp/node/${pkg} --out-name ${pkg}`
);
execSync(
	`wasm-bindgen --target web ../../target/wasm32-unknown-unknown/release/${
		packagePrefix + pkg
	}.wasm --out-dir ./temp/web/${pkg} --out-name ${pkg}`
);

// encode wasm as base64 into a json for cjs and esm that can be natively imported
// in js modules in order to avoid using fetch or fs operations
const wasmCjsBytes = fs.readFileSync(`./temp/node/${pkg}/${pkg}_bg.wasm`);
fs.writeFileSync(
	`./dist/cjs/${pkg}/orderbook_wbg.json`,
	JSON.stringify({
		wasm: Buffer.from(wasmCjsBytes, 'binary').toString('base64')
	})
);
const wasmEsmBytes = fs.readFileSync(`./temp/web/${pkg}/${pkg}_bg.wasm`);
fs.writeFileSync(
	`./dist/esm/${pkg}/orderbook_wbg.json`,
	JSON.stringify({
		wasm: Buffer.from(wasmEsmBytes, 'binary').toString('base64')
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
fs.writeFileSync(`./dist/cjs/${pkg}/index.d.ts`, dts);
if (buildType === 'webapp') {
	dts += `\nexport function init(): Promise<void>;`
}
fs.writeFileSync(`./dist/esm/${pkg}/index.d.ts`, dts);

// prepare cjs
let cjs = fs.readFileSync(`./temp/node/${pkg}/${pkg}.js`, {
	encoding: 'utf-8'
});
cjs = cjs.replace(
	`const path = require('path').join(__dirname, '${pkg}_bg.wasm');
const bytes = require('fs').readFileSync(path);`,
	`
const { Buffer } = require('buffer');
const wasmB64 = require('../cjs/orderbook_wbg.json');
const bytes = Buffer.from(wasmB64.wasm, 'base64');`
);
cjs = cjs.replace('const { TextEncoder, TextDecoder } = require(`util`);', '');
cjs = '/* this file is auto-generated, do not modify */\n' + cjs;
fs.writeFileSync(`./dist/cjs/${pkg}/index.js`, cjs);

// prepare esm
let esm = fs.readFileSync(`./temp/web/${pkg}/${pkg}.js`, {
	encoding: 'utf-8'
});
if (buildType === 'tauri') {
	// for tauri we need sync init (tauri v1 doesn't support top level await)
	esm = esm.replace(
	`export { initSync };
export default __wbg_init;`,
	`import { Buffer } from 'buffer';
import wasmB64 from './orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');
initSync(bytes);`
);
} else if (buildType === 'webapp') {
	if (pkg === 'js_api') {
// for webapp we need async init export, so that the webapp client hook
	// can call it during its initialization phase and await it, this is because
	// of safari issue with top level await imports on multiple module.
	// once safari bug is resolved this can be reverted to top level await again
	esm = esm.replace(
	`export { initSync };
export default __wbg_init;`,
	`import { Buffer } from 'buffer';
import wasmB64 from './orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');\n
/** Initialize the Orderbook pkg WebAssembly module */
export async function init_wasm() {
	await __wbg_init(bytes);
}`
);
	} else {
    // for webapp we need async init export, so that the webapp client hook
	// can call it during its initialization phase and await it, this is because
	// of safari issue with top level await imports on multiple module.
	// once safari bug is resolved this can be reverted to top level await again
	esm = esm.replace(
	`export { initSync };
export default __wbg_init;`,
	`import { Buffer } from 'buffer';
import wasmB64 from './orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');\n
initSync(bytes);`
);
	}
	
} else {
	// default case with top level await for node and modern esm bundlers
	esm = esm.replace(
	`export { initSync };
export default __wbg_init;`,
	`import { Buffer } from 'buffer';
import wasmB64 from './orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');
await __wbg_init(bytes);`
);
}
esm = '/* this file is auto-generated, do not modify */\n' + esm;
fs.writeFileSync(`./dist/esm/${pkg}/index.js`, esm);
