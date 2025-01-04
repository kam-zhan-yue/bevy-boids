use bevy::prelude::*;

mod boid;

use boid::BoidPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.0, 0.15)))
        .insert_resource(AmbientLight {
            color: Color::default(),
            brightness: 750.0,
        })
        .add_plugins(BoidPlugin)
        .add_plugins(DefaultPlugins)
        .run();
}
