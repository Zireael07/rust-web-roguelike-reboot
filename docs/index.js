// The global itself is the initialization
// function and then the properties of the global are all the exported
// functions.
//const { wasm } = wasm_bindgen;

async function run() {
    await wasm_bindgen('./rust-web-roguelike_bg.wasm');
    initButtons();
  }

  run();

const initButtons = () => {
    //click listeners
    //no jQuery this time
    document.getElementsByClassName("key_arrow2")[0].addEventListener("click", event => {
        console.log("clicked up arrow");
        console.log(wasm_bindgen.Command);
        wasm_bindgen.setCommand(wasm_bindgen.Command.MoveUp);
        //console.log("After move: " + game.pos_x() + game.pos_y());
      });
      
      document.getElementsByClassName("key_arrow4")[0].addEventListener("click", event => {
        console.log("clicked left arrow");
        console.log(wasm_bindgen.Command);
        wasm_bindgen.setCommand(wasm_bindgen.Command.MoveLeft);
        //console.log("After move: " + game.pos_x() + game.pos_y());
      });
      
      document.getElementsByClassName("key_arrow6")[0].addEventListener("click", event => {
        console.log("clicked right arrow");
        console.log(wasm_bindgen.Command);
        wasm_bindgen.setCommand(wasm_bindgen.Command.MoveRight);
        //console.log("After move: " + game.pos_x() + " " + game.pos_y());
      });
      
      document.getElementsByClassName("key_arrow8")[0].addEventListener("click", event => {
        console.log("clicked down arrow");
        console.log(wasm_bindgen.Command);
        wasm_bindgen.setCommand(wasm_bindgen.Command.MoveDown);
        //console.log("After move: " + game.pos_x() + " " + game.pos_y());
      });
    }