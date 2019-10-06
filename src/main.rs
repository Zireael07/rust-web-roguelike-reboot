extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

//the websys Canvas bindings uses it
use wasm_bindgen::JsCast; // for dyn_into
use std::f64;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
#[repr(u8)] //single byte representation - enough for our needs since a byte can carry 256 things
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    MoveLeft = 0,
    MoveRight = 1,
    MoveDown = 2,
    MoveUp = 3,
}

#[wasm_bindgen]
pub fn setCommand(com_id: i32){
        log!("{}", com_id);
        //need to match back to enum
        // https://stackoverflow.com/questions/28028854/how-do-i-match-enum-values-with-an-integer
        match com_id {
            com_id if com_id == Command::MoveLeft as i32 => log!("{:?}", Command::MoveLeft),
            com_id if com_id == Command::MoveRight as i32 => log!("{:?}", Command::MoveRight),
            com_id if com_id == Command::MoveDown as i32 => log!("{:?}", Command::MoveDown),
            com_id if com_id == Command::MoveUp as i32 => log!("{:?}", Command::MoveUp),
            _ => log!("Unknown command"),
}
}



pub fn main() {
 let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    log!("We have a context {:?}", context);

    //clear
    context.set_fill_style(&wasm_bindgen::JsValue::from_str("black"));
    context.fill_rect(0.0, 0.0, 800.0, 600.0);
}


// Auto-starts on page load
//start section of the executable may not literally point to main
#[wasm_bindgen(start)]
pub fn start() {
   main()
} 
