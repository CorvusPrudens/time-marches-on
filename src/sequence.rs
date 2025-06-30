use bevy::ecs::{
    error::Result,
    observer::ObserverState,
    prelude::{Bundle, Trigger},
    schedule::{Fallible, Infallible},
    world::World,
};
use bevy::prelude::*;
use bevy_sequence::prelude::{FragmentEndEvent, FragmentEvent, IntoFragment};
use core::marker::PhantomData;
use std::time::Duration;

use bevy::ecs::system::IntoSystem;

pub struct ObserverSequencePlugin;

impl Plugin for ObserverSequencePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FragmentEvent<ObserverSequence>>()
            .add_systems(
                Update,
                (observer_sequence_runner, observer_sequence_delay).chain(),
            )
            .add_observer(disable_observer)
            .add_observer(enable_observer);
    }
}

#[derive(Debug, Event, Clone)]
pub enum ObserverSequence {
    /// An observer-monitored condition.
    Condition,
    /// A simple delay.
    Delay(Duration),
}

#[derive(Component)]
pub struct DisabledObserver(Observer);

#[derive(Event)]
struct DisableObserver;

#[derive(Event, Component, Clone)]
struct EnableObserver(FragmentEndEvent);

fn disable_observer(trigger: Trigger<DisableObserver>, mut commands: Commands) {
    let target = trigger.target();

    commands.entity(target).log_components();

    commands.queue(move |world: &mut World| -> Result {
        let mut entity = world.entity_mut(target);

        let observer = entity
            .take::<Observer>()
            .ok_or("expected `Observer` on sequence item")?;
        entity.remove::<ObserverState>();
        let id = entity
            .take::<EnableObserver>()
            .ok_or("expected `EnableObserver` on sequence item")?;

        entity.insert(DisabledObserver(observer));
        world.send_event(id.0);

        Ok(())
    });
}

fn enable_observer(trigger: Trigger<EnableObserver>, mut commands: Commands) {
    let enable_observer = trigger.clone();
    let target = trigger.target();

    commands.queue(move |world: &mut World| -> Result {
        let mut entity = world.entity_mut(target);

        let observer = entity
            .take::<DisabledObserver>()
            .ok_or("expected `DisabledObserver` on sequence item")?;
        entity.insert((observer.0, enable_observer));

        Ok(())
    });
}

fn observer_sequence_runner(
    mut reader: EventReader<FragmentEvent<ObserverSequence>>,
    mut commands: Commands,
) {
    for event in reader.read() {
        match event.data {
            ObserverSequence::Condition => {
                commands
                    .entity(event.id.fragment.entity())
                    .trigger(EnableObserver(event.end()));
            }
            ObserverSequence::Delay(delay) => {
                commands.spawn(DelayTimer {
                    timer: Timer::new(delay, TimerMode::Once),
                    event: event.end(),
                });
            }
        }
    }
}

#[derive(Component)]
struct DelayTimer {
    timer: Timer,
    event: FragmentEndEvent,
}

fn observer_sequence_delay(
    mut delays: Query<(Entity, &mut DelayTimer)>,
    mut writer: EventWriter<FragmentEndEvent>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let delta = time.delta();

    for (entity, mut delay) in &mut delays {
        if delay.timer.tick(delta).just_finished() {
            commands.entity(entity).despawn();
            writer.write(delay.event);
        }
    }
}

pub struct Delay {
    delay: Duration,
}

pub fn delay(seconds: f32) -> Delay {
    Delay {
        delay: Duration::from_secs_f32(seconds),
    }
}

impl IntoFragment<ObserverSequence> for Delay {
    fn into_fragment(
        self,
        context: &bevy_sequence::prelude::Context<()>,
        commands: &mut Commands,
    ) -> bevy_sequence::prelude::FragmentId {
        <_ as IntoFragment<ObserverSequence>>::into_fragment(
            bevy_sequence::fragment::DataLeaf::new(ObserverSequence::Delay(self.delay)),
            context,
            commands,
        )
    }
}

pub struct Con<S, I, B, O, M> {
    system: S,
    marker: PhantomData<fn() -> (I, B, O, M)>,
}

/// Construct a _condition_ -- an observer that returns whether the
/// condition has been satisfied.
pub fn con<S, E, B, O, M>(system: S) -> Con<S, E, B, O, M> {
    Con {
        system,
        marker: PhantomData,
    }
}

/// A marker struct for observers that, when triggered, always advance the sequence.
pub struct Inherent;

