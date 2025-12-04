
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;

const { open } = require(`node:fs/promises`);
const { env, stdout } = require(`node:process`);

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => state.dtor(state.a, state.b));

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

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

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

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

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

let WASM_VECTOR_LEN = 0;

function wasm_bindgen__convert__closures_____invoke__h1b6609413577a3fc(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h1b6609413577a3fc(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__he91f9a0170836f5d(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__he91f9a0170836f5d(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__h72f6d572297204b0(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__h72f6d572297204b0(arg0, arg1, arg2, arg3);
}

const __wbindgen_enum_RequestCache = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];

const __wbindgen_enum_RequestCredentials = ["omit", "same-origin", "include"];

const __wbindgen_enum_RequestMode = ["same-origin", "no-cors", "cors", "navigate"];

const CreateReadStreamOptionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_createreadstreamoption_free(ptr >>> 0, 1));

const CreateWriteStreamOptionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_createwritestreamoption_free(ptr >>> 0, 1));

const IntegerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_integer_free(ptr >>> 0, 1));

const WriteOptionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_writeoption_free(ptr >>> 0, 1));

class CreateReadStreamOption {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CreateReadStreamOptionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_createreadstreamoption_free(ptr, 0);
    }
}
if (Symbol.dispose) CreateReadStreamOption.prototype[Symbol.dispose] = CreateReadStreamOption.prototype.free;
exports.CreateReadStreamOption = CreateReadStreamOption;

class CreateWriteStreamOption {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CreateWriteStreamOptionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_createwritestreamoption_free(ptr, 0);
    }
}
if (Symbol.dispose) CreateWriteStreamOption.prototype[Symbol.dispose] = CreateWriteStreamOption.prototype.free;
exports.CreateWriteStreamOption = CreateWriteStreamOption;

/**
 * Integer type
 *
 * Primary usage is exporting Rust value into Node world.
 * For most cases, it is prefered using i32 or u32 for integer parameter.
 * Some Node library function, however, accept safe-integer and infinite as integer.
 * Integer covers such cases.
 */
class Integer {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntegerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_integer_free(ptr, 0);
    }
}
if (Symbol.dispose) Integer.prototype[Symbol.dispose] = Integer.prototype.free;
exports.Integer = Integer;

class WriteOption {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WriteOption.prototype);
        obj.__wbg_ptr = ptr;
        WriteOptionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WriteOptionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_writeoption_free(ptr, 0);
    }
}
if (Symbol.dispose) WriteOption.prototype[Symbol.dispose] = WriteOption.prototype.free;
exports.WriteOption = WriteOption;

/**
 * @returns {Promise<any>}
 */
function start() {
    const ret = wasm.start();
    return ret;
}
exports.start = start;

exports.__wbg_Error_52673b7de5a0ca89 = function(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

exports.__wbg___wbindgen_debug_string_adfb662ae34724b6 = function(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

exports.__wbg___wbindgen_is_function_8d400b8b1af978cd = function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

exports.__wbg___wbindgen_is_object_ce774f3490692386 = function(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

exports.__wbg___wbindgen_is_undefined_f6b95eab589e0269 = function(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

exports.__wbg___wbindgen_number_get_9619185a74197f95 = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

exports.__wbg___wbindgen_string_get_a2a31e16edf96e42 = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

exports.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

exports.__wbg__wbg_cb_unref_87dfb5aaa0cbcea7 = function(arg0) {
    arg0._wbg_cb_unref();
};

exports.__wbg_abort_07646c894ebbf2bd = function(arg0) {
    arg0.abort();
};

exports.__wbg_abort_399ecbcfd6ef3c8e = function(arg0, arg1) {
    arg0.abort(arg1);
};

exports.__wbg_append_c5cbdf46455cc776 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
    arg0.append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
}, arguments) };

exports.__wbg_arrayBuffer_c04af4fce566092d = function() { return handleError(function (arg0) {
    const ret = arg0.arrayBuffer();
    return ret;
}, arguments) };

exports.__wbg_buffer_6cb2fecb1f253d71 = function(arg0) {
    const ret = arg0.buffer;
    return ret;
};

exports.__wbg_byteLength_166ad9a51ecaa5f1 = function(arg0) {
    const ret = arg0.byteLength;
    return ret;
};

exports.__wbg_call_3020136f7a2d6e44 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

exports.__wbg_call_abb4ff46ce38be40 = function() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

exports.__wbg_clearTimeout_7a42b49784aea641 = function(arg0) {
    const ret = clearTimeout(arg0);
    return ret;
};

exports.__wbg_close_70c4165e1166967a = function(arg0) {
    const ret = arg0.close();
    return ret;
};

exports.__wbg_done_62ea16af4ce34b24 = function(arg0) {
    const ret = arg0.done;
    return ret;
};

exports.__wbg_fetch_74a3e84ebd2c9a0e = function(arg0) {
    const ret = fetch(arg0);
    return ret;
};

exports.__wbg_fetch_90447c28cc0b095e = function(arg0, arg1) {
    const ret = arg0.fetch(arg1);
    return ret;
};

exports.__wbg_getTime_ad1e9878a735af08 = function(arg0) {
    const ret = arg0.getTime();
    return ret;
};

exports.__wbg_get_af9dab7e9603ea93 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

exports.__wbg_has_0e670569d65d3a45 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.has(arg0, arg1);
    return ret;
}, arguments) };

