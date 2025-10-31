
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
const { writeFileSync } = require(`node:fs`);

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    }
}

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => state.dtor(state.a, state.b));

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            state.dtor(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
/**
 * @returns {Promise<void>}
 */
exports.start = function() {
    const ret = wasm.start();
    return ret;
};

function wasm_bindgen__convert__closures_____invoke__hf3b193970988da45(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__hf3b193970988da45(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__hf99040ea1e6226e0(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__hf99040ea1e6226e0(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h01bd32e0d7a493ef(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__h01bd32e0d7a493ef(arg0, arg1, arg2, arg3);
}

const __wbindgen_enum_RequestCache = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];

const __wbindgen_enum_RequestCredentials = ["omit", "same-origin", "include"];

const __wbindgen_enum_RequestMode = ["same-origin", "no-cors", "cors", "navigate"];

exports.__wbg_Error_e83987f665cf5504 = function(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

exports.__wbg___wbindgen_debug_string_df47ffb5e35e6763 = function(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

exports.__wbg___wbindgen_is_function_ee8a6c5833c90377 = function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

exports.__wbg___wbindgen_is_object_c818261d21f283a4 = function(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

exports.__wbg___wbindgen_is_string_fbb76cb2940daafd = function(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

exports.__wbg___wbindgen_is_undefined_2d472862bd29a478 = function(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

exports.__wbg___wbindgen_string_get_e4f06c90489ad01b = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

exports.__wbg___wbindgen_throw_b855445ff6a94295 = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

exports.__wbg__wbg_cb_unref_2454a539ea5790d9 = function(arg0) {
    arg0._wbg_cb_unref();
};

exports.__wbg_abort_28ad55c5825b004d = function(arg0, arg1) {
    arg0.abort(arg1);
};

exports.__wbg_abort_e7eb059f72f9ed0c = function(arg0) {
    arg0.abort();
};

exports.__wbg_append_b577eb3a177bc0fa = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    arg0.append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

exports.__wbg_arrayBuffer_b375eccb84b4ddf3 = function() { return handleError(function (arg0) {
    const ret = arg0.arrayBuffer();
    return ret;
}, arguments) };

exports.__wbg_buffer_ccc4520b36d3ccf4 = function(arg0) {
    const ret = arg0.buffer;
    return ret;
};

exports.__wbg_byteLength_eb3438154e05658e = function(arg0) {
    const ret = arg0.byteLength;
    return ret;
};

exports.__wbg_call_525440f72fbfc0ea = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

exports.__wbg_call_e762c39fa8ea36bf = function() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

exports.__wbg_clearTimeout_7a42b49784aea641 = function(arg0) {
    const ret = clearTimeout(arg0);
    return ret;
};

exports.__wbg_done_2042aa2670fb1db1 = function(arg0) {
    const ret = arg0.done;
    return ret;
};

exports.__wbg_fetch_74a3e84ebd2c9a0e = function(arg0) {
    const ret = fetch(arg0);
    return ret;
};

exports.__wbg_fetch_f8ba0e29a9d6de0d = function(arg0, arg1) {
    const ret = arg0.fetch(arg1);
    return ret;
};

exports.__wbg_getTime_14776bfb48a1bff9 = function(arg0) {
    const ret = arg0.getTime();
    return ret;
};

exports.__wbg_get_efcb449f58ec27c2 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

exports.__wbg_has_787fafc980c3ccdb = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.has(arg0, arg1);
    return ret;
}, arguments) };

exports.__wbg_headers_b87d7eaba61c3278 = function(arg0) {
    const ret = arg0.headers;
    return ret;
};

exports.__wbg_importKey_2be19189a1451235 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = arg0.importKey(getStringFromWasm0(arg1, arg2), arg3, arg4, arg5 !== 0, arg6);
    return ret;
}, arguments) };

exports.__wbg_instanceof_Response_f4f3e87e07f3135c = function(arg0) {
    let result;
    try {
        result = arg0 instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

exports.__wbg_iterator_e5822695327a3c39 = function() {
    const ret = Symbol.iterator;
    return ret;
};

exports.__wbg_length_69bca3cb64fc8748 = function(arg0) {
    const ret = arg0.length;
    return ret;
};

exports.__wbg_log_d6d3266657dcba52 = function(arg0, arg1) {
    console.log(getStringFromWasm0(arg0, arg1));
};

exports.__wbg_new_0_f9740686d739025c = function() {
    const ret = new Date();
    return ret;
};

exports.__wbg_new_1acc0b6eea89d040 = function() {
    const ret = new Object();
    return ret;
};

exports.__wbg_new_2531773dac38ebb3 = function() { return handleError(function () {
    const ret = new AbortController();
    return ret;
}, arguments) };

exports.__wbg_new_3c3d849046688a66 = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return wasm_bindgen__convert__closures_____invoke__h01bd32e0d7a493ef(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return ret;
    } finally {
        state0.a = state0.b = 0;
    }
};

exports.__wbg_new_5a79be3ab53b8aa5 = function(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
};

exports.__wbg_new_9edf9838a2def39c = function() { return handleError(function () {
    const ret = new Headers();
    return ret;
}, arguments) };

exports.__wbg_new_from_slice_92f4d78ca282a2d2 = function(arg0, arg1) {
    const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
    return ret;
};

exports.__wbg_new_no_args_ee98eee5275000a4 = function(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return ret;
};

exports.__wbg_new_with_byte_offset_and_length_46e3e6a5e9f9e89b = function(arg0, arg1, arg2) {
    const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
    return ret;
};

exports.__wbg_new_with_str_and_init_0ae7728b6ec367b1 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
    return ret;
}, arguments) };

exports.__wbg_next_020810e0ae8ebcb0 = function() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments) };

