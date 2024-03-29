import { exec as e } from 'child_process';
import { promisify } from 'util';
const exec = promisify(e);
// unset NODE && ERROR_ON_UNDEFINED_SYMBOLS=0 
const script = 'STANDALONE_WASM=true cargo build --release';

await exec(script);