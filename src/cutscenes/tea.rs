use crate::cutscene::{chara::Character, fragments::IntoBox};
use bevy::prelude::*;
use bevy_sequence::prelude::*;

#[derive(Debug, Component)]
pub struct TeaCutscene;

pub fn tea_cutscene() -> impl IntoBox<TeaCutscene> {
    (
        "Oh, Luna, there you are.".father(),
        "Hey dad! I made some tea.".luna(),
        1500,
        "Well come on then, sit down.".luna(),
    )
        .always()
        .once()
}
