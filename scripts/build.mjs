import { readFile, writeFile, mkdir, readdir } from 'fs/promises';
import * as esbuild from 'esbuild';
import { exec as e } from 'child_process';
import { promisify } from 'util';

const { version } = JSON.parse((await readFile('./package.json')).toString());
const exec = promisify(e);
const opt = [
  '--optimize-instructions',
  '--duplicate-import-elimination',
  '--duplicate-function-elimination',
  '-ffm',
  '-aimfs',
  '--memory64-lowering',
  '--memory-packing',
  '-n',
  // '--strip-target-features',
  '--dae-optimizing',
  '--vacuum',
  '--strip-producers',
  '--strip-dwarf',
  '--strip-debug',
  '--inlining-optimizing',
  '-O3',
  '--dce',
  '--enable-bulk-memory',
  '-ffm',
  '--detect-features'
];
const options = opt.join(' ');

function btoa(target) { return Buffer.from(target, 'binary').toString('base64') }
function atob(target) { return Buffer.from(target, 'base64').toString('binary') }

const key = 'WASM_BYTES';
await mkdir(`./target`).catch(() => { });
const dir = 'captcha';
console.time(`building ${dir}`);

await mkdir(`./dist`).catch(() => { });
try { await exec(`~/Downloads/binaryen-version_117/bin/wasm-opt ${options} -o ./dist/${dir}.wasm ./target/wasm32-unknown-unknown/release/${dir}.wasm`); } catch (e) { console.error('wasm-opt failed', e); }

const wb = await readFile(`./dist/${dir}.wasm`);
const wasm = btoa((wb).toString('base64'));

const js = (await readFile(`./lib/${dir}.js`)).toString();

const { code: cjs } = (await esbuild.transform(replacer(js, 'node'), { format: 'cjs', minify: false, target: 'node14' }));
const ops = [
  writeFile(`./dist/node.cjs`, Buffer.from(cjs.replace(key, `require('fs').readFileSync(require('path').join(__dirname, './${dir}.wasm'))`))),
  // writeFile(`./dist/deno.js`, Buffer.from(replacer(js, 'deno').replace(key, `Uint8Array.from(atob('${wasm}'), char => char.codePointAt(0))`))),
  writeFile(`./dist/node.mjs`, Buffer.from(replacer(js, 'node').replace(key, `await import('fs/promises').then(fs => fs.readFile(new URL('./${dir}.wasm', import.meta.url)))`))),
];
await Promise.all(ops);
console.timeEnd(`building ${dir}`);


function replacer(code, runtime) {
  return code
    .replace(/.+\/\/ !(\w+)/g, (str, runtime_str) => runtime !== runtime_str ? '' : str)
    .replace(/\/\/ ![^]+?\n\/\/ !(\w+)/g, (str, runtime_str) => runtime !== runtime_str ? '' : str);
}