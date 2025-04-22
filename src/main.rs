use bevy::prelude::*;
use bevy_hanabi::prelude::*;

#[derive(Component)]
pub struct SpawnTimer(Timer);

fn main() -> AppExit {
    App::new()
        .add_plugins((DefaultPlugins, HanabiPlugin))
        .add_systems(Startup, (setup_effect, setup))
        .add_systems(Update, (keyboard_input_system, spawn, cleanup))
        .run()
}

#[derive(Resource)]
pub struct ExplosionParticleEffect {
    pub effect: Handle<EffectAsset>,
}

fn setup(mut cmd: Commands) {
    cmd.spawn((
        Transform::from_translation(Vec3::new(0., 20., 50.)),
        Camera3d::default(),
        SpawnTimer(Timer::from_seconds(1., TimerMode::Repeating)),
        Camera {
            hdr: true,
            clear_color: Color::BLACK.into(),
            ..default()
        },
    ));
}

fn spawn(
    mut timer: Single<&mut SpawnTimer>,
    mut cmd: Commands,
    my_effect: Res<ExplosionParticleEffect>,

    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    };

    cmd.spawn((
        Lifetime(Timer::from_seconds(1., TimerMode::Once)),
        ParticleEffect::new(my_effect.effect.clone()),
        Transform::from_translation(Vec3::splat(0.).with_y(20.)),
    ));
}

fn keyboard_input_system(
    mut cmd: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !keyboard_input.pressed(KeyCode::Space) {
        return;
    };

    cmd.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_translation(Vec3::splat(0.).with_y(20.)),
    ));
}

#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct Lifetime(Timer);

pub fn cleanup(
    mut commands: Commands,
    mut q_values: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut q_values {
        lifetime.0.tick(time.delta());

        if !lifetime.0.just_finished() || commands.get_entity(entity).is_none() {
            continue;
        }

        commands.entity(entity).despawn_recursive();
    }
}

pub(crate) fn setup_effect(mut cmd: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // Define a color gradient from red to transparent black
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1., 0., 0., 1.));
    gradient.add_key(1.0, Vec4::splat(0.));

    // Create a new expression module
    let mut module = Module::default();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: module.lit(Vec3::ZERO),
        radius: module.lit(2.),
        dimension: ShapeDimension::Surface,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: module.lit(Vec3::ZERO),
        speed: module.lit(6.),
    };

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = module.lit(1.); // literal value "10.0"
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0., -3., 0.));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    let effect = EffectAsset::new(
        // Maximum number of particles alive at a time
        32768,
        // Spawn at a rate of 5 particles per second
        SpawnerSettings::once(5.0.into()),
        // Move the expression module into the asset
        module,
    )
    .with_name("MyEffect")
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .update(update_accel)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    .render(ColorOverLifetimeModifier {
        gradient,

        blend: ColorBlendMode::Modulate,
        mask: ColorBlendMask::RGBA,
    });

    // Insert into the asset system
    let effect_handle = effects.add(effect);

    cmd.insert_resource(ExplosionParticleEffect {
        effect: effect_handle,
    })
}
