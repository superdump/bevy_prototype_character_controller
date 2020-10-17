# bevy_prototype_character_controller

Implementations of character controllers that take common input events and map them to relative / absolute rotation and change in position (translation) / change in velocity (impulse) / change in acceleration (force).

This design is intended to allow use of these controller systems with whichever physics or camera setup you like. For example, you may use a kinematic rigid body in your physics engine and set its position based on the translation events, or a dynamic rigid body and apply the impulses or forces.

## Demos

### First-Person Character Controller
`cargo run --release --example first_person`
![First-person character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201011%20142238%20-%20Bevy%20First%20Person%20Character%20Controller.gif)

### Third-Person Character Controller
`cargo run --release --example third_person`
![Third-person character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201011%20142657%20-%20Bevy%20Third%20Person%20Character%20Controller.gif)

### Third-Person Pseudo-Isometric Character Controller
`cargo run --release --example pseudo_isometric`
![Third-person pseudo-isometric character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201011%20142953%20-%20Bevy%20Pseudo-Isometric%20Character%20Controller.gif)

### Rapier Third-Person Character Controller

You can choose between:
* `DynamicImpulse` - uses the `ImpulseEvent`s to apply impulses to the body
* `DynamicForce` - uses the `ForceEvent`s to apply forces to the body. This is the default.

Pitch and yaw are handled in the same way for both options.

`cargo run --release --example rapier3d -- DynamicImpulse`
![Third-person Rapier 3D dynamic impulse character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201017%20212543%20-%20Bevy%20Rapier%203D%20Dynamic%20Impulse%20Character%20Controller.gif)
`cargo run --release --example rapier3d -- DynamicForce`
![Third-person Rapier 3D dynamic force character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201017%20213044%20-%20Bevy%20Rapier%203D%20Dynamic%20Force%20Character%20Controller.gif)

### PhysX Third-Person Character Controller

You can choose between:
* `KinematicTranslation` - uses the `TranslationEvent`s to apply translations to the body
* `DynamicImpulse` - uses the `ImpulseEvent`s to apply impulses to the body
* `DynamicForce` - uses the `ForceEvent`s to apply forces to the body. This is the default.

Pitch and yaw are handled in the same way for both options.

`cargo run --release --example physx -- KinematicTranslation`
![Third-person PhysX kinematic translation character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201017%20224653%20-%20Bevy%20PhysX%20Kinematic%20Translation%20Character%20Controller.gif)
`cargo run --release --example physx -- DynamicImpulse`
![Third-person PhysX dynamic impulse character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201017%20224853%20-%20Bevy%20PhysX%20Dynamic%20Impulse%20Character%20Controller.gif)
`cargo run --release --example physx -- DynamicForce`
![Third-person PhysX dynamic force character controller demo](https://github.com/superdump/bevy_prototype_character_controller/raw/gh-pages/images/20201017%20225103%20Bevy%20PhysX%20Dynamic%20Force%20Character%20Controller.gif)

## How-To

See the examples for the different types of controller to get a better idea of how to use this stuff. Below is a description of an approach to structuring the hierarchy of entities necessary to make this system work.

### Entity structure

The structure of the Character is:
* body with Transform to set the position and yaw of the character, a CharacterController component and a tag so you can identify the body Transform for translating and yawing. Add rigid body and collider, or character controller components here.
  * body model
  * head with Transform offset upward to head position in y to give a point of reference for where the head / eyes are, and a tag so you can identify the head Transform for pitching. This design can be used for first- or third-person controllers.
    * head model
    * camera with Transform to offset for third person view like a camera boom arm stuck to the character's head, a LookDirection to get forward / right / up vectors, and a tag so you can identify the camera Transform

### Handling events

* When translating, the position of the body should be manipulated.
* When yawing (rotating about the y axis), the orientation of the body should be manipulated.
* When pitching (rotating about the right axis relative to the character), the orientation of the head should be manipulated.
* When zooming in and out (TODO) or changing the focal point (TODO - defaults to look at the head position for third-person), the translation and orientation of the camera boom should be manipulated.

## TODO

- use components rather than resources as appropriate to support multiple controllers in a scene (e.g. split-screen local co-op)
- add indirect third-person modes
  - the pseudo-isometric example should make the body face the direction of movement
  - the third-person indirect example would have the player control the character and the follow camera would lag, seek and follow after
  - allow temporary absolute orientation through pitch / yaw events
- avoid clipping the camera through obstacles
  - ray or box cast and adjust the camera position
- add juicy controllers that use Attack, Decay, Sustain, Release curves, with easing using `bevy_easings`

## License

MIT. See the [LICENSE file](LICENSE).
