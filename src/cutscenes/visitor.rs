use crate::cutscene::{chara::Character, fragments::IntoBox};
use bevy::prelude::*;
use bevy_sequence::prelude::*;

#[derive(Debug, Component)]
pub struct FrontDoorCutscene;

pub fn visitor() -> impl IntoBox<FrontDoorCutscene> {
    (
        0.5,
        "Hello?".father(),
        1.0,
        "Hey, man.".stranger(),
        "How are you doing?".stranger(),
        1.0,
        "I thought I'd swing by and check in on you.".stranger(),
        "Oh, well...".father(),
        "That's very kind of you. I'm doing well.".father(),
        1.0,
        "How's Luna?".stranger(),
        "Oh she's becoming a real artist!".father(),
        "Finally picking up a thing or two from her old man.".father(),
        1.0,
        "That's nice.".stranger(),
        1.0,
        "Well, if you need anything, just give me a call.".stranger(),
        "Be seeing you.".stranger(),
        0.5,
    )
        .always()
        .once()
}
