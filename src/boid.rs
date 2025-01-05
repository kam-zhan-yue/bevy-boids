use bevy::prelude::*;
use rand::Rng;

const BOID_COLOUR: Color = Color::srgb(0.1, 0.2, 0.12);
const BOID_LENGTH: f32 = 15.;
const BOID_SPEED: f32 = 20.;

#[derive(Bundle)]
pub struct BoidBundle {
    pub boid: Boid,
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct Boid {
    separation: Vec2,
    alignment: Vec2,
    cohesion: Vec2,
}

impl Boid {
    pub fn new() -> Self {
        Self {
            separation: Vec2::ZERO,
            alignment: Vec2::ZERO,
            cohesion: Vec2::ZERO,
        }
    }
}

#[derive(Component, Debug)]
pub struct Velocity(Vec2);

#[derive(Component, Debug)]
pub struct Acceleration(Vec2);

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_boids);
        app.add_systems(Update, (update_velocity, update_position, simulate_boids));
    }
}

fn spawn_boid_group(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    group_num: i32,
    position: Vec2,
    radius: f32,
    starting_velocity: Vec2,
) {
    let mut rng = rand::thread_rng();
    let transform =
        Transform::from_translation(position.extend(0.)).with_scale(Vec3::splat(BOID_LENGTH));
    for _ in 0..group_num {
        let random_unit_vector =
            Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), 0.).normalize_or_zero();
        let random_radius = rng.gen_range(-radius..radius);
        let mut random_transform = transform;
        random_transform.translation += random_unit_vector * random_radius;

        commands.spawn((
            Mesh2d(meshes.add(Triangle2d::default())),
            MeshMaterial2d(materials.add(BOID_COLOUR)),
            random_transform,
            BoidBundle {
                boid: Boid::new(),
                velocity: Velocity(starting_velocity),
                acceleration: Acceleration(Vec2::ZERO),
            },
        ));
    }
}

fn update_velocity(mut query: Query<(&Acceleration, &mut Velocity)>, time: Res<Time>) {
    for (acceleration, mut velocity) in query.iter_mut() {
        velocity.0 += acceleration.0 * time.delta_secs();
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.);
    }
}

fn spawn_boids(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2d);
    spawn_boid_group(
        commands,
        meshes,
        materials,
        10,
        Vec2::ZERO,
        100.0,
        Vec2::Y * BOID_SPEED,
    );
}

fn can_see(transform1: &GlobalTransform, transform2: &GlobalTransform) -> bool {
    true
}

fn simulate_boids(mut query: Query<(&mut GlobalTransform, &mut Boid, &mut Velocity)>) {
    let mut iter = query.iter_combinations_mut();
    while let Some([(transform1, mut boid1, _), (transform2, boid2, velocity2)]) = iter.fetch_next()
    {
        if *boid1 == *boid2 || !can_see(&transform1, &transform2) {
            continue;
        }
        let separation = get_separation_force(&transform1, &transform2);
        let alignment = get_alignment_force(&velocity2);
        let cohesion = get_cohesion_force(&transform2);
        boid1.separation = separation;
        boid1.alignment = alignment;
        boid1.cohesion = cohesion;
    }
}

fn get_separation_force(transform1: &GlobalTransform, transform2: &GlobalTransform) -> Vec2 {
    let difference: Vec3 = transform2.translation() - transform1.translation();
    let force: Vec3 = difference.normalize() / difference.length_squared();
    return Vec2::new(force.x, force.y);
}

fn get_cohesion_force(transform: &GlobalTransform) -> Vec2 {
    Vec2::new(transform.translation().x, transform.translation().y)
}

fn get_alignment_force(velocity: &Velocity) -> Vec2 {
    velocity.0
}
