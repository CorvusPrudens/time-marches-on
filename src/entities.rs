use bevy::prelude::*;
use bevy_optix::zorder::YOrigin;

use crate::world;

pub struct EntityPlugin;

impl Plugin for EntityPlugin {
    fn build(&self, app: &mut App) {
        app.register_required_components_with::<world::TeddyBear, _>(|| YOrigin(-0.))
            .register_required_components_with::<world::TeddyBearBody, _>(|| YOrigin(-0.))
            .register_required_components_with::<world::TeddyBearHead, _>(|| YOrigin(-0.));
    }
}