exports.__wbg_headers_654c30e1bcccc552 = function(arg0) {
    const ret = arg0.headers;
    return ret;
};

exports.__wbg_importKey_d2e413c2af4484d1 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = arg0.importKey(getStringFromWasm0(arg1, arg2), arg3, arg4, arg5 !== 0, arg6);
    return ret;
}, arguments) };

exports.__wbg_instanceof_Response_cd74d1c2ac92cb0b = function(arg0) {
    let result;
    try {
        result = arg0 instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

exports.__wbg_iterator_27b7c8b35ab3e86b = function() {
    const ret = Symbol.iterator;
    return ret;
};

exports.__wbg_length_22ac23eaec9d8053 = function(arg0) {
    const ret = arg0.length;
    return ret;
};

exports.__wbg_new_0_23cedd11d9b40c9d = function() {
    const ret = new Date();
    return ret;
};

exports.__wbg_new_1ba21ce319a06297 = function() {
    const ret = new Object();
    return ret;
};

exports.__wbg_new_3c79b3bb1b32b7d3 = function() { return handleError(function () {
    const ret = new Headers();
    return ret;
}, arguments) };

exports.__wbg_new_6421f6084cc5bc5a = function(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
};

exports.__wbg_new_881a222c65f168fc = function() { return handleError(function () {
    const ret = new AbortController();
    return ret;
}, arguments) };

exports.__wbg_new_ff12d2b041fb48f1 = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return wasm_bindgen__convert__closures_____invoke__h72f6d572297204b0(a, state0.b, arg0, arg1);
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

exports.__wbg_new_from_slice_f9c22b9153b26992 = function(arg0, arg1) {
    const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
    return ret;
};

exports.__wbg_new_no_args_cb138f77cf6151ee = function(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return ret;
};

exports.__wbg_new_with_byte_offset_and_length_d85c3da1fd8df149 = function(arg0, arg1, arg2) {
    const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
    return ret;
};

exports.__wbg_new_with_str_and_init_c5748f76f5108934 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
    return ret;
}, arguments) };

exports.__wbg_next_138a17bbf04e926c = function(arg0) {
    const ret = arg0.next;
    return ret;
};

exports.__wbg_next_3cfe5c0fe2a4cc53 = function() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments) };

exports.__wbg_open_d1405bcd72f3f33b = function(arg0, arg1, arg2, arg3, arg4) {
    const ret = open(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), arg4 >>> 0);
    return ret;
};

exports.__wbg_prototypesetcall_dfe9b766cdc1f1fd = function(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
};

exports.__wbg_queueMicrotask_9b549dfce8865860 = function(arg0) {
    const ret = arg0.queueMicrotask;
    return ret;
};

exports.__wbg_queueMicrotask_fca69f5bfad613a5 = function(arg0) {
    queueMicrotask(arg0);
};

