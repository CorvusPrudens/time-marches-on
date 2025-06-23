use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_sequence::Threaded;
use bevy_sequence::prelude::*;
use std::any::TypeId;
use std::marker::PhantomData;
use std::time::Duration;

use super::{IntoCurve, fragments::IntoBox};

/// This is applied to entities whose movement should be handled
/// purely by cutscene directives.
#[derive(Debug, Clone, Copy, Component)]
pub struct CutsceneMovement;

#[derive(Debug, Clone, Component)]
pub struct MovementClip<C> {
    timer: Timer,
    curve: C,
}

/// The instantaneous velocity resulting from cutscene movements.
#[derive(Debug, Component)]
pub struct CutsceneVelocity(pub Vec3);

impl<C> MovementClip<C>
where
    C: Curve<Vec3>,
{
    pub fn position(&self) -> Option<Vec3> {
        self.curve.sample(self.timer.fraction())
    }

    /// Advance the clip by the given time.
    pub fn tick(&mut self, duration: Duration) {
        self.timer.tick(duration);
    }

    pub fn complete(&self) -> bool {
        self.timer.finished()
    }
}

pub trait CutsceneFragment<C>: Sized
where
    Self: IntoBox<C>,
{
    fn move_to<M: Component>(
        self,
        marker: M,
        position: Vec3,
        duration: Duration,
    ) -> impl IntoBox<C>;

    fn move_curve<M: Component, I>(
        self,
        marker: M,
        position: Vec3,
        duration: Duration,
        curve: impl IntoCurve<I> + Threaded,
    ) -> CutsceneFrag<impl IntoBox<C>, I>
    where
        I: Curve<Vec3> + Threaded;

    /// Lock entity movement during a cutscene.
    fn lock<M: Component>(self, marker: M) -> impl IntoBox<C>;
}

impl<T, C: Component> CutsceneFragment<C> for T
where
    T: IntoBox<C>,
{
    fn move_to<M: Component>(
        self,
        _marker: M,
        position: Vec3,
        duration: Duration,
    ) -> impl IntoBox<C> {
        let system = move |q: Query<(Entity, &Transform), With<M>>,
                           root: Single<&Transform, With<C>>,
                           mut commands: Commands| {
            let root = root.into_inner();
            for (entity, transform) in q.iter() {
                commands.entity(entity).insert((
                    CutsceneMovement,
                    CutsceneVelocity(Vec3::ZERO),
                    MovementClip {
                        curve: EaseFunction::Linear
                            .into_curve(transform.translation, root.translation - position),
                        timer: Timer::new(duration, TimerMode::Once),
                    },
                ));
            }
        };

        self.on_start(system)
    }

    fn move_curve<M: Component, I>(
        self,
        _marker: M,
        position: Vec3,
        duration: Duration,
        curve: impl IntoCurve<I> + Threaded,
    ) -> CutsceneFrag<impl IntoBox<C>, I>
    where
        I: Curve<Vec3> + Threaded,
    {
        let system = move |q: Query<(Entity, &Transform), With<M>>,
                           root: Single<&Transform, With<C>>,
                           mut commands: Commands| {
            let root = root.into_inner();
            for (entity, transform) in q.iter() {
                commands.entity(entity).insert((
                    CutsceneMovement,
                    CutsceneVelocity(Vec3::ZERO),
                    MovementClip {
                        curve: curve.into_curve(transform.translation, root.translation - position),
                        timer: Timer::new(duration, TimerMode::Once),
                    },
                ));
            }
        };

        CutsceneFrag {
            fragment: self.on_start(system),
            _marker: PhantomData,
        }
    }

    fn lock<M: Component>(self, _marker: M) -> impl IntoBox<C> {
        let start = |q: Query<Entity, With<M>>, mut commands: Commands| {
            for entity in q.iter() {
                commands
                    .entity(entity)
                    .insert((CutsceneMovement, CutsceneVelocity(Vec3::ZERO)));
            }
        };

        let end = |q: Query<Entity, (With<M>, With<CutsceneMovement>)>, mut commands: Commands| {
            for entity in q.iter() {
                commands
                    .entity(entity)
                    .remove::<(CutsceneMovement, CutsceneVelocity)>();
            }
        };

        self.on_start(start).on_end(end)
    }
}

#[derive(Default, Resource)]
pub struct MovementSystemCache(pub HashSet<TypeId>);

pub struct CutsceneFrag<F, M> {
    fragment: F,
    _marker: PhantomData<fn() -> M>,
}

impl<D, C, F, M> IntoFragment<D, C> for CutsceneFrag<F, M>
where
    D: Threaded,
    F: IntoFragment<D, C>,
    M: Curve<Vec3> + Threaded,
{
    fn into_fragment(self, context: &Context<C>, commands: &mut Commands) -> FragmentId {
        let id = self.fragment.into_fragment(context, commands);

        commands.queue(|world: &mut World| {
            world.schedule_scope(PostUpdate, |world: &mut World, schedule: &mut Schedule| {
                let mut cache = world.resource_mut::<MovementSystemCache>();
                if cache.0.insert(std::any::TypeId::of::<M>()) {
                    schedule.add_systems(
                        apply_movements::<M>.before(TransformSystem::TransformPropagate),
                    );
                }
            })
        });

        id
    }
}

pub fn apply_movements<C: Curve<Vec3> + Threaded>(
    mut q: Query<
        (
            Entity,
            &mut Transform,
            &mut MovementClip<C>,
            &mut CutsceneVelocity,
        ),
        With<CutsceneMovement>,
    >,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut clip, mut velocity) in q.iter_mut() {
        clip.tick(time.delta());

        if let Some(new_position) = clip.position() {
            let difference = new_position - transform.translation;
            transform.translation = new_position;
            velocity.0 = difference;
        }

        if clip.complete() {
            // Not sure if this is totally ideal
            velocity.0 = Vec3::ZERO;
            commands.entity(entity).remove::<MovementClip<C>>();
        }
    }
}
