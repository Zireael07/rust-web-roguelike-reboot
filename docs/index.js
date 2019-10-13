// The global itself is the initialization
// function and then the properties of the global are all the exported
// functions.
//const { wasm } = wasm_bindgen;

async function run() {
    await wasm_bindgen('./rust-web-roguelike_bg.wasm');
    //initButtons();
  }

  run();

// const initButtons = () => {
//     //it used to contain click listeners
//     }