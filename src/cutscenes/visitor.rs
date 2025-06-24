use crate::cutscene::{chara::Character, fragments::IntoBox};
use bevy::prelude::*;
use bevy_sequence::prelude::*;

#[derive(Debug, Component)]
pub struct FrontDoorCutscene;

pub fn visitor() -> impl IntoBox<FrontDoorCutscene> {
    (
        500,
        "Hello?".father(),
        1000,
        "Hey, man.".stranger(),
        "How are you doing?".stranger(),
        1000,
        "I thought I'd swing by and check in on you.".stranger(),
        "Oh, well...".father(),
        "That's very kind of you. I'm doing well.".father(),
        1000,
        "How's Luna?".stranger(),
        "Oh she's becoming a real artist!".father(),
        "Finally picking up a thing or two from her old man.".father(),
        1000,
        "That's nice.".stranger(),
        1000,
        "Well, if you need anything, just give me a call.".stranger(),
        "Be seeing you.".stranger(),
        500,
    )
        .always()
        .once()
}
