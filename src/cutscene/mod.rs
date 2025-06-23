use bevy::prelude::*;
use bevy_sequence::{
    Threaded,
    combinators::delay::run_after,
    prelude::{FragmentEndEvent, FragmentEvent, FragmentExt},
};
use fragments::IntoBox;
use std::{any::TypeId, collections::VecDeque, time::Duration};

use crate::textbox::{TextBlurb, TextboxClosedEvent, TextboxEvent};

mod fragments;
mod movement;

pub struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        let mut cache = movement::MovementSystemCache::default();
        cache.0.insert(TypeId::of::<EasingCurve<Vec3>>());

        app.init_resource::<FragmentEndEvents>()
            .insert_resource(cache)
            .add_event::<FragmentEvent<fragments::SectionFrag>>()
            .add_systems(
                PostUpdate,
                movement::apply_movements::<EasingCurve<Vec3>>
                    .before(TransformSystem::TransformPropagate),
            )
            .add_systems(
                PreUpdate,
                fragment_bridge_start.after(bevy_sequence::SequenceSets::Emit),
            )
            .add_systems(
                PostUpdate,
                fragment_bridge_end.before(bevy_sequence::SequenceSets::Respond),
            )
            .add_systems(Startup, |mut commands: Commands| {
                run_after(Duration::from_millis(500), fragment_test, &mut commands);
            });
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

#[derive(Resource, Default)]
struct FragmentEndEvents(VecDeque<FragmentEndEvent>);

fn fragment_bridge_start(
    mut fragment_events: EventReader<FragmentEvent<fragments::SectionFrag>>,
    mut textbox: EventWriter<TextboxEvent>,

    mut ids: ResMut<FragmentEndEvents>,
) {
    for event in fragment_events.read() {
        ids.0.push_back(event.end());
        textbox.write(TextboxEvent::section(TextBlurb::main_character(
            event.data.section.clone(),
        )));
    }
}

fn fragment_bridge_end(
    mut text_end: EventReader<TextboxClosedEvent>,
    mut fragment_end: EventWriter<FragmentEndEvent>,

    mut ids: ResMut<FragmentEndEvents>,
) {
    for _event in text_end.read() {
        if let Some(end) = ids.0.pop_front() {
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
