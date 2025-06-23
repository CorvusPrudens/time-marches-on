use bevy::prelude::*;
use bevy_sequence::{fragment::DataLeaf, prelude::*};
use std::marker::PhantomData;

use crate::textbox::TextboxCloseEvent;

pub trait IntoBox<C = EmptyCutscene>: IntoFragment<SectionFrag, TextBoxContext<C>> {
    fn spawn_box(self, commands: &mut Commands);
}

impl<C, T> IntoBox<C> for T
where
    T: IntoFragment<SectionFrag, TextBoxContext<C>>,
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
pub struct SectionFrag {
    pub section: String,
}

macro_rules! impl_into_frag {
    ($ty:ty, $x:ident, $into:expr) => {
        impl<C> IntoFragment<SectionFrag, TextBoxContext<C>> for $ty {
            fn into_fragment(
                self,
                context: &Context<TextBoxContext<C>>,
                commands: &mut Commands,
            ) -> FragmentId {
                let $x = self;
                <_ as IntoFragment<SectionFrag, TextBoxContext<C>>>::into_fragment(
                    DataLeaf::new(SectionFrag { section: $into }),
                    context,
                    commands,
                )
            }
        }
    };
}

impl_into_frag!(&'static str, slf, slf.into());
impl_into_frag!(String, slf, slf.into());
