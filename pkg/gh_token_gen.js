
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { writeFileSync } = require(`node:fs`);
const { TextDecoder, TextEncoder } = require(`util`);

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

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_2.set(idx, obj);
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

function isLikeNone(x) {
    return x === undefined || x === null;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder('utf-8');

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

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
        const ret = encodeString(arg, view);

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

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(
state => {
    wasm.__wbindgen_export_5.get(state.dtor)(state.a, state.b);
}
);

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
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_5.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_export_2.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}
/**
 * @returns {Promise<void>}
 */
module.exports.start = function() {
    const ret = wasm.start();
    return ret;
};

function __wbg_adapter_6(arg0, arg1, arg2) {
    wasm.closure90_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_11(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__h4db9b66db395760e(arg0, arg1);
}

function __wbg_adapter_51(arg0, arg1, arg2, arg3) {
    wasm.closure69_externref_shim(arg0, arg1, arg2, arg3);
}

const __wbindgen_enum_RequestCache = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];

const __wbindgen_enum_RequestCredentials = ["omit", "same-origin", "include"];

const __wbindgen_enum_RequestMode = ["same-origin", "no-cors", "cors", "navigate"];

module.exports.__wbg_Error_1f3748b298f99708 = function(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbg_abort_6665281623826052 = function(arg0) {
    arg0.abort();
};

module.exports.__wbg_abort_c11a5d245a242912 = function(arg0, arg1) {
    arg0.abort(arg1);
};

module.exports.__wbg_append_3e86b0cd6215edd8 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    arg0.append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

module.exports.__wbg_arrayBuffer_55e4a430671abfd8 = function() { return handleError(function (arg0) {
    const ret = arg0.arrayBuffer();
    return ret;
}, arguments) };

module.exports.__wbg_buffer_1f897e9f3ed6b41d = function(arg0) {
    const ret = arg0.buffer;
    return ret;
};

module.exports.__wbg_byteLength_8d9dcb51e13dc4c2 = function(arg0) {
    const ret = arg0.byteLength;
    return ret;
};

module.exports.__wbg_call_2f8d426a20a307fe = function() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

module.exports.__wbg_call_f53f0647ceb9c567 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

module.exports.__wbg_clearTimeout_6222fede17abcb1a = function(arg0) {
    const ret = clearTimeout(arg0);
    return ret;
};

module.exports.__wbg_done_4a7743b6f942c9f3 = function(arg0) {
    const ret = arg0.done;
    return ret;
};

module.exports.__wbg_fetch_9885d2e26ad251bb = function(arg0, arg1) {
    const ret = arg0.fetch(arg1);
    return ret;
};

module.exports.__wbg_fetch_f156d10be9a5c88a = function(arg0) {
    const ret = fetch(arg0);
    return ret;
};

module.exports.__wbg_getTime_5b1dd03bb6d4b784 = function(arg0) {
    const ret = arg0.getTime();
    return ret;
};

module.exports.__wbg_get_27b4bcbec57323ca = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

module.exports.__wbg_has_85abdd8aeb8edebf = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.has(arg0, arg1);
    return ret;
}, arguments) };

module.exports.__wbg_headers_177bc880a5823968 = function(arg0) {
    const ret = arg0.headers;
    return ret;
};

module.exports.__wbg_importKey_746ccc8f5f169c1f = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = arg0.importKey(getStringFromWasm0(arg1, arg2), arg3, arg4, arg5 !== 0, arg6);
    return ret;
}, arguments) };

module.exports.__wbg_instanceof_Response_0ab386c6818f788a = function(arg0) {
    let result;
    try {
        result = arg0 instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

module.exports.__wbg_iterator_96378c3c9a17347c = function() {
    const ret = Symbol.iterator;
    return ret;
};

module.exports.__wbg_length_904c0910ed998bf3 = function(arg0) {
    const ret = arg0.length;
    return ret;
};

module.exports.__wbg_log_9ba9f245c7648f5b = function(arg0, arg1) {
    console.log(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbg_new0_85cc856927102294 = function() {
    const ret = new Date();
    return ret;
};

module.exports.__wbg_new_12588505388d0897 = function() { return handleError(function () {
    const ret = new Headers();
    return ret;
}, arguments) };

module.exports.__wbg_new_1930cbb8d9ffc31b = function() {
    const ret = new Object();
    return ret;
};

module.exports.__wbg_new_6a8b180049d9484e = function() { return handleError(function () {
    const ret = new AbortController();
    return ret;
}, arguments) };

module.exports.__wbg_new_9190433fb67ed635 = function(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
};

module.exports.__wbg_new_d5e3800b120e37e1 = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_51(a, state0.b, arg0, arg1);
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

module.exports.__wbg_newfromslice_d0d56929c6d9c842 = function(arg0, arg1) {
    const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbg_newnoargs_a81330f6e05d8aca = function(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbg_newwithbyteoffsetandlength_9aade108cd45cf37 = function(arg0, arg1, arg2) {
    const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
    return ret;
};

module.exports.__wbg_newwithstrandinit_e8e22e9851f3c2fe = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
    return ret;
}, arguments) };

module.exports.__wbg_next_2e6b37020ac5fe58 = function() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments) };

module.exports.__wbg_next_3de8f2669431a3ff = function(arg0) {
    const ret = arg0.next;
    return ret;
};

module.exports.__wbg_prototypesetcall_c5f74efd31aea86b = function(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
};

module.exports.__wbg_queueMicrotask_bcc6e26d899696db = function(arg0) {
    const ret = arg0.queueMicrotask;
    return ret;
};

module.exports.__wbg_queueMicrotask_f24a794d09c42640 = function(arg0) {
    queueMicrotask(arg0);
};

module.exports.__wbg_resolve_5775c0ef9222f556 = function(arg0) {
    const ret = Promise.resolve(arg0);
    return ret;
};

module.exports.__wbg_setTimeout_2b339866a2aa3789 = function(arg0, arg1) {
    const ret = setTimeout(arg0, arg1);
    return ret;
};

module.exports.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
};

