use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_tween::prelude::Interpolator;

#[derive(Debug, Default, Clone, PartialEq, Reflect)]
pub struct InterpolateVolume {
    start: f32,
    end: f32,
}

pub fn volume(start: f32, end: f32) -> InterpolateVolume {
    InterpolateVolume { start, end }
}

pub fn volume_to(to: f32) -> impl Fn(&mut f32) -> InterpolateVolume {
    move |state| {
        let start = *state;

        let end = to;
        *state = to;
        volume(start, end)
    }
}

impl Interpolator for InterpolateVolume {
    type Item = VolumeNode;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.volume = Volume::Decibels(self.start.lerp(self.end, value));
    }
}

#[derive(Debug, Default, Clone, PartialEq, Reflect)]
pub struct InterpolateLowPass {
    start: f32,
    end: f32,
}

pub fn low_pass(start: f32, end: f32) -> InterpolateLowPass {
    InterpolateLowPass { start, end }
}

pub fn low_pass_to(to: f32) -> impl Fn(&mut f32) -> InterpolateLowPass {
    move |state| {
        let start = *state;

        let end = to;
        *state = to;
        low_pass(start, end)
    }
}

impl Interpolator for InterpolateLowPass {
    type Item = LowPassNode;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.frequency = self.start.lerp(self.end, value);
    }
}

#[derive(Debug, Default, Clone, PartialEq, Reflect)]
pub struct InterpolateSampleSpeed {
    start: f32,
    end: f32,
}

pub fn sample_speed(start: f32, end: f32) -> InterpolateSampleSpeed {
    InterpolateSampleSpeed { start, end }
}

pub fn sample_speed_to(to: f32) -> impl Fn(&mut f32) -> InterpolateSampleSpeed {
    move |state: &mut f32| {
        let start = *state;

        let end = to;
        *state = to;
        sample_speed(start, end)
    }
}

impl Interpolator for InterpolateSampleSpeed {
    type Item = PlaybackSettings;

    fn interpolate(&self, item: &mut Self::Item, value: f32) {
        item.speed = self.start.lerp(self.end, value) as f64;
    }
}
