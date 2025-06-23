use bevy::prelude::*;
use bevy_sequence::{fragment::DataLeaf, prelude::*};
use std::marker::PhantomData;

pub trait IntoBox<C = EmptyCutscene>: IntoFragment<SectionFrag, TextBoxContext<C>> {
    fn spawn_box(self, commands: &mut Commands);
    // fn spawn_box_with(self, commands: &mut Commands, _root: C);
    // fn textbox(self) -> impl IntoBox<C>;
    // fn textbox_with(
    //     self,
    //     f: impl Fn(Res<AssetServer>, &mut Commands) + 'static + Send + Sync,
    // ) -> impl IntoBox<C>;
}

impl<C, T> IntoBox<C> for T
where
    T: IntoFragment<SectionFrag, TextBoxContext<C>>,
    C: 'static,
{
    fn spawn_box(self, commands: &mut Commands) {
        spawn_root_with(self, commands, TextBoxContext::new());
    }

    // fn spawn_box_with(self, commands: &mut Commands, _root: C) {
    //     self.spawn_box(commands);
    // }
    //
    // fn textbox(self) -> impl IntoBox<C> {
    //     self.textbox_with(crate::textbox::spawn_textbox)
    // }
    //
    // fn textbox_with(
    //     self,
    //     f: impl Fn(Res<AssetServer>, &mut Commands) + 'static + Send + Sync,
    // ) -> impl IntoBox<C> {
    //     self.on_start(
    //         move |InRef(ctx): InRef<TextBoxContext<C>>,
    //               mut commands: Commands,
    //               asset_server: Res<AssetServer>| {
    //             f(asset_server, &mut commands);
    //         },
    //     )
    //     .on_end(
    //         |InRef(ctx): InRef<TextBoxContext<C>>, mut commands: Commands| {
    //             todo!();
    //         },
    //     )
    // }
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

#[derive(Debug, Clone, Component)]
pub struct TextBoxEntity(Entity);

impl TextBoxEntity {
    pub fn entity(&self) -> Entity {
        self.0
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
// impl_into_frag!(TypeWriterSection, slf, slf);
