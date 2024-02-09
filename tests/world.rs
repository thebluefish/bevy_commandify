use bevy::prelude::*;
use bevy_commandify::*;
use std::marker::PhantomData;

#[command]
fn foo(_world: &mut World) {}

#[command]
fn generic_foo<T: Send + Sync + 'static>(_world: &mut World, _phantom: PhantomData<T>) {}

#[entity_command]
fn mut_entity(_entity: Entity, _world: &mut World) {}

#[test]
fn world_impl_works() {
    let mut world = World::new();

    world.foo();
}

#[test]
fn generic_world_impl_works() {
    let mut world = World::new();

    world.generic_foo::<()>(PhantomData);
}

#[test]
fn world_mut_entity_impl_works() {
    let mut world = World::new();
    let mut entity = world.spawn_empty();

    entity.mut_entity();
}
