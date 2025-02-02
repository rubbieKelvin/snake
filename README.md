# Snake Game in Rust (SDL2)

## Overview
Snake game implemented in [Rust](https://www.rust-lang.org/learn/get-started) using the [SDL2](https://wiki.libsdl.org/SDL2/FrontPage) library. Collect eggs to grow longer while avoiding self-collision. The game supports teleporting at screen edges, and visual feedback when attempting illegal moves.

## Features
- Classic Snake gameplay
- Smooth movement with a timer-based system
- Randomly spawning collectible eggs
- Flash warning on invalid turns
- Edge teleportation
- Pause and resume functionality
- Score tracking

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
- Use the arrow keys (or WASD) to control the snake.
- Eat eggs to grow longer and increase your score.
- Avoid colliding with yourself.
- The snake can teleport through the screen edges.
- Press `P` to pause or resume the game.
- The game ends when the snake collides with itself.

## Dependencies
The game uses the following Rust crates:
- `sdl2` for graphics, events, and rendering
- `sdl2::ttf` for text rendering
- `rand` for generating random positions

## Future Improvements
- Add sound effects
- Improve graphics with textures
- Implement different difficulty levels
- Introduce additional collectibles with power-ups

## License
This project is licensed under the MIT License.


