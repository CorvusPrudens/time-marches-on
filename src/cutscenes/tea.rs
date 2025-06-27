use crate::{
    audio::MusicPool,
    cutscene::{chara::Character, fragments::IntoBox},
};
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_sequence::{combinators::delay::run_after, prelude::*};
use bevy_tween::{combinator::tween, prelude::*};

#[derive(Debug, Component)]
pub struct TeaCutscene;

// -- luna wants to take a trip to the mountains together
// -- father re-tells one of his stories again

pub fn tea_cutscene() -> impl IntoBox<TeaCutscene> {
    (
        (
            "Oh, Luna, there you are."
                .father()
                .on_start(fade_out_music(3.5)),
            "Hey dad! I made some tea.".luna(),
            1.5,
            "Well come on then, sit down.".luna(),
            1.5,
            "Or... not, haha."
                .luna()
                .on_end(|mut commands: Commands, server: Res<AssetServer>| {
                    commands.spawn((
                        SamplePlayer::new(server.load("audio/music/luna.ogg"))
                            .looping()
                            .with_volume(Volume::Decibels(-6.0)),
                        MusicPool,
                    ));
                }),
            2.5,
            "You know, it's been a while since we visited the mountains.".luna(),
            "Those fishing rods ARE getting a little dusty.".father(),
            "I was thinking, well... maybe we could take a trip this weekend!".luna(),
            1.5,
            "Maybe if we reschedule your checkup, then...".luna(),
        ),
        (
            "Heh heh, feeling a little cooped up, are ya?".father(),
            "Well, I'll see what I can do, little birdy!".father(),
            "(I TOLD you to stop calling me that?)".luna(),
            "(Tweet, tweet!)".father(),
            2.0,
            "Did I ever tell you about that time I almost caught a sturgeon?".father(),
            "You probably did, but you regale her anyway.".narrator(),
            "This sucker was MASSIVE — bigger than me!".father(),
            "And you know darn well I didn't reel it in. It just leapt right into my boat."
                .father(),
            "I couldn't believe it!".father(),
            "But it managed to give you the slip, huh?".luna(),
            "Well, you know... this was a BIG guy. Didn't take much for him to flop outta there."
                .father(),
            "Even gave me a good wallop on the way out!"
                .father()
                .on_end(|mut commands: Commands, server: Res<AssetServer>| {
                    commands.spawn(SamplePlayer::new(server.load("audio/sfx/laugh.wav")));
                }),
            2.5,
            "Thanks for the tea, honey."
                .father()
                .on_end(fade_out_music(3.5)),
        ),
    )
        .always()
        .once()
}

fn fade_out_music(
    seconds: f32,
) -> impl Fn(Single<(Entity, &VolumeNode), With<SamplerPool<MusicPool>>>, Commands) {
    move |music: Single<(Entity, &VolumeNode), With<SamplerPool<MusicPool>>>,
          mut commands: Commands| {
        let (music, current_volume) = music.into_inner();

        let duration = Duration::from_secs_f32(seconds);

        let mut target = music.into_target().state(current_volume.volume.decibels());
        commands.animation().insert(tween(
            duration,
            EaseKind::Linear,
            target.with(crate::audio::tween::volume_to(-48.0)),
        ));

        run_after(
            duration + Duration::from_millis(32),
            |samples: Query<Entity, (With<SamplePlayer>, With<MusicPool>)>,
             mut music: Single<&mut VolumeNode, With<SamplerPool<MusicPool>>>,
             mut commands: Commands| {
                for sample in &samples {
                    commands.entity(sample).despawn();
                }

                music.volume = Volume::Decibels(0.0);
            },
            &mut commands,
        );
    }
}

pub fn fade_in_music(
    seconds: f32,
) -> impl Fn(Single<Entity, With<SamplerPool<MusicPool>>>, Commands) {
    move |music: Single<Entity, With<SamplerPool<MusicPool>>>, mut commands: Commands| {
        let music = music.into_inner();

        let duration = Duration::from_secs_f32(seconds);

        let mut target = music.into_target().state(-48.0);
        commands.animation().insert(tween(
            duration,
            EaseKind::Linear,
            target.with(crate::audio::tween::volume_to(0.0)),
        ));
    }
}
