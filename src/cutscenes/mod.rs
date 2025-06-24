use bevy::prelude::*;
use bevy_sequence::combinators::delay::run_after;
use std::time::Duration;

pub mod tea;
pub mod visitor;

/// This is mainly useful for quick testing at the moment.
pub struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, move |mut commands: Commands| {
            run_after(
                Duration::from_secs(2),
                |mut commands: Commands| {
                    // tea_cutscene().spawn_box(&mut commands);
                },
                &mut commands,
            );
        });
    }
}