exports.__wbg_resolve_fd5bfbaa4ce36e1e = function(arg0) {
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

exports.__wbg_set_body_8e743242d6076a4f = function(arg0, arg1) {
    arg0.body = arg1;
};

exports.__wbg_set_cache_0e437c7c8e838b9b = function(arg0, arg1) {
    arg0.cache = __wbindgen_enum_RequestCache[arg1];
};

exports.__wbg_set_credentials_55ae7c3c106fd5be = function(arg0, arg1) {
    arg0.credentials = __wbindgen_enum_RequestCredentials[arg1];
};

exports.__wbg_set_headers_5671cf088e114d2b = function(arg0, arg1) {
    arg0.headers = arg1;
};

exports.__wbg_set_method_76c69e41b3570627 = function(arg0, arg1, arg2) {
    arg0.method = getStringFromWasm0(arg1, arg2);
};

exports.__wbg_set_mode_611016a6818fc690 = function(arg0, arg1) {
    arg0.mode = __wbindgen_enum_RequestMode[arg1];
};

exports.__wbg_set_signal_e89be862d0091009 = function(arg0, arg1) {
    arg0.signal = arg1;
};

exports.__wbg_sign_8577ce5795fd016a = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
    const ret = arg0.sign(getStringFromWasm0(arg1, arg2), arg3, getArrayU8FromWasm0(arg4, arg5));
    return ret;
}, arguments) };

exports.__wbg_signal_3c14fbdc89694b39 = function(arg0) {
    const ret = arg0.signal;
    return ret;
};

exports.__wbg_static_accessor_CRYPTO_251bf8f194f7249c = function() {
    const ret = crypto;
    return ret;
};

exports.__wbg_static_accessor_ENV_a76308d71f30158c = function() {
    const ret = env;
    return ret;
};

exports.__wbg_static_accessor_GLOBAL_769e6b65d6557335 = function() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_static_accessor_GLOBAL_THIS_60cf02db4de8e1c1 = function() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_static_accessor_SELF_08f5a74c69739274 = function() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_static_accessor_STDOUT_05465658486c30ce = function() {
    const ret = stdout;
    return ret;
};

exports.__wbg_static_accessor_WINDOW_a8924b26aa92d024 = function() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

exports.__wbg_status_9bfc680efca4bdfd = function(arg0) {
    const ret = arg0.status;
    return ret;
};

exports.__wbg_stringify_655a6390e1f5eb6b = function() { return handleError(function (arg0) {
    const ret = JSON.stringify(arg0);
    return ret;
}, arguments) };

exports.__wbg_subtle_0109c00de0ea1788 = function(arg0) {
    const ret = arg0.subtle;
    return ret;
};

exports.__wbg_then_429f7caf1026411d = function(arg0, arg1, arg2) {
    const ret = arg0.then(arg1, arg2);
    return ret;
};

exports.__wbg_then_4f95312d68691235 = function(arg0, arg1) {
    const ret = arg0.then(arg1);
    return ret;
};

exports.__wbg_url_b6d11838a4f95198 = function(arg0, arg1) {
    const ret = arg1.url;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

exports.__wbg_value_57b7b035e117f7ee = function(arg0) {
    const ret = arg0.value;
    return ret;
};

exports.__wbg_write_1552ccd3734e9625 = function(arg0, arg1, arg2) {
    const ret = arg0.write(arg1, WriteOption.__wrap(arg2));
    return ret;
};

exports.__wbg_write_bb96f4bd34a4fe6e = function(arg0, arg1, arg2) {
    const ret = arg0.write(arg1, arg2);
    return ret;
};

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
    const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__hd1b9e485ae067070, wasm_bindgen__convert__closures_____invoke__he91f9a0170836f5d);
    return ret;
};

exports.__wbindgen_cast_c1093d7c8dafc10c = function(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { dtor_idx: 21, function: Function { arguments: [Externref], shim_idx: 82, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__hd1b9e485ae067070, wasm_bindgen__convert__closures_____invoke__h1b6609413577a3fc);
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
};

const wasmPath = `${__dirname}/gh_token_gen_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
const wasm = exports.__wasm = new WebAssembly.Instance(wasmModule, imports).exports;

wasm.__wbindgen_start();
