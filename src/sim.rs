use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::utils::Duration;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use ndarray::Array2;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

#[derive(Copy, Clone)]
enum Dir {
    NULL,
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Distribution<Dir> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Dir {
        match rng.gen_range(0..9) {
            0 => Dir::NULL,
            1 => Dir::N,
            2 => Dir::S,
            3 => Dir::E,
            4 => Dir::W,
            5 => Dir::NE,
            6 => Dir::NW,
            7 => Dir::SE,
            _ => Dir::SW,
        }
    }
}

impl Dir {
    pub fn value(self) -> Coord<i8> {
        match self {
            Self::NULL => Coord { x: 0, y: 0 },
            Self::N => Coord { x: 0, y: 1 },
            Self::S => Coord { x: 0, y: -1 },
            Self::E => Coord { x: 1, y: 0 },
            Self::W => Coord { x: -1, y: 0 },
            Self::NE => Coord { x: 1, y: 1 },
            Self::NW => Coord { x: -1, y: 1 },
            Self::SE => Coord { x: 1, y: -1 },
            Self::SW => Coord { x: -1, y: -1 },
        }
    }
}

#[derive(Component, Clone)]
pub struct Particle;

#[derive(Component, Clone, PartialEq)]
struct Coord<T> {
    pub x: T,
    pub y: T,
}

#[derive(Component, Clone)]
struct Grid {
    pub data: Array2<bool>,
}

#[derive(Component, Clone)]
struct Parameters {
    pub grid_size: usize,
    pub n_particles: usize,
}

struct SimTimer(Timer);

#[derive(Component)]
struct SimState {
    pub paused: bool,
    pub reset: bool,
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTimer(Timer::from_seconds(0.05, true)))
            .insert_resource(SimState {
                paused: false,
                reset: false,
            })
            .add_startup_system(setup)
            .add_system(handle_input)
            .add_system(sim_step)
            .add_system(reset_sim)
            .add_system(center_camera);
    }
}

fn setup(
    mut windows: ResMut<Windows>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(Camera2dBundle {
        camera_2d: Camera2d {
            clear_color: ClearColorConfig::Custom(Color::MIDNIGHT_BLUE),
        },
        ..Default::default()
    });

    for window in windows.iter_mut() {
        window.set_resizable(false);

        let params = Parameters {
            grid_size: 300,
            n_particles: 100,
        };

        commands.insert_resource(params.clone());

        let particle_width = window.width() / params.grid_size as f32;
        let particle_height = window.height() / params.grid_size as f32;
        let particle_size = 1.5 * particle_height;

        let (particles, grid) = get_particles(params.n_particles, params.grid_size);
        for p in particles {
            commands
                .spawn()
                .insert(Particle)
                .insert(Coord::<isize> {
                    x: p.x as isize,
                    y: p.y as isize,
                })
                .insert_bundle(MaterialMesh2dBundle {
                    mesh: meshes
                        .add(
                            shape::Quad::new(Vec2 {
                                x: particle_size,
                                y: particle_size,
                            })
                            .into(),
                        )
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE)),
                    transform: Transform::from_translation(Vec3::new(
                        (p.x as f32) * particle_width,
                        (p.y as f32) * particle_height,
                        0.,
                    )),
                    ..default()
                });
        }

        commands.insert_resource(grid);
    }
}

fn sim_step(
    time: Res<Time>,
    windows: Res<Windows>,
    sim_state: Res<SimState>,
    params: Res<Parameters>,
    mut timer: ResMut<SimTimer>,
    mut grid: ResMut<Grid>,
    mut particles: Query<(&mut Particle, &mut Coord<isize>, &mut Transform)>,
) {
    if !sim_state.paused && !sim_state.reset && timer.0.tick(time.delta()).just_finished() {
        for window in windows.iter() {
            let mut rng = rand::thread_rng();

            let particle_width = window.width() / params.grid_size as f32;
            let particle_height = window.height() / params.grid_size as f32;

            for (_, mut coord, mut transform) in particles.iter_mut() {
                let next_dir: Dir = rng.gen();
                let dir_coord: Coord<i8> = next_dir.value();
                let next_coord = Coord {
                    x: coord.x + dir_coord.x as isize,
                    y: coord.y + dir_coord.y as isize,
                };

                //world bounds check
                if next_coord.x < 0
                    || next_coord.x >= params.grid_size as isize
                    || next_coord.y < 0
                    || next_coord.y >= params.grid_size as isize
                {
                    continue;
                }

                //collision check
                if *coord != next_coord && grid.data[[next_coord.x as usize, next_coord.y as usize]]
                {
                    continue;
                }

                transform.translation.x = next_coord.x as f32 * particle_width;
                transform.translation.y = next_coord.y as f32 * particle_height;

                grid.data[[coord.x as usize, coord.y as usize]] = false;
                grid.data[[next_coord.x as usize, next_coord.y as usize]] = true;

                *coord = next_coord;
            }
        }
    }
}

