use crate::cutscene::{chara::Character, fragments::IntoBox};
use bevy::prelude::*;
use bevy_sequence::prelude::*;

#[derive(Debug, Component)]
pub struct ParkCutscene;

pub fn park() -> impl IntoBox<ParkCutscene> {
    ("Luna!".father(), "Honey, hold up just a moment!".father())
        .always()
        .once()
}

pub fn park_man_one() -> impl IntoBox<ParkCutscene> {
    (
        "There's a man behind the tree.",
        1.0,
        "Hello.".stranger(),
        "Good evening, young man.".father(),
        "How did you get here?".stranger(),
        1.5,
        "Well I just, uh... I took a little walk, and...".father(),
        "Maybe... got turned around just a hair...".father(),
        "You're not supposed to be here.".stranger(),
        "Go home.".stranger(),
        2.0,
        "I'll be on my way, then.".father(),
    )
        .always()
        .once()
}

pub fn park_man_two() -> impl IntoBox<ParkCutscene> {
    (
        "Why are you letting her get away?".stranger(),
        "She's so fast, I...".father(),
        1.5,
        "Don't let her go.".stranger(),
        1.5,
        "Right.".father(),
    )
        .always()
        .once()
}
