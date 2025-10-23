const fs = require('fs');
const { execSync } = require('child_process');

const packagePrefix = 'rain_orderbook_';
const [pkg, buildType = ''] = process.argv.slice(2);

// generate web bindgens for esm output
fs.mkdirSync(`./dist/esm/${pkg}`, { recursive: true });
execSync(
    `wasm-bindgen --target web ../../target/wasm32-unknown-unknown/release/${
        packagePrefix + pkg
    }.wasm --out-dir ./temp/web/${pkg} --out-name ${pkg}`
);

// encode wasm as base64 into a json for esm that can be natively imported
// in js modules in order to avoid using fetch or fs operations
const wasmEsmBytes = fs.readFileSync(`./temp/web/${pkg}/${pkg}_bg.wasm`);
fs.writeFileSync(
    `./dist/esm/${pkg}/orderbook_wbg.json`,
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
if (pkg !== 'wasm_async_compile_wrapper') {
    dts += "/** Initialize the Orderbook pkg WebAssembly module */\nexport function init_wasm(): Promise<void>\n";
}
fs.writeFileSync(`./dist/esm/${pkg}/index.d.ts`, dts);

// prepare esm .js
let esm = fs.readFileSync(`./temp/web/${pkg}/${pkg}.js`, {
    encoding: 'utf-8'
});

// in esm due to issues/limitations with safari and chrome we sync init the wrapper
// but do NOT init the other pkgs, as they will get initialized from inside of the wrapper.
// the wrapper can be removed once safari issue with "top-level-await" is resolved
if (pkg === 'wasm_async_compile_wrapper') {
    esm = esm.replace(`export { initSync };
export default __wbg_init;`,
    `import { Buffer } from 'buffer';
import wasmB64 from './orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');\n
initSync(bytes);`
    );
} else {
    esm = esm.replace(`export { initSync };
export default __wbg_init;`,
    `import { Buffer } from 'buffer';
import wasmB64 from './orderbook_wbg.json';
const bytes = Buffer.from(wasmB64.wasm, 'base64');\n
/** Initialize the Orderbook pkg WebAssembly module */
export async function init_wasm() {
    try {
        initSync(bytes);
    } catch (error) {
        await __wbg_init(bytes);   
    }
}`
    );
}
esm = '/* this file is auto-generated, do not modify */\n' + esm;
fs.writeFileSync(`./dist/esm/${pkg}/index.js`, esm);
