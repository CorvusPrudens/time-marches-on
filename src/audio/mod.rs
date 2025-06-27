use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_tween::{BevyTweenRegisterSystems, component_dyn_tween_system, component_tween_system};

pub mod tween;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            |mut commands: Commands,
             server: Res<AssetServer>,
             default_pool: Single<Entity, With<SamplerPool<DefaultPool>>>,
             mut scale: ResMut<DefaultSpatialScale>| {
                // Create the SFX bus.
                commands.spawn((SfxBus, VolumeNode::default()));

                // Re-route the default pool to the SFX bus.
                commands
                    .entity(*default_pool)
                    .disconnect(MainBus)
                    .connect(SfxBus);

                // Adjust the default spatial scale for our pixel scale.
                scale.0 = Vec3::splat(0.1);

                commands
                    .spawn((
                        SamplerPool(SpatialPool),
                        sample_effects![SpatialBasicNode::default()],
                    ))
                    .connect(SfxBus);

                commands.spawn((
                    SamplerPool(MusicPool),
                    sample_effects![VolumeNode::default()],
                ));

                // commands.spawn((
                //     MusicPool,
                //     SamplePlayer::new(server.load("audio/music/quiet-halls.ogg"))
                //         // SamplePlayer::new(server.load("audio/music/luna.ogg"))
                //         .with_volume(Volume::Decibels(-6.0))
                //         .looping(),
                // ));
            },
        )
        .add_tween_systems((
            component_tween_system::<tween::InterpolateSampleSpeed>(),
            component_dyn_tween_system::<PlaybackSettings>(),
            component_tween_system::<tween::InterpolateLowPass>(),
            component_dyn_tween_system::<LowPassNode>(),
            component_tween_system::<tween::InterpolateVolume>(),
            component_dyn_tween_system::<VolumeNode>(),
        ));
    }
}

/// A pool for all spatial sounds.
#[derive(PoolLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SpatialPool;

/// A pool for music.
#[derive(PoolLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MusicPool;

/// Volume node through which all sound effects are routed.
#[derive(NodeLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SfxBus;
