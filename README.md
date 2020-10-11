# bevy_prototype_character_controller

Implementations of character controllers that take common input events and map them to relative / absolute rotation and change in position (translation) / change in velocity (impulse) / change in acceleration (force).

This design is intended to allow use of these controller systems with whichever physics or camera setup you like. For example, you may use a kinematic rigid body in your physics engine and set its position based on the translation events, or a dynamic rigid body and apply the impulses or forces.

## Demos
### First-person Character Controller
`cargo run --release --example first_person`
![First-person character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201011%20142238%20-%20Bevy%20First%20Person%20Character%20Controller.gif)

### Third-person Character Controller
`cargo run --release --example third_person`
![Third-person character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201011%20142657%20-%20Bevy%20Third%20Person%20Character%20Controller.gif)

### Third-person Pseudo-isometric Character Controller
`cargo run --release --example pseudo_isometric`
![Third-person pseudo-isometric character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201011%20142953%20-%20Bevy%20Pseudo-Isometric%20Character%20Controller.gif)

## TODO

- refactoring - initially I thought I would have different files and plugins for look, translation, impulse, and force but it's looking like for the physical-feel they can be implemented in one system using the same physical model and emit the different events of interest
- use components rather than resources as appropriate to support multiple controllers in a scene (e.g. split-screen local co-op)
- add indirect third-person modes
  - the pseudo-isometric example should make the body face the direction of movement
  - the third-person indirect example would have the player control the character and the follow camera would lag, seek and follow after
- support impulses and forces
- avoid clipping the camera through obstacles
  - ray or box cast and adjust the camera position
- add examples that use physics engines (`rapier` through `bevy_rapier3d`, `physx` through `bevy_prototype_physx`)
- add juicy controllers that use Attack, Decay, Sustain, Release curves, with easing using `bevy_easings`

## License

MIT. See the [LICENSE file](LICENSE).
