use bevy::prelude::*;
use bevy_seedling::{
    prelude::Volume,
    sample::{PitchRange, SamplePlayer},
};
use bevy_sequence::{
    Threaded,
    combinators::delay::run_after,
    prelude::{FragmentEndEvent, FragmentEvent, FragmentExt, FragmentId, IdPair},
};
use fragments::IntoBox;
use std::{any::TypeId, collections::VecDeque, time::Duration};

use crate::textbox::{TextBlurb, TextboxClosedEvent, TextboxEvent, glyph_sample};

mod fragments;
mod movement;

pub struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        let mut cache = movement::MovementSystemCache::default();
        cache.0.insert(TypeId::of::<EasingCurve<Vec3>>());

        app.insert_resource(cache)
            .add_event::<FragmentEvent<fragments::SectionFrag>>()
            .add_systems(
                PostUpdate,
                movement::apply_movements::<EasingCurve<Vec3>>
                    .before(TransformSystem::TransformPropagate),
            )
            .add_systems(Update, fragment_bridge);
        // .add_systems(Startup, |mut commands: Commands| {
        //     run_after(Duration::from_millis(500), fragment_test, &mut commands);
        // });
    }
}

pub trait IntoCurve<C> {
    fn into_curve(&self, start: Vec3, end: Vec3) -> impl Curve<Vec3> + Threaded;
}

impl IntoCurve<EasingCurve<Vec3>> for EaseFunction {
    fn into_curve(&self, start: Vec3, end: Vec3) -> impl Curve<Vec3> + Threaded {
        EasingCurve::new(start, end, *self)
    }
}

fn fragment_bridge(
    mut fragment_events: EventReader<FragmentEvent<fragments::SectionFrag>>,
    mut textbox: EventWriter<TextboxEvent>,

    mut text_end: EventReader<TextboxClosedEvent>,
    mut fragment_end: EventWriter<FragmentEndEvent>,

    mut ids: Local<VecDeque<FragmentEndEvent>>,
) {
    for event in fragment_events.read() {
        ids.push_back(event.end());
        textbox.write(TextboxEvent::section(TextBlurb::main_character(
            event.data.section.clone(),
        )));
    }

    for _event in text_end.read() {
        if let Some(end) = ids.pop_front() {
            fragment_end.write(end);
        }
    }
}

fn intro() -> impl IntoBox {
    ("Hello, world!", "How are you?").always().once()
}

fn fragment_test(mut commands: Commands) {
    intro().spawn_box(&mut commands);
}