impl<S, E, B, M> IntoFragment<ObserverSequence> for Con<S, E, B, (), (M, Inherent)>
where
    S: IntoSystem<Trigger<'static, E, B>, (), M> + Send + Clone + 'static,
    E: Event,
    B: Bundle,
{
    fn into_fragment(
        self,
        context: &bevy_sequence::prelude::Context<()>,
        commands: &mut Commands,
    ) -> bevy_sequence::prelude::FragmentId {
        let system = self.system;

        let leaf = <_ as IntoFragment<ObserverSequence>>::into_fragment(
            bevy_sequence::fragment::DataLeaf::new(ObserverSequence::Condition),
            context,
            commands,
        );

        let observer = Observer::new(system.pipe(move |_: In<()>, mut commands: Commands| {
            commands.entity(leaf.entity()).trigger(DisableObserver);
        }));

        commands
            .entity(leaf.entity())
            .insert(DisabledObserver(observer));

        leaf
    }
}

impl<S, E, B, M> IntoFragment<ObserverSequence> for Con<S, E, B, Result, (M, Inherent, Fallible)>
where
    S: IntoSystem<Trigger<'static, E, B>, Result, M> + Send + Clone + 'static,
    E: Event,
    B: Bundle,
{
    fn into_fragment(
        self,
        context: &bevy_sequence::prelude::Context<()>,
        commands: &mut Commands,
    ) -> bevy_sequence::prelude::FragmentId {
        let system = self.system;

        let leaf = <_ as IntoFragment<ObserverSequence>>::into_fragment(
            bevy_sequence::fragment::DataLeaf::new(ObserverSequence::Condition),
            context,
            commands,
        );

        let observer = Observer::new(system.pipe(
            move |result: In<Result>, mut commands: Commands| -> Result {
                result.0?;

                commands.entity(leaf.entity()).trigger(DisableObserver);

                Ok(())
            },
        ));

        commands
            .entity(leaf.entity())
            .insert(DisabledObserver(observer));

        leaf
    }
}

impl<S, E, B, M> IntoFragment<ObserverSequence> for Con<S, E, B, bool, (M, Infallible)>
where
    S: IntoSystem<Trigger<'static, E, B>, bool, M> + Send + Clone + 'static,
    E: Event,
    B: Bundle,
{
    fn into_fragment(
        self,
        context: &bevy_sequence::prelude::Context<()>,
        commands: &mut Commands,
    ) -> bevy_sequence::prelude::FragmentId {
        let system = self.system;

        let leaf = <_ as IntoFragment<ObserverSequence>>::into_fragment(
            bevy_sequence::fragment::DataLeaf::new(ObserverSequence::Condition),
            context,
            commands,
        );

        let observer = Observer::new(system.pipe(
            move |complete: In<bool>, mut commands: Commands| {
                if complete.0 {
                    commands.entity(leaf.entity()).trigger(DisableObserver);
                }
            },
        ));

        commands
            .entity(leaf.entity())
            .insert(DisabledObserver(observer));

        leaf
    }
}

impl<S, E, B, M> IntoFragment<ObserverSequence> for Con<S, E, B, Result<bool>, (M, Fallible)>
where
    S: IntoSystem<Trigger<'static, E, B>, Result<bool>, M> + Send + Clone + 'static,
    E: Event,
    B: Bundle,
{
    fn into_fragment(
        self,
        context: &bevy_sequence::prelude::Context<()>,
        commands: &mut Commands,
    ) -> bevy_sequence::prelude::FragmentId {
        let system = self.system;

        let leaf = <_ as IntoFragment<ObserverSequence>>::into_fragment(
            bevy_sequence::fragment::DataLeaf::new(ObserverSequence::Condition),
            context,
            commands,
        );

        let observer = Observer::new(system.pipe(
            move |complete: In<Result<bool>>, mut commands: Commands| -> Result {
                if complete.0? {
                    commands.entity(leaf.entity()).trigger(DisableObserver);
                }

                Ok(())
            },
        ));

        commands
            .entity(leaf.entity())
            .insert(DisabledObserver(observer));

        leaf
    }
}

// impl<S, E, B, M> IntoFragment<ObserverSequence> for Con<S, E, B, Never, (M, Never)>
// where
//     S: IntoSystem<Trigger<'static, E, B>, Never, M> + Send + 'static,
//     E: Event,
//     B: Bundle,
// {
//     fn into_fragment(
//         self,
//         context: &bevy_sequence::prelude::Context<()>,
//         commands: &mut Commands,
//     ) -> bevy_sequence::prelude::FragmentId {
//         let system = self.system;
//         // // register it
//         // let system_id: SystemId<Trigger<E, B>, bool> = commands.register_system(system);
//
//         let id = commands.spawn(Observer::new(system)).id();
//
//         bevy_sequence::prelude::FragmentId::new(id)
//     }
// }
