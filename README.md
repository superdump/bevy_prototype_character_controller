# bevy_prototype_character_controller

Implementations of character controllers that take common input events and map them to relative / absolute rotation and change in position (translation) / change in velocity (impulse) / change in acceleration (force).

This design is intended to allow use of these controller systems with whichever physics or camera setup you like. For example, you may use a kinematic rigid body in your physics engine and set its position based on the translation events, or a dynamic rigid body and apply the impulses or forces.

## TODO

- add indirect third-person modes
  - the pseudo-isometric example should make the body face the direction of movement
  - the third-person indirect example would have the player control the character and the follow camera would lag, seek and follow after
- support impulses and forces
- avoid clipping the camera through obstacles
  - ray or box cast and adjust the camera position
- add examples that use physics engines (rapier, physx)
- add juicy controllers that use Attack, Decay, Sustain, Release curves, with easing using bevy_easings
- move yaw / pitch / roll out to a component

## License

MIT. See the [LICENSE file](LICENSE).
