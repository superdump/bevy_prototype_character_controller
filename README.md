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

## How-to

See the examples for the different types of controller to get a better idea of how to use this stuff. Below is a description of an approach to structuring the hierarchy of entities necessary to make this system work.

### Entity structure

The structure of the Character is:
* body with Transform to set the position and yaw of the character, a TranslationController component and a tag so you can identify the body Transform for translating and yawing. Add rigid body and collider, or character controller components here.
  * body model
  * head with Transform offset upward to head position in y to give a point of reference for where the head / eyes are, and a tag so you can identify the head Transform for pitching. This design can be used for first- or third-person controllers.
    * head model
    * camera with Transform to offset for third person view like a camera boom arm stuck to the character's head, a LookDirection to get forward / right / up vectors, and a tag so you can identify the camera Transform

### Handling events

* When translating, the position of the body should be manipulated.
* When yawing (rotating about the y axis), the orientation of the body should be manipulated.
* W*hen pitching (rotating about the right axis relative to the character), the orientation of the head should be manipulated.
* When zooming in and out (TODO) or changing the focal point (TODO - defaults to look at the head position for third-person), the translation and orientation of the camera boom should be manipulated.

## TODO

- refactoring - initially I thought I would have different files and plugins for look, translation, impulse, and force but it's looking like for the physical-feel they can be implemented in one system using the same physical model and emit the different events of interest
- use components rather than resources as appropriate to support multiple controllers in a scene (e.g. split-screen local co-op)
- add indirect third-person modes
  - the pseudo-isometric example should make the body face the direction of movement
  - the third-person indirect example would have the player control the character and the follow camera would lag, seek and follow after
  - allow temporary absolute orientation through pitch / yaw events
- support impulses and forces
- avoid clipping the camera through obstacles
  - ray or box cast and adjust the camera position
- add examples that use physics engines (`rapier` through `bevy_rapier3d`, `physx` through `bevy_prototype_physx`)
- add juicy controllers that use Attack, Decay, Sustain, Release curves, with easing using `bevy_easings`

## License

MIT. See the [LICENSE file](LICENSE).
