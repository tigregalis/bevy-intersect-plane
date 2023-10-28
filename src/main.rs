use bevy::{prelude::*, window::PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, my_cursor_system)
        .run();
}

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Component, Clone, Copy)]
struct MyPlane {
    size: f32,
}

impl MyPlane {
    fn new(size: f32) -> Self {
        Self { size }
    }
    fn to_plane(self) -> shape::Plane {
        shape::Plane::from_size(self.size)
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane 1
    let plane = MyPlane::new(3.0);
    commands.spawn((
        // we need the size of the plane in future calculations, so we store it as a component
        plane,
        // generate the mesh from the plane
        PbrBundle {
            mesh: meshes.add(plane.to_plane().into()),
            material: materials.add(Color::rgb(0.6, 0.55, 0.3).into()),
            ..default()
        },
    ));
    // plane 2
    let plane = MyPlane::new(2.0);
    commands.spawn((
        // we need the size of the plane in future calculations, so we store it as a component
        plane,
        PbrBundle {
            // generate the mesh from the plane
            mesh: meshes.add(plane.to_plane().into()),
            material: materials.add(Color::rgb(0.8, 0.75, 0.6).into()),
            transform: {
                let mut transform = Transform::from_xyz(1.5, 0.5, -0.8);
                transform.rotate_axis(Vec3::Z, std::f32::consts::TAU * 0.1);
                transform.rotate_axis(Vec3::Y, std::f32::consts::TAU * 0.1);
                transform.rotate_axis(Vec3::X, std::f32::consts::TAU * 0.1);
                transform
            },
            ..default()
        },
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));
}

fn my_cursor_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // query to get the window (so we can read the current cursor position)
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    // plane
    q_plane: Query<(Entity, &Transform, &MyPlane)>,
    // mouse
    mouse: Res<Input<MouseButton>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    if let Some(ray) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
    {
        for (entity, transform, plane) in q_plane.iter() {
            let plane_origin = transform.translation;
            // we know the unrotated plane mesh has a normal of Vec3::Y from `impl From<Plane> for Mesh`
            // if that's not the case for your shape then you need to rotate it
            let plane_normal = (transform.rotation * Vec3::Y).normalize();
            let intersection = ray
                .intersect_plane(plane_origin, plane_normal)
                .map(|distance| ray.get_point(distance));

            // now that we have the intersection point, we need to check that it lies within the plane mesh
            if let Some(world_intersection) = intersection {
                // translate so that the plane origin is at the origin
                let local_intersection = world_intersection - plane_origin;

                // project
                // we know the unrotated plane mesh has a normal of Vec3::Y from `impl From<Plane> for Mesh`
                // which means unrotated plane mesh has horizontal and vertical axes of Vec3::X and Vec3::Z
                // if that's not the case for your shape then you need to rotate it
                let x_axis = (transform.rotation * Vec3::X).normalize();
                let z_axis = (transform.rotation * Vec3::Z).normalize();
                let x_projection = local_intersection.dot(x_axis);
                let z_projection = local_intersection.dot(z_axis);
                let local_intersection = Vec2::new(x_projection, z_projection) / plane.size;

                // we know the finite plane goes from (-0.5, -0.5)*plane_size to (0.5, 0.5)*plane_size from `impl From<Plane> for Mesh`
                // if that's not the case for your shape then adjust this "hit detection" code
                // the other assumption is that the plane is always square, so the plane side lengths are the same
                // which might not be true for you
                if local_intersection.x.abs() <= 0.5 && local_intersection.y.abs() <= 0.5 {
                    // (-0.5, 0.5)..(0.5, 0.5) => (0, 1.0)..(0, 1.0)
                    let local_intersection = local_intersection + Vec2::splat(0.5);
                    println!(
                        "{entity:?}: hit at {x:.2},{y:.2} within surface",
                        x = local_intersection.x,
                        y = local_intersection.y
                    );
                    // cube
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                        material: materials.add(Color::rgb(0.1, 0.8, 0.2).into()),
                        transform: Transform::from_translation(world_intersection),
                        ..default()
                    });
                } else {
                    // cube
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                        material: materials.add(Color::rgb(0.8, 0.1, 0.2).into()),
                        transform: Transform::from_translation(world_intersection),
                        ..default()
                    });
                }
            }
        }
    }
}
