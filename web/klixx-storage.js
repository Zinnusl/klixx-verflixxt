(function () {
  "use strict";

  const decoder = new TextDecoder("utf-8");
  const encoder = new TextEncoder();
  const missing = 0xffffffff;

  function bytes(ptr, len) {
    return new Uint8Array(wasm_memory.buffer, ptr, len);
  }

  function readString(ptr, len) {
    return decoder.decode(bytes(ptr, len));
  }

  function encodedValue(keyPtr, keyLen) {
    const value = localStorage.getItem(readString(keyPtr, keyLen));
    return value === null ? null : encoder.encode(value);
  }

  function register(importObject) {
    importObject.env.klixx_storage_get_len = function (keyPtr, keyLen) {
      const value = encodedValue(keyPtr, keyLen);
      return value === null ? missing : value.length;
    };

    importObject.env.klixx_storage_get = function (keyPtr, keyLen, outPtr, outLen) {
      const value = encodedValue(keyPtr, keyLen);
      if (value === null) return 0;

      const out = bytes(outPtr, outLen);
      const count = Math.min(value.length, outLen);
      out.set(value.subarray(0, count));
      return count;
    };

    importObject.env.klixx_storage_set = function (keyPtr, keyLen, valuePtr, valueLen) {
      localStorage.setItem(readString(keyPtr, keyLen), readString(valuePtr, valueLen));
    };

    importObject.env.klixx_storage_remove = function (keyPtr, keyLen) {
      localStorage.removeItem(readString(keyPtr, keyLen));
    };

    importObject.env.klixx_loading_complete = function () {
      const el = document.getElementById("loading");
      if (el) el.style.display = "none";
    };
  }

  miniquad_add_plugin({
    register_plugin: register,
    name: "klixx_storage",
    version: 1,
  });
})();