fn center_camera(mut windows: ResMut<Windows>, mut camera: Query<&mut Transform, With<Camera>>) {
    for window in windows.iter_mut() {
        for mut transform in camera.iter_mut() {
            transform.translation.x = window.width() / 2.;
            transform.translation.y = window.height() / 2.;
        }
    }
}

fn reset_sim(
    windows: Res<Windows>,
    params: Res<Parameters>,
    mut sim_state: ResMut<SimState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    particles: Query<(Entity, &Particle)>,
) {
    if sim_state.reset {
        for window in windows.iter() {
            for e in particles.iter() {
                commands.entity(e.0).despawn_recursive();
            }
            commands.remove_resource::<Grid>();

            let particle_width = window.width() / params.grid_size as f32;
            let particle_height = window.height() / params.grid_size as f32;

            let particle_size = 1.5 * particle_height;

            let (particles, grid) = get_particles(params.n_particles, params.grid_size);
            for p in particles {
                commands
                    .spawn()
                    .insert(Particle)
                    .insert(Coord::<isize> {
                        x: p.x as isize,
                        y: p.y as isize,
                    })
                    .insert_bundle(MaterialMesh2dBundle {
                        mesh: meshes
                            .add(
                                shape::Quad::new(Vec2 {
                                    x: particle_size,
                                    y: particle_size,
                                })
                                .into(),
                            )
                            .into(),
                        material: materials.add(ColorMaterial::from(Color::WHITE)),
                        transform: Transform::from_translation(Vec3::new(
                            (p.x as f32) * particle_width,
                            (p.y as f32) * particle_height,
                            0.,
                        )),
                        ..default()
                    });
            }

            commands.insert_resource(grid);

            sim_state.reset = false;
        }
    }
}

fn handle_input(
    keys: Res<Input<KeyCode>>,
    mut timer: ResMut<SimTimer>,
    mut sim_state: ResMut<SimState>,
) {
    if keys.just_pressed(KeyCode::Space) {
        sim_state.paused ^= true;
    }
    if keys.just_pressed(KeyCode::R) {
        sim_state.reset = true;
    }

    //sim speed control
    if keys.just_pressed(KeyCode::Key1) {
        timer.0.set_duration(Duration::from_secs_f32(0.1));
    }
    if keys.just_pressed(KeyCode::Key2) {
        timer.0.set_duration(Duration::from_secs_f32(0.05));
    }
    if keys.just_pressed(KeyCode::Key3) {
        timer.0.set_duration(Duration::from_secs_f32(0.025));
    }
}

fn get_particles(n_particles: usize, grid_size: usize) -> (Vec<Coord<isize>>, Grid) {
    let mut res = Vec::<Coord<isize>>::new();
    res.reserve_exact(n_particles);

    let v = vec![false; grid_size * grid_size];
    let mut grid = Grid {
        data: Array2::<bool>::from_shape_vec((grid_size, grid_size), v).unwrap(),
    };

    let mut rng = rand::thread_rng();

    let mut n = 0;
    while n < n_particles {
        let x = rng.gen_range(0..grid_size);
        let y = rng.gen_range(0..grid_size);

        if grid.data[[x, y]] {
            continue;
        }

        grid.data[[x, y]] = true;

        let coord = Coord::<isize> {
            x: x as isize,
            y: y as isize,
        };
        res.push(coord);

        n += 1;
    }
    (res, grid)
}
