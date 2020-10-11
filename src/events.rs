use bevy::prelude::*;
use std::ops::Deref;

#[derive(Debug)]
pub struct LookDeltaEvent {
    rotation_delta: Vec3,
}

impl LookDeltaEvent {
    pub fn new(other: &Vec3) -> Self {
        Self {
            rotation_delta: *other,
        }
    }
}

impl Deref for LookDeltaEvent {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.rotation_delta
    }
}

#[derive(Debug)]
pub struct LookEvent {
    rotation: Vec3,
}

impl LookEvent {
    pub fn new(other: &Vec3) -> Self {
        Self { rotation: *other }
    }
}

impl Deref for LookEvent {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.rotation
    }
}

#[derive(Debug)]
pub struct PitchEvent {
    pitch: f32,
}

impl PitchEvent {
    pub fn new(value: f32) -> Self {
        Self { pitch: value }
    }
}

impl Deref for PitchEvent {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.pitch
    }
}

#[derive(Debug)]
pub struct YawEvent {
    yaw: f32,
}

impl YawEvent {
    pub fn new(value: f32) -> Self {
        Self { yaw: value }
    }
}

impl Deref for YawEvent {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.yaw
    }
}

#[derive(Debug)]
pub struct TranslationEvent {
    translation: Vec3,
}

impl TranslationEvent {
    pub fn new(other: &Vec3) -> Self {
        Self {
            translation: *other,
        }
    }
}

impl Deref for TranslationEvent {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.translation
    }
}
