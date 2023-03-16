const concurrency = document.getElementById('concurrency');
const concurrencyAmt = document.getElementById('concurrency-amt');
const button = document.getElementById('button');

// First up, but try to do feature detection to provide better error messages
function loadWasm() {
  let msg = 'This demo requires a current version of Firefox (e.g., 79.0)';
  if (typeof SharedArrayBuffer !== 'function') {
    alert('this browser does not have SharedArrayBuffer support enabled' + '\n\n' + msg);
    return
  }
  // Test for bulk memory operations with passive data segments
  //  (module (memory 1) (data passive ""))
  const buf = new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x05, 0x03, 0x01, 0x00, 0x01, 0x0b, 0x03, 0x01, 0x01, 0x00]);
  if (!WebAssembly.validate(buf)) {
    alert('this browser does not support passive wasm memory, demo does not work' + '\n\n' + msg);
    return
  }

  wasm_bindgen('./gomoku_bg.wasm')
    .then(run)
    .catch(console.error);
}

loadWasm();

const { think_and_move, make_move, rank_of, file_of } = wasm_bindgen;

function run() {
  // The maximal concurrency of our web worker pool is `hardwareConcurrency`,
  // so set that up here and this ideally is the only location we create web
  // workers.
  // pool = new WorkerPool(navigator.hardwareConcurrency);
  // Configure various buttons and such.
  button.onclick = function() {
    // test();
    let next_move = think_and_move([make_move(7, 7)], 30);

    console.log([rank_of(next_move), file_of(next_move)]);
    // console.log("test");
  };

  concurrency.oninput = function() {
    concurrencyAmt.innerText = 'Concurrency: ' + concurrency.value;
  };
  concurrency.min = 1;
  concurrency.step = 1;
  concurrency.max = navigator.hardwareConcurrency;
  concurrency.value = concurrency.max;
  concurrency.oninput();
  concurrency.disabled = false;
}
