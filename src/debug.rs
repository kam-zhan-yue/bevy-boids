use bevy::prelude::*;

use crate::boid::{Boid, BoidSettings, Velocity};

const DEBUG_VISION_COLOUR: Color = Color::srgb(0.1, 0.1, 0.1);
const DEBUG_VELOCITY_COLOUR: Color = Color::srgb(0.5, 0., 0.);
const DEBUG_VELOCITY_WIDTH: f32 = 0.3;
const DEBUG_VELOCITY_LENGTH: f32 = 1.5;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, setup);
        app.add_systems(Update, update_debug_visuals);
    }
}

#[derive(Component, Debug, Default)]
pub struct DebugRadius;

#[derive(Component, Debug, Default)]
pub struct DebugVelocity;

#[derive(Component, Debug)]
#[require(DebugRadius, DebugVelocity)]
pub struct DebugVisuals;

fn setup(
    mut query: Query<(Entity, &mut GlobalTransform, &mut Velocity), With<Boid>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    settings: Res<BoidSettings>,
) {
    for (entity, _, _velocity) in query.iter_mut() {
        spawn_debug_visuals(
            &mut commands,
            &entity,
            &mut meshes,
            &mut materials,
            &settings,
        );
    }
}

fn spawn_debug_visuals(
    commands: &mut Commands,
    entity: &Entity,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    settings: &Res<BoidSettings>,
) {
    let mut debug_radius = commands.spawn((
        Mesh2d(meshes.add(Circle::new(settings.vision_radius))),
        MeshMaterial2d(materials.add(DEBUG_VISION_COLOUR)),
        Transform::from_xyz(0., 0., -1.),
        DebugRadius,
    ));

    debug_radius.set_parent(*entity);

    let mut debug_velocity = commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(DEBUG_VELOCITY_WIDTH, DEBUG_VELOCITY_LENGTH))),
        MeshMaterial2d(materials.add(DEBUG_VELOCITY_COLOUR)),
        Transform::from_xyz(0., 0., -0.5),
        DebugVelocity,
    ));

    debug_velocity.set_parent(*entity);
}

fn update_debug_visuals(
    mut query: Query<(&Parent, &mut Transform), With<DebugVelocity>>,
    parent_query: Query<&Velocity>,
) {
    for (parent, mut transform) in query.iter_mut() {
        if let Ok(velocity) = parent_query.get(parent.get()) {
            let angle = -(velocity.0.y.atan2(velocity.0.x));
            transform.rotation = Quat::from_rotation_z(angle);
        }
    }
}
