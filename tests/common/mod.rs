use bevy::prelude::*;

#[derive(Resource, Component, Debug, Deref, DerefMut)]
pub struct TestUsize(pub usize);
