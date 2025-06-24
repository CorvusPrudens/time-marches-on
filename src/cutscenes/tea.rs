use crate::cutscene::{chara::Character, fragments::IntoBox};
use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use bevy_sequence::prelude::*;

#[derive(Debug, Component)]
pub struct TeaCutscene;

// -- luna wants to take a trip to the mountains together
// -- father re-tells one of his stories again

pub fn tea_cutscene() -> impl IntoBox<TeaCutscene> {
    (
        (
            "Oh, Luna, there you are.".father(),
            "Hey dad! I made some tea.".luna(),
            1500,
            "Well come on then, sit down.".luna(),
            1500,
            "Or... not, haha.".luna(),
            1500,
            "You know, it's been a while since we visited the mountains.".luna(),
            "Those fishing rods ARE getting a little dusty.".father(),
            "I was thinking, well... maybe we could take a trip this weekend!".luna(),
            1500,
            "Maybe if we reschedule your checkup, then...".luna(),
        ),
        (
            "Heh heh, feeling a little cooped up, are ya?".father(),
            "Well, I'll see what I can do, little birdy!".father(),
            "(I TOLD you to stop calling me that?)".luna(),
            "(Tweet, tweet!)".father(),
            2000,
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
            2500,
            "Thanks for the tea, honey.".father(),
        ),
    )
        .always()
        .once()
}
