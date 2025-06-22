use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::{ecs::system::SystemId, prelude::*};

/// A callback that's generic over input and fully type-erased.
pub struct Callback<I: SystemInput + 'static = ()>(pub Arc<Mutex<dyn DynamicSystem<I>>>);

impl<I: SystemInput + 'static> Callback<I> {
    /// Create a new [`Callback`] with an arbitrary system.
    pub fn new<S, M>(system: S) -> Self
    where
        S: IntoSystem<I, (), M> + Send + Sync + 'static,
        M: 'static,
    {
        Self(Arc::new(Mutex::new(MaybeRegistered::System {
            system: Some(system),
            marker: PhantomData,
        })))
    }
}

/// A dyn-comptabible trait for calling/registering and unregistering arbitrary systems.
pub trait DynamicSystem<I: SystemInput + 'static>: Send + Sync {
    /// Call a system.
    fn call(&mut self, world: &mut World, input: I::Inner<'_>) -> Result;

    /// Perform any cleanup that may be required before dropping.
    fn unregister(&mut self, world: &mut World) -> Result;
}

// // We'll want to make sure we unregister the system if the button component is removed.
// impl CoreButton {
//     fn on_remove_hook(mut world: DeferredWorld, context: HookContext) {
//         let Some(mut callback) = world
//             .get_mut::<Self>(context.entity)
//             .and_then(|mut b| b.on_click.take())
//         else {
//             return;
//         };
//
//         world
//             .commands()
//             .queue(move |world: &mut World| callback.unregister(world));
//     }
// }

// impl<I: SystemInput + 'static> DynamicSystem<I> for Callback<I> {
//     fn call(&mut self, world: &mut World, input: <I as SystemInput>::Inner<'_>) -> Result {
//         self.0.call(world, input)
//     }
//
//     fn unregister(&mut self, world: &mut World) -> Result {
//         self.0.unregister(world)
//     }
// }

enum MaybeRegistered<S, I: SystemInput, M> {
    System {
        system: Option<S>,
        marker: PhantomData<fn() -> M>,
    },
    SystemId(SystemId<I>),
}

impl<S, I, M> DynamicSystem<I> for MaybeRegistered<S, I, M>
where
    S: IntoSystem<I, (), M> + Send + Sync + 'static,
    I: SystemInput + 'static,
{
    fn call(&mut self, world: &mut World, input: I::Inner<'_>) -> Result {
        let id = self.id(world);
        world.run_system_with(id, input)?;
        Ok(())
    }

    fn unregister(&mut self, world: &mut World) -> Result {
        if let Self::SystemId(id) = self {
            world.unregister_system(*id)?;
        }

        Ok(())
    }
}

impl<S, I, M> MaybeRegistered<S, I, M>
where
    S: IntoSystem<I, (), M> + 'static,
    I: SystemInput + 'static,
{
    fn id(&mut self, world: &mut World) -> SystemId<I> {
        match self {
            Self::System { system, .. } => {
                let id = world.register_system(system.take().unwrap());
                *self = Self::SystemId(id);
                id
            }
            Self::SystemId(id) => *id,
        }
    }
}
