use bevy::prelude::*;
use rand::Rng;

const BOID_COLOUR: Color = Color::srgb(0.1, 0.2, 0.12);
const BOID_LENGTH: f32 = 15.;
const BOID_SPEED: f32 = 20.;
const SCREEN_X: f32 = 1000.;
const SCREEN_Y: f32 = 1000.;

#[derive(Resource, Debug, Default)]
struct NextBoidId(u32);

#[derive(Resource, Debug)]
pub struct BoidSettings {
    // Speed
    pub min_speed: f32,
    pub max_speed: f32,
    pub max_steer_force: f32,
    // Vision
    pub vision_radius: f32,
    pub vision_angle: f32,
    // Weights
    pub separation_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,

    // Flags
    pub separation: bool,
    pub alignment: bool,
    pub cohesion: bool,
}

impl Default for BoidSettings {
    fn default() -> Self {
        Self {
            min_speed: 70.,
            max_speed: 200.,
            max_steer_force: 10.,
            vision_radius: 1.,
            vision_angle: 360.,
            separation_weight: 1.,
            alignment_weight: 1.,
            cohesion_weight: 1.,
            separation: true,
            alignment: false,
            cohesion: false,
        }
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
#[require(Velocity, Acceleration)]
pub struct BoidData {
    pub separation: Vec2,
    pub alignment: Vec2,
    pub cohesion: Vec2,
}

impl Default for BoidData {
    fn default() -> Self {
        Self {
            separation: Vec2::ZERO,
            alignment: Vec2::ZERO,
            cohesion: Vec2::ZERO,
        }
    }
}

#[derive(Component, PartialEq, Debug)]
#[require(BoidData, Velocity, Acceleration)]
pub struct Boid {
    id: u32,
}

#[derive(Component, Default, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default, Debug)]
pub struct Acceleration(Vec2);

pub struct BoidPlugin;

impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoidSettings>();
        app.init_resource::<NextBoidId>();
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (update_velocity, update_position, simulate_boids, bound),
        );
    }
}

fn spawn_boid_group(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    next_boid_id: &mut ResMut<NextBoidId>,
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
        let boid_id = next_boid_id.0;
        next_boid_id.0 += 1;
        commands.spawn((
            Mesh2d(meshes.add(Triangle2d::default())),
            MeshMaterial2d(materials.add(BOID_COLOUR)),
            random_transform,
            Boid { id: boid_id },
            Velocity(starting_velocity),
        ));
    }
}

fn update_velocity(
    mut query: Query<(&Acceleration, &mut Velocity)>,
    time: Res<Time>,
    settings: Res<BoidSettings>,
) {
    for (acceleration, mut velocity) in query.iter_mut() {
        velocity.0 += acceleration.0 * time.delta_secs();
        // Clamp the velocity and final speed
        let final_speed = velocity
            .0
            .length()
            .clamp(settings.min_speed, settings.max_speed);
        velocity.0 = velocity.0.normalize_or_zero() * final_speed;
    }
}

fn update_position(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += (velocity.0 * time.delta_secs()).extend(0.);
    }
}

fn bound(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        let mut translation = transform.translation;
        if translation.x > SCREEN_X / 2. {
            translation.x = -SCREEN_X / 2.;
        } else if translation.x < -SCREEN_X / 2. {
            translation.x = SCREEN_X / 2.;
        }

        if translation.y > SCREEN_Y / 2. {
            translation.y = -SCREEN_Y / 2.;
        } else if translation.y < -SCREEN_Y / 2. {
            translation.y = SCREEN_Y / 2.;
        }
        transform.translation = translation;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut next_boid_id: ResMut<NextBoidId>,
) {
    commands.spawn(Camera2d);
    spawn_boid_group(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut next_boid_id,
        2,
        Vec2::ZERO,
        100.0,
        Vec2::Y * BOID_SPEED,
    );
}

fn can_see(transform1: &GlobalTransform, transform2: &GlobalTransform) -> bool {
    true
}

fn simulate_boids(
    mut query: Query<(
        &Boid,
        &mut GlobalTransform,
        &mut BoidData,
        &mut Velocity,
        &mut Acceleration,
    )>,
    settings: Res<BoidSettings>,
) {
    // Calculate the separation, alignment, and cohesion for each boid
    // Combinations doesn't give us order, so we need to apply forces for both boids
    let mut iter = query.iter_combinations_mut();
    while let Some(
        [(boid1, transform1, mut data1, velocity1, _), (boid2, transform2, mut data2, velocity2, _)],
    ) = iter.fetch_next()
    {
        if *boid1 == *boid2 || !can_see(&transform1, &transform2) {
            continue;
        }
        // Calculate it for first boid
        let separation = get_separation_force(&transform1, &transform2);
        let alignment = get_alignment_force(&velocity2);
        let cohesion = get_cohesion_force(&transform2);
        data1.separation = separation;
        data1.alignment = alignment;
        data1.cohesion = cohesion;

        // Calculate it for second boid
        let separation = get_separation_force(&transform2, &transform1);
        let alignment = get_alignment_force(&velocity1);
        let cohesion = get_cohesion_force(&transform1);
        data2.separation = separation;
        data2.alignment = alignment;
        data2.cohesion = cohesion;
    }

    // Simulate the acceleration / velocity for each boid
    for (_, _, boid, velocity, mut acceleration) in query.iter_mut() {
        let separation_force = steer_towards(&boid.separation, &settings, &velocity);
        let alignment_force = steer_towards(&boid.alignment, &settings, &velocity);
        let cohesion_force = steer_towards(&boid.cohesion, &settings, &velocity);

        if settings.separation {
            acceleration.0 += separation_force * settings.separation_weight;
        }
        if settings.alignment {
            acceleration.0 += alignment_force * settings.alignment_weight;
        }
        if settings.cohesion {
            acceleration.0 += cohesion_force * settings.cohesion_weight
        }
    }
}

fn steer_towards(target: &Vec2, settings: &Res<BoidSettings>, velocity: &Velocity) -> Vec2 {
    if target.length() == 0. {
        return Vec2::ZERO;
    }
    let v = target.normalize() * settings.max_speed - velocity.0;
    v.clamp_length(-settings.max_steer_force, settings.max_steer_force)
}

fn get_separation_force(transform1: &GlobalTransform, transform2: &GlobalTransform) -> Vec2 {
    let difference: Vec3 = transform2.translation() - transform1.translation();
    let force: Vec3 = difference.normalize() / difference.length_squared();
    // println!("Separation Force {:?}", force);
    return Vec2::new(force.x, force.y);
}

fn get_cohesion_force(transform: &GlobalTransform) -> Vec2 {
    Vec2::new(transform.translation().x, transform.translation().y)
}

fn get_alignment_force(velocity: &Velocity) -> Vec2 {
    velocity.0
}
