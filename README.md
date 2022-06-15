# Move To Target

A small game to learn keyboard input, meshes, and some more complex
organizational patterns.

## To Play

Follow the setup listed in [the Bevy Book][bevy]. This will get you setup with
Rust and Bevy on your particular platform better than I could.

The game has two players starting in the bottom left and right. To control the
left player use W, A, S, D to move. Q rotates counter clockwise and E rotates
clockwise. The corresponding keys for the right player are ↑, ←, ↓, →, ?, and
Shift.

The game ends when the two players roughly tile the target. Five seconds later
the game restarts.

[bevy]: https://bevyengine.org/learn/book/getting-started/setup/
