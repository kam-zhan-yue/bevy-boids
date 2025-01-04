use bevy::prelude::*;
use rand::Rng;

const BOID_COLOUR: Color = Color::srgb(1.0, 0.0, 0.0);
const BOID_LENGTH: f32 = 30.;

#[derive(Component, Debug)]
pub struct Boid;

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_boids);
        app.add_systems(Update, handle_boids);
    }
}

fn spawn_boid_group(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    group_num: i32,
    position: Vec2,
    radius: f32,
) {
    let mut rng = rand::thread_rng();
    let transform =
        Transform::from_translation(position.extend(0.)).with_scale(Vec3::splat(BOID_LENGTH));
    for _ in 0..group_num {
        let random_x = rng.gen_range(-radius..radius);
        let random_y = rng.gen_range(-radius..radius);
        let mut random_transform = transform;
        random_transform.translation.x += random_x;
        random_transform.translation.y += random_y;

        commands.spawn((
            Mesh2d(meshes.add(Triangle2d::default())),
            MeshMaterial2d(materials.add(BOID_COLOUR)),
            random_transform,
            Boid,
        ));
    }
}

fn spawn_boids(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    spawn_boid_group(commands, meshes, materials, 10, Vec2::ZERO, 100.0);
}

fn handle_boids(mut query: Query<&mut Transform, With<Boid>>) {}
