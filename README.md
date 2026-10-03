# Snake Game in Rust (SDL2)

## Overview
Snake game implemented in [Rust](https://www.rust-lang.org/learn/get-started) using the [SDL2](https://wiki.libsdl.org/SDL2/FrontPage) library. Collect eggs to grow longer while avoiding self-collision. The game supports teleporting at screen edges, and visual feedback when attempting illegal moves.

## Features
- Title screen, pause, game over and instant restart (ENTER)
- Cyan eggs (+1) and yellow eggs (+3, they vanish after a few seconds)
- Red viruses: each hit costs a life and part of your tail (3 lives)
- Levels: speed rises and more viruses appear every 10 points
- Persistent high score (`highscore.txt`)
- Queued turns, so fast key presses can never reverse the snake into itself
- Edge wrap-around

## Requirements
To run this game, you need:
- Rust and Cargo installed
- SDL2 installed on your system
- SDL2_ttf for text rendering

## Installation
1. Clone this repository:
   ```sh
   git clone https://github.com/rubbieKelvin/snake.git
   cd snake
   ```
2. Install dependencies:
   ```sh
   cargo build
   ```
3. Run the game:
   ```sh
   cargo run
   ```

## Controls
| Key      | Action               |
|----------|----------------------|
| W / UP   | Move Up              |
| A / LEFT | Move Left            |
| S / DOWN | Move Down            |
| D / RIGHT| Move Right           |
| P        | Pause/Resume Game    |
| ESC      | Quit the Game        |

## How to Play
- Move with WASD or the arrow keys; ENTER starts or restarts, `P` pauses.
- Eat eggs to grow and score; avoid viruses and your own body.
- Biting yourself or losing all lives ends the game.

## Dependencies
The game uses the following Rust crates:
- `sdl2` for graphics, events, and rendering
- `sdl2::ttf` for text rendering
- `rand` for generating random positions

## Future Improvements
- Sound effects (needs SDL2_mixer)
- Textures/sprites
- Difficulty selection and power-ups

## License
This project is licensed under the MIT License.


