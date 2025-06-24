use bevy::prelude::*;
use bevy_seedling::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            |mut commands: Commands, server: Res<AssetServer>| {
                commands.spawn(
                    SamplePlayer::new(server.load("audio/music/quiet-halls.ogg"))
                        .with_volume(Volume::Decibels(-6.0))
                        .looping(),
                );
            },
        );
    }
}
