import { StringDecoder } from 'string_decoder'; // !node

function rand(min, max) { return Math.floor(Math.random() * (max - min/* + 1 */)) + min; };

let registry;
let wasm;
let ref = { deref() { } };

{
    const module = new WebAssembly.Module(WASM_BYTES);
    const instance = ref.deref() ?? (ref = new WeakRef(new WebAssembly.Instance(module, {
        env: {
            rand_range(min, max) { return rand(min, max); },
            rgb(min, max) {
                const ptr = mem.alloc(3);
                mem.u8(ptr, 3).set(new Uint8Array([rand(min, max), rand(min, max), rand(min, max)]));
                return ptr;
            },
            console_log(ptr, len) {
                console.log(decode_utf8(mem.u8(ptr, len ?? mem.length())));
            }
        }
    }))).deref();

    wasm = instance.exports;
}

class mem {
    static length() { return wasm.wlen(); }
    static token() { return wasm.wtoken(); }
    static alloc(size) { return wasm.walloc(size); }
    static free(ptr, size) { return wasm.wfree(ptr, size); }
    static u8(ptr, size) { return new Uint8Array(wasm.memory.buffer, ptr, size); }
    static u32(ptr, size) { return new Uint32Array(wasm.memory.buffer, ptr, size); }
    static gc(f) { return !('FinalizationRegistry' in globalThis) ? { delete(_) { }, add(_, __) { } } : { r: new FinalizationRegistry(f), delete(k) { this.r.unregister(k); }, add(k, v) { this.r.register(k, v, k); } }; }

    static copy_and_free(ptr, size) {
        let slice = mem.u8(ptr, size).slice();
        return (wasm.wfree(ptr, size), slice);
    }
}

const decode_utf8 = globalThis.Deno?.core?.decode ?? StringDecoder.prototype.end.bind(new StringDecoder);
const encode_utf8 = globalThis.Deno?.core?.encode ?? globalThis.Buffer?.from.bind(globalThis.Buffer) ?? TextEncoder.prototype.encode.bind(new TextEncoder);

if ('FinalizationRegistry' in globalThis) {
    registry = new FinalizationRegistry(([t, ptr]) => {
        if (t === 0) wasm.font_free(ptr);
        if (t === 1) wasm.captcha_free(ptr);
    });
}


export class Font {
    constructor(scale, buffer) {
        this.scale = scale;
        const ptr = mem.alloc(buffer.length);
        mem.u8(ptr, buffer.length).set(buffer);
        this.ptr = wasm.font_new(ptr, buffer.length, scale);

        if (!this.ptr) throw new Error('invalid font');
        if (registry) registry.register(this, [0, this.ptr], this);
    }

    free() {
        this.ptr = wasm.font_free(this.ptr);
        if (registry) registry.unregister(this);
    }
}

export class Captcha {
    #ptr;
    constructor(font, width, height, chars = 6) {
        this.width = width;
        this.height = height;

        let captcha_code = '';
        let charss = chars;
        while (charss--) {
            switch (rand(0, 3)) {
                case 0: captcha_code += String.fromCharCode(rand(0, 10) + 48); break;
                case 1: captcha_code += String.fromCharCode(rand(0, 26) + 97); break;
                case 2: captcha_code += String.fromCharCode(rand(0, 26) + 65); break;
            }
        };
        const tbuf = encode_utf8(captcha_code);
        const tptr = mem.alloc(tbuf.length);
        mem.u8(tptr, tbuf.length).set(tbuf);

        this.#ptr = wasm.draw_captcha(font.ptr, width, height, tptr, chars);
        //if (registry) registry.register(this, [1, this.#ptr], this);
    }

    free() {
        this.#ptr = wasm.captcha_free(this.#ptr);
        //if (registry) registry.unregister(this);
    }

    get buffer() { return mem.u8(wasm.captcha_buffer(this.#ptr), mem.length()); };
    get solution() { return decode_utf8(mem.u8(wasm.captcha_solution(this.#ptr), mem.length())); };

    png() {
        const ptr = wasm.captcha_as_png(this.#ptr, 0);
        if (ptr === 0) throw new Error('captcha: error encoding as png');
        return mem.u8(ptr, mem.length());
    };
}