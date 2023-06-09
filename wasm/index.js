// First up, but try to do feature detection to provide better error messages
async function check_wasm() {
  let msg = 'This demo requires a current version of Firefox (e.g., 79.0)';
  if (typeof SharedArrayBuffer !== 'function') {
    alert('this browser does not have SharedArrayBuffer support enabled' + '\n\n' + msg);
    return false
  }
  // Test for bulk memory operations with passive data segments
  //  (module (memory 1) (data passive ""))
  const buf = new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x05, 0x03, 0x01, 0x00, 0x01, 0x0b, 0x03, 0x01, 0x01, 0x00]);
  if (!WebAssembly.validate(buf)) {
    alert('this browser does not support passive wasm memory, demo does not work' + '\n\n' + msg);
    return false
  }

  return true
}

const { think_and_move } = wasm_bindgen;
async function _think_and_move(moves, time_limit) {
  if (await check_wasm()) {
    await wasm_bindgen('./gomoku_bg.wasm');
    return think_and_move(moves, time_limit);
  }
}