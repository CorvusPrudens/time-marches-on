use super::fragments::IntoBox;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_sequence::prelude::FragmentExt;
use std::sync::Arc;

use crate::textbox::{CharacterEvent, CharacterSprite, glyph_sample};

#[derive(Clone, Copy)]
pub enum Chara {
    Narrator,
    DistressedNarrator1,
    DistressedNarrator2,
    Father,
    Luna,
    Stranger,
    Sturgeon,
    Shadow,
}

impl Chara {
    pub fn sprite(&self) -> Option<CharacterSprite> {
        match self {
            Self::Narrator => None,
            Self::DistressedNarrator1 => None,
            Self::DistressedNarrator2 => None,
            Self::Father => Some(CharacterSprite::new("main.png")),
            Self::Luna => Some(CharacterSprite::new("luna.png")),
            Self::Stranger => None,
            Self::Sturgeon => Some(CharacterSprite::new("sturgeon.png")),
            Self::Shadow => Some(CharacterSprite::new("shadow.png")),
        }
    }

    pub fn glyphs(&self) -> Arc<dyn Fn(&mut Commands, &AssetServer) + Send + Sync> {
        match self {
            Self::Narrator => Arc::new(move |commands, server| {
                commands.spawn((
                    PitchRange::new(0.02),
                    SamplePlayer {
                        sample: server.load(glyph_sample("low.wav")),
                        volume: Volume::Linear(0.5),
                        ..Default::default()
                    },
                ));
            }),
            Self::DistressedNarrator1 => Arc::new(move |commands, server| {
                commands.spawn((
                    PitchRange(1.0..1.15),
                    SamplePlayer {
                        sample: server.load(glyph_sample("low.wav")),
                        volume: Volume::Linear(0.6),
                        ..Default::default()
                    },
                ));
            }),
            Self::DistressedNarrator2 => Arc::new(move |commands, server| {
                commands.spawn((
                    PitchRange(1.0..1.30),
                    SamplePlayer {
                        sample: server.load(glyph_sample("low.wav")),
                        volume: Volume::Linear(0.7),
                        ..Default::default()
                    },
                ));
            }),
            Self::Father => Arc::new(move |commands, server| {
                commands.spawn((
                    PitchRange::new(0.02),
                    SamplePlayer {
                        sample: server.load(glyph_sample("medium.wav")),
                        volume: Volume::Linear(0.5),
                        ..Default::default()
                    },
                ));
            }),
            Self::Luna => Arc::new(move |commands, server| {
                commands.spawn((
                    PitchRange(0.75..0.85),
                    SamplePlayer {
                        sample: server.load(glyph_sample("high.wav")),
                        volume: Volume::Linear(0.5),
                        ..Default::default()
                    },
                ));
            }),
            Self::Stranger => Arc::new(move |commands, server| {
                commands.spawn((
                    PitchRange(0.75..0.85),
                    SamplePlayer {
                        sample: server.load(glyph_sample("low.wav")),
                        volume: Volume::Linear(0.5),
                        ..Default::default()
                    },
                ));
            }),
            Self::Sturgeon | Self::Shadow => Arc::new(move |commands, server| {
                commands.spawn((
                    PitchRange(0.45..0.75),
                    SamplePlayer {
                        sample: server.load(glyph_sample("low.wav")),
                        volume: Volume::Linear(0.5),
                        ..Default::default()
                    },
                ));
            }),
        }
    }
}

pub trait Character<C: Component>
where
    Self: IntoBox<C> + Sized,
{
    fn chara(self, chara: Chara) -> impl IntoBox<C> {
        self.on_start(move |mut writer: EventWriter<CharacterEvent>| {
            let sprite = chara.sprite();
            let glyph = chara.glyphs();

            writer.write(CharacterEvent { sprite, glyph });
        })
    }

    fn narrator(self) -> impl IntoBox<C> {
        self.chara(Chara::Narrator)
    }

    fn distressed_narrator(self) -> impl IntoBox<C> {
        self.chara(Chara::DistressedNarrator1)
    }

    fn distressed_narrator2(self) -> impl IntoBox<C> {
        self.chara(Chara::DistressedNarrator2)
    }

    fn father(self) -> impl IntoBox<C> {
        self.chara(Chara::Father)
    }

    fn luna(self) -> impl IntoBox<C> {
        self.chara(Chara::Luna)
    }

    fn stranger(self) -> impl IntoBox<C> {
        self.chara(Chara::Stranger)
    }

    fn shadow(self) -> impl IntoBox<C> {
        self.chara(Chara::Shadow)
    }
}

impl<T, C: Component> Character<C> for T where T: IntoBox<C> {}
