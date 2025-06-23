use avian2d::prelude::*;
use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_optix::zorder::YOrigin;

use crate::world;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components::<world::Piano, Piano>()
            .register_required_components::<world::Easel1, Easel>()
            .register_required_components::<world::Easel2, Easel>()
            .register_required_components::<world::Easel3, Easel>();
    }
}

#[derive(Default, Component)]
#[require(RigidBody::Static, Collider::rectangle(32., 16.), YOrigin(-8.))]
struct Piano;

#[derive(Default, Component)]
#[require(RigidBody::Static, YOrigin(-32.))]
#[component(on_add = Self::add)]
struct Easel;

impl Easel {
    fn add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().entity(ctx.entity).with_child((
            Collider::rectangle(8., 8.),
            Transform::from_xyz(8., -24., 0.),
        ));
    }
}
