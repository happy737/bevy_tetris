# Tetris
This project is a simple Tetris game, visualized in the bevy engine for rust. The game is visualized as a bunch of 3d cubes, which replace the usual sqares, but otherwise holds itself to the usual Tetris formula. The code base is split into multiple segments, with the core Tetris model being completely independent of the bevy implementation, bevy only manages the visualization and user interaction. Theoretically, the model could be taken out of the game and grafted into another user abstraction, porting the game to let's say a terminal. The other two domains are UI and bridging the gap between the tetris model and the user, both of which make much use of bevys ECS. 

Upon starting the executable, it automatically starts the game with the following control scheme: 
- Down: S
- Left: A
- Right: D
- Drop all the way: Space
- Rotate Counterclockwise: Q
- Rotate Clockwise: E
- Store the active piece: W

These settings are however not set in stone and can be reassigned in the pause menu via Escape. 