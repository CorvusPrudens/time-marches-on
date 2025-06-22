use bevy::prelude::*;
use bevy_sequence::Threaded;
use std::any::TypeId;

mod movement;

pub struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        let mut cache = movement::MovementSystemCache::default();
        cache.0.insert(TypeId::of::<EasingCurve<Vec3>>());

        app.insert_resource(cache).add_systems(
            PostUpdate,
            movement::apply_movements::<EasingCurve<Vec3>>
                .before(TransformSystem::TransformPropagate),
        );
    }
}

pub trait IntoCurve<C> {
    fn into_curve(&self, start: Vec3, end: Vec3) -> impl Curve<Vec3> + Threaded;
}

impl IntoCurve<EasingCurve<Vec3>> for EaseFunction {
    fn into_curve(&self, start: Vec3, end: Vec3) -> impl Curve<Vec3> + Threaded {
        EasingCurve::new(start, end, *self)
    }
}