exports.__wbg_next_2c826fe5dfec6b6a = function(arg0) {
    const ret = arg0.next;
    return ret;
};

exports.__wbg_prototypesetcall_2a6620b6922694b2 = function(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
};

exports.__wbg_queueMicrotask_34d692c25c47d05b = function(arg0) {
    const ret = arg0.queueMicrotask;
    return ret;
};

exports.__wbg_queueMicrotask_9d76cacb20c84d58 = function(arg0) {
    queueMicrotask(arg0);
};

exports.__wbg_resolve_caf97c30b83f7053 = function(arg0) {
    const ret = Promise.resolve(arg0);
    return ret;
};

exports.__wbg_setTimeout_7bb3429662ab1e70 = function(arg0, arg1) {
    const ret = setTimeout(arg0, arg1);
    return ret;
};

exports.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
};

exports.__wbg_set_body_3c365989753d61f4 = function(arg0, arg1) {
    arg0.body = arg1;
};

exports.__wbg_set_cache_2f9deb19b92b81e3 = function(arg0, arg1) {
    arg0.cache = __wbindgen_enum_RequestCache[arg1];
};

exports.__wbg_set_credentials_f621cd2d85c0c228 = function(arg0, arg1) {
    arg0.credentials = __wbindgen_enum_RequestCredentials[arg1];
};

exports.__wbg_set_headers_6926da238cd32ee4 = function(arg0, arg1) {
    arg0.headers = arg1;
};

exports.__wbg_set_method_c02d8cbbe204ac2d = function(arg0, arg1, arg2) {
    arg0.method = getStringFromWasm0(arg1, arg2);
};

exports.__wbg_set_mode_52ef73cfa79639cb = function(arg0, arg1) {
    arg0.mode = __wbindgen_enum_RequestMode[arg1];
};

exports.__wbg_set_signal_dda2cf7ccb6bee0f = function(arg0, arg1) {
    arg0.signal = arg1;
};

exports.__wbg_sign_2fa4f68647bacd99 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    const ret = arg0.sign(getStringFromWasm0(arg1, arg2), arg3, getArrayU8FromWasm0(arg4, arg5));
    return ret;
}, arguments) };

exports.__wbg_signal_4db5aa055bf9eb9a = function(arg0) {
    const ret = arg0.signal;
    return ret;
};

exports.__wbg_static_accessor_CRYPTO_6d4c140e4274af65 = function() {
    const ret = crypto;
    return ret;
};

exports.__wbg_static_accessor_ENV_4039f412b11ee961 = function() {
    const ret = process.env;
    return ret;
};

exports.__wbg_static_accessor_GLOBAL_89e1d9ac6a1b250e = function() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_static_accessor_GLOBAL_THIS_8b530f326a9e48ac = function() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_static_accessor_SELF_6fdf4b64710cc91b = function() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_static_accessor_WINDOW_b45bfc5a37f6cfa2 = function() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_status_de7eed5a7a5bfd5d = function(arg0) {
    const ret = arg0.status;
    return ret;
};

exports.__wbg_stringify_b5fb28f6465d9c3e = function() { return handleError(function (arg0) {
    const ret = JSON.stringify(arg0);
    return ret;
}, arguments) };

exports.__wbg_subtle_a158c8cba320b8ed = function(arg0) {
    const ret = arg0.subtle;
    return ret;
};

exports.__wbg_then_4f46f6544e6b4a28 = function(arg0, arg1) {
    const ret = arg0.then(arg1);
    return ret;
};

exports.__wbg_then_70d05cf780a18d77 = function(arg0, arg1, arg2) {
    const ret = arg0.then(arg1, arg2);
    return ret;
};

exports.__wbg_url_b36d2a5008eb056f = function(arg0, arg1) {
    const ret = arg1.url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

exports.__wbg_value_692627309814bb8c = function(arg0) {
    const ret = arg0.value;
    return ret;
};

exports.__wbg_writeFileSync_bfa18463b6f3d5ad = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = writeFileSync(arg0, arg1, arg2);
    return ret;
}, arguments) };

exports.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

exports.__wbindgen_cast_25a0a844437d0e92 = function(arg0, arg1) {
    var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
    wasm.__wbindgen_free(arg0, arg1 * 4, 4);
    // Cast intrinsic for `Vector(NamedExternref("string")) -> Externref`.
    const ret = v0;
    return ret;
};

exports.__wbindgen_cast_2e42aeecbc2281d3 = function(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { dtor_idx: 21, function: Function { arguments: [], shim_idx: 22, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h7e298734f96c9ead, wasm_bindgen__convert__closures_____invoke__hf3b193970988da45);
    return ret;
};

exports.__wbindgen_cast_4ea0a60081e5f2ec = function(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { dtor_idx: 90, function: Function { arguments: [Externref], shim_idx: 91, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__hd05edbca92891fdd, wasm_bindgen__convert__closures_____invoke__hf99040ea1e6226e0);
    return ret;
};

exports.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return ret;
};

exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_externrefs;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

const wasmPath = `${__dirname}/gh_token_gen_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
const wasm = exports.__wasm = new WebAssembly.Instance(wasmModule, imports).exports;

wasm.__wbindgen_start();

