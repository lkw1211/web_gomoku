declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	/**
	*/
	export function start(): void;
	/**
	* @param {any} ms
	* @param {number} tl
	* @returns {number}
	*/
	export function think_and_move(ms: any, tl: number): number;
	/**
	* @param {number} r
	* @param {number} f
	* @returns {number}
	*/
	export function make_move(r: number, f: number): number;
	/**
	* @param {number} m
	* @returns {number}
	*/
	export function rank_of(m: number): number;
	/**
	* @param {number} m
	* @returns {number}
	*/
	export function file_of(m: number): number;
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly start: () => void;
  readonly think_and_move: (a: number, b: number, c: number) => void;
  readonly make_move: (a: number, b: number) => number;
  readonly rank_of: (a: number) => number;
  readonly file_of: (a: number) => number;
  readonly memory: WebAssembly.Memory;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __wbindgen_thread_destroy: (a: number, b: number) => void;
  readonly __wbindgen_start: () => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
* @param {WebAssembly.Memory} maybe_memory
*
* @returns {Promise<InitOutput>}
*/
declare function wasm_bindgen (module_or_path?: InitInput | Promise<InitInput>, maybe_memory?: WebAssembly.Memory): Promise<InitOutput>;