module.exports.__wbg_setbody_e324371c31597f2a = function(arg0, arg1) {
    arg0.body = arg1;
};

module.exports.__wbg_setcache_7c95e3469a5bfb76 = function(arg0, arg1) {
    arg0.cache = __wbindgen_enum_RequestCache[arg1];
};

module.exports.__wbg_setcredentials_55a9317ed2777533 = function(arg0, arg1) {
    arg0.credentials = __wbindgen_enum_RequestCredentials[arg1];
};

module.exports.__wbg_setheaders_ac0b1e4890a949cd = function(arg0, arg1) {
    arg0.headers = arg1;
};

module.exports.__wbg_setmethod_9ce6e95af1ae0eaf = function(arg0, arg1, arg2) {
    arg0.method = getStringFromWasm0(arg1, arg2);
};

module.exports.__wbg_setmode_b89d1784e7e7f118 = function(arg0, arg1) {
    arg0.mode = __wbindgen_enum_RequestMode[arg1];
};

module.exports.__wbg_setsignal_e663c6d962763cd5 = function(arg0, arg1) {
    arg0.signal = arg1;
};

module.exports.__wbg_sign_853570a519179959 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    const ret = arg0.sign(getStringFromWasm0(arg1, arg2), arg3, getArrayU8FromWasm0(arg4, arg5));
    return ret;
}, arguments) };

module.exports.__wbg_signal_bdb003fe19e53a13 = function(arg0) {
    const ret = arg0.signal;
    return ret;
};

module.exports.__wbg_static_accessor_CRYPTO_0b52a9a6eaef7cbf = function() {
    const ret = crypto;
    return ret;
};

module.exports.__wbg_static_accessor_ENV_f7e6665b59ba6f10 = function() {
    const ret = process.env;
    return ret;
};

module.exports.__wbg_static_accessor_GLOBAL_1f13249cc3acc96d = function() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_GLOBAL_THIS_df7ae94b1e0ed6a3 = function() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_SELF_6265471db3b3c228 = function() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_WINDOW_16fb482f8ec52863 = function() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_status_31874648c8651949 = function(arg0) {
    const ret = arg0.status;
    return ret;
};

module.exports.__wbg_stringify_1f41b6198e0932e0 = function() { return handleError(function (arg0) {
    const ret = JSON.stringify(arg0);
    return ret;
}, arguments) };

module.exports.__wbg_subtle_40dd37072861a308 = function(arg0) {
    const ret = arg0.subtle;
    return ret;
};

module.exports.__wbg_then_8d2fcccde5380a03 = function(arg0, arg1, arg2) {
    const ret = arg0.then(arg1, arg2);
    return ret;
};

module.exports.__wbg_then_9cc266be2bf537b6 = function(arg0, arg1) {
    const ret = arg0.then(arg1);
    return ret;
};

module.exports.__wbg_url_d5273b9e10503471 = function(arg0, arg1) {
    const ret = arg1.url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbg_value_09d0b4eaab48b91d = function(arg0) {
    const ret = arg0.value;
    return ret;
};

module.exports.__wbg_wbindgencbdrop_a85ed476c6a370b9 = function(arg0) {
    const obj = arg0.original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

module.exports.__wbg_wbindgendebugstring_bb652b1bc2061b6d = function(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbg_wbindgenisfunction_ea72b9d66a0e1705 = function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

module.exports.__wbg_wbindgenisobject_dfe064a121d87553 = function(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

module.exports.__wbg_wbindgenisstring_4b74e4111ba029e6 = function(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

module.exports.__wbg_wbindgenisundefined_71f08a6ade4354e7 = function(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

module.exports.__wbg_wbindgenstringget_43fe05afe34b0cb1 = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbg_wbindgenthrow_4c11a24fca429ccf = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbg_writeFileSync_1d1075ff715ab4c7 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = writeFileSync(arg0, arg1, arg2);
    return ret;
}, arguments) };

module.exports.__wbindgen_cast_1f4084f597db5a1b = function(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { dtor_idx: 21, function: Function { arguments: [Externref], shim_idx: 90, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, 21, __wbg_adapter_6);
    return ret;
};

module.exports.__wbindgen_cast_2241b6af4c4b2941 = function(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

module.exports.__wbindgen_cast_25a0a844437d0e92 = function(arg0, arg1) {
    var v0 = getArrayJsValueFromWasm0(arg0, arg1).slice();
    wasm.__wbindgen_free(arg0, arg1 * 4, 4);
    // Cast intrinsic for `Vector(NamedExternref("string")) -> Externref`.
    const ret = v0;
    return ret;
};

module.exports.__wbindgen_cast_2e42aeecbc2281d3 = function(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { dtor_idx: 21, function: Function { arguments: [], shim_idx: 22, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, 21, __wbg_adapter_11);
    return ret;
};

module.exports.__wbindgen_cast_d6cd19b81560fd6e = function(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return ret;
};

module.exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_export_2;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

const path = require('path').join(__dirname, 'gh_token_gen_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

wasm.__wbindgen_start();

