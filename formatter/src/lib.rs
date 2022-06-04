mod utils;

use prettyplease;
use syn;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn pretty(code: String) -> String {
    let syntax_tree = syn::parse_file(&code).unwrap();
    let formatted = prettyplease::unparse(&syntax_tree);
    formatted
}
