mod features;
mod foundry;

use crate::features::init_features;

const ID: &str = "johnys-module";
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn main() {
    init_features();
}
