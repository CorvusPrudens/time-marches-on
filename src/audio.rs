use bevy::prelude::*;
use bevy_seedling::prelude::*;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            |mut commands: Commands,
             server: Res<AssetServer>,
             mut scale: ResMut<DefaultSpatialScale>| {
                commands.spawn(
                    SamplePlayer::new(server.load("audio/music/quiet-halls.ogg"))
                        // SamplePlayer::new(server.load("audio/music/luna.ogg"))
                        .with_volume(Volume::Decibels(-6.0))
                        .looping(),
                );

                scale.0 = Vec3::splat(0.1);

                commands.spawn((
                    SamplerPool(SpatialSound),
                    sample_effects![SpatialBasicNode::default()],
                ));
            },
        );
    }
}

#[derive(PoolLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpatialSound;
