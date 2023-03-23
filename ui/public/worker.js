importScripts('gomoku.js');

function load_wasm() {
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
  
    return wasm_bindgen('./gomoku_bg.wasm')
}

onmessage = message => {
    let initialised = load_wasm().catch(err => {
      // Propagate to main `onerror`:
      setTimeout(() => {
        throw err;
      });
      // Rethrow to keep promise rejected and prevent execution of further commands:
      throw err;
    });

    self.onmessage = async event => {
        // This will queue further commands up until the module is fully initialised:
        await initialised;

        const { think_and_move, make_move, rank_of, file_of, foul_moves, check_wld_already } = wasm_bindgen;

        let funcs = {
            'think_and_move': think_and_move,
            'make_move': make_move,
            'rank_of': rank_of,
            'file_of': file_of,
            'foul_moves': foul_moves,
            'check_wld_already': check_wld_already,
        };
        const { func_name, args } = JSON.parse(event.data);
        result = funcs[func_name](...Object.values(args));
        postMessage(result);
    };
}