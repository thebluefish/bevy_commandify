use bevy::prelude::*;

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct TestUsize(pub usize);
