# Rust-Ascii-Snake
Another simple text graphics game written in rust- mainly for more practice with the language.

This program uses the `termion` crate to capture keyboard input and get cursor control; which as far as I know is only compatible with MacOS and Linux platforms.
So no Windows support, but I have heard it will be ported eventually.

### How to run this thing: 
<sup>(you will need cargo to be installed obviously) </sup>
1. CD into the project directory after cloning:
  ```
  git clone https://github.com/Da-Real-Kryall/Rust-Ascii-Snake/
  cd ./Rust-Ascii-Snake
  ```

2. run `cargo run --release` in CMD to build and run the program; it will begin immediately after you execute that command.

---

### How to play this thing:

The only controls are `WASD` and arrow keys for changing snake direction:
- `W` / `▲` points the snake upwards.
- `A` / `◄` points the snake leftwards.
- `S` / `▼` points the snake downwards.
- `D` / `►` points the snake rightwards.

I also took the liberty of adding a whole bunch of quality of life features to make gameplay a little bit better:
* If a keypress results in an immediate death, it will be ignored. <br>
  e.g. If the snake is running along the edge of the board, and you try to turn into the edge, nothing will happen.
* Keypresses are buffered to a set extent; if you are moving to the Right, and within a single game frame you press Down and Left, in the next two game frames the snake will move down and to the left.
* You are given a small amount of grace time to fix your mistakes if you end up hitting an edge; the snake will darken and you will stay alive for a small time before you actually lose.


### Have fun!
