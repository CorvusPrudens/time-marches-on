use bevy::prelude::*;
use bevy_sequence::{fragment::DataLeaf, prelude::*};
use std::{marker::PhantomData, time::Duration};

use crate::textbox::TextboxCloseEvent;

pub trait IntoBox<C = EmptyCutscene>: IntoFragment<CutsceneFragment, TextBoxContext<C>> {
    fn spawn_box(self, commands: &mut Commands);
}

impl<C, T> IntoBox<C> for T
where
    T: IntoFragment<CutsceneFragment, TextBoxContext<C>>,
    C: 'static,
{
    fn spawn_box(self, commands: &mut Commands) {
        spawn_root_with(
            self.on_end(|mut writer: EventWriter<TextboxCloseEvent>| {
                writer.write(TextboxCloseEvent);
            }),
            commands,
            TextBoxContext::new(),
        );
    }
}

#[derive(Debug, Component)]
pub struct EmptyCutscene;

#[derive(Debug, Component)]
pub struct TextBoxContext<Cutscene = EmptyCutscene>(PhantomData<fn() -> Cutscene>);

impl<C> Clone for TextBoxContext<C> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl<C> TextBoxContext<C> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

#[derive(Debug, Clone)]
pub enum CutsceneFragment {
    Dialog(String),
    Pause(Duration),
}

impl From<f32> for CutsceneFragment {
    fn from(value: f32) -> Self {
        Self::Pause(Duration::from_secs_f32(value))
    }
}

impl From<Duration> for CutsceneFragment {
    fn from(value: Duration) -> Self {
        Self::Pause(value)
    }
}

impl From<String> for CutsceneFragment {
    fn from(value: String) -> Self {
        Self::Dialog(value)
    }
}

impl<'a> From<&'a str> for CutsceneFragment {
    fn from(value: &'a str) -> Self {
        Self::Dialog(value.into())
    }
}

macro_rules! impl_into_frag {
    ($ty:ty, $x:ident, $into:expr) => {
        impl<C> IntoFragment<CutsceneFragment, TextBoxContext<C>> for $ty {
            fn into_fragment(
                self,
                context: &Context<TextBoxContext<C>>,
                commands: &mut Commands,
            ) -> FragmentId {
                let $x = self;
                <_ as IntoFragment<CutsceneFragment, TextBoxContext<C>>>::into_fragment(
                    DataLeaf::new($into),
                    context,
                    commands,
                )
            }
        }
    };
}

impl_into_frag!(&'static str, slf, slf);
impl_into_frag!(String, slf, slf);
impl_into_frag!(Duration, slf, slf);
impl_into_frag!(f32, slf, slf);
