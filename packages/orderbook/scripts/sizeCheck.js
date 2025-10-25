const fs = require('fs');

const SIZE_LIMIT = 8_388_608; // 8 MB binary

const [pkg] = process.argv.slice(2);

// we only need to check size on web/esm
const wasmEsmBytes = fs.readFileSync(`./temp/web/${pkg}/${pkg}_bg.wasm`);
if (wasmEsmBytes.length > SIZE_LIMIT) {
    throw new Error("ESM wasm size exceeds 8 MB limit!")
}
