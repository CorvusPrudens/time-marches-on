use bevy::prelude::*;
use bevy_sequence::{
    Threaded,
    prelude::{FragmentEndEvent, FragmentEvent, FragmentExt},
};
use fragments::IntoBox;
use std::{any::TypeId, collections::VecDeque};

use crate::textbox::{TextBlurb, TextboxCloseEvent, TextboxCloseInteraction, TextboxEvent};

pub mod chara;
pub mod fragments;
pub mod movement;

pub struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        let mut cache = movement::MovementSystemCache::default();
        cache.0.insert(TypeId::of::<EasingCurve<Vec3>>());

        app.init_resource::<FragmentEndEvents>()
            .insert_resource(cache)
            .add_event::<FragmentEvent<fragments::CutsceneFragment>>()
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
                (fragment_bridge_end, tick_delay).before(bevy_sequence::SequenceSets::Respond),
            );
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

#[derive(Component)]
struct Delay {
    timer: Timer,
    id: FragmentEndEvent,
}

fn fragment_bridge_start(
    mut fragment_events: EventReader<FragmentEvent<fragments::CutsceneFragment>>,
    mut textbox: EventWriter<TextboxEvent>,

    mut close: EventWriter<TextboxCloseEvent>,

    mut ids: ResMut<FragmentEndEvents>,
    mut commands: Commands,
) {
    for event in fragment_events.read() {
        match &event.data {
            fragments::CutsceneFragment::Dialog(d) => {
                ids.0.push_back(event.end());
                textbox.write(TextboxEvent::section_retained(TextBlurb::main_character(
                    d.clone(),
                )));
            }
            fragments::CutsceneFragment::Pause(p) => {
                commands.spawn(Delay {
                    timer: Timer::new(*p, TimerMode::Once),
                    id: event.end(),
                });
                close.write_default();
            }
        }
    }
}

fn fragment_bridge_end(
    mut text_end: EventReader<TextboxCloseInteraction>,
    mut fragment_end: EventWriter<FragmentEndEvent>,

    mut ids: ResMut<FragmentEndEvents>,
) {
    for _event in text_end.read() {
        if let Some(end) = ids.0.pop_front() {
            fragment_end.write(end);
        }
    }
}

fn tick_delay(
    mut delays: Query<(Entity, &mut Delay)>,
    mut commands: Commands,
    time: Res<Time>,

    mut fragment_end: EventWriter<FragmentEndEvent>,
) {
    let delta = time.delta();

    for (entity, mut delay) in delays.iter_mut() {
        if delay.timer.tick(delta).just_finished() {
            commands.entity(entity).despawn();
            fragment_end.write(delay.id);
        }
    }
}

fn intro() -> impl IntoBox {
    ("Hello, world!", "How are you?").always().once()
}

fn fragment_test(mut commands: Commands) {
    intro().spawn_box(&mut commands);
}
