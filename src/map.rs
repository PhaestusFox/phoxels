use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, SuperSimplex};

pub fn plugin(app: &mut App) {
    app.init_resource::<MapDescriptor>();
    app.add_systems(Startup, spawn_world);
}

#[derive(Component)]
struct Chunk;
fn spawn_world(
    mut commands: Commands,
    map_descriptor: Res<MapDescriptor>,
    mut voxel_count: ResMut<super::diganostics::VoxelCount>,
) {
    // map_descriptor.min_max_y();
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE * 100.).looking_at(Vec3::NEG_Y * 100., Vec3::Y),
    ));
    for z in -100..100 {
        for x in -100..100 {
            let h = map_descriptor.get_y(x, z);
            for y in 0..=h {
                let block = if y == h {
                    map_descriptor.blocks[0].clone()
                } else if y + 3 > h {
                    map_descriptor.blocks[1].clone()
                } else {
                    map_descriptor.blocks[2].clone()
                };
                commands.spawn((
                    Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32)),
                    Mesh3d(map_descriptor.mesh()),
                    MeshMaterial3d(block),
                ));
                voxel_count.inc();
            }
        }
    }
}

#[derive(Debug, Resource)]
struct MapDescriptor {
    noise: Fbm<SuperSimplex>,
    blocks: Vec<Handle<StandardMaterial>>,
    mesh: Handle<Mesh>,
}

impl FromWorld for MapDescriptor {
    fn from_world(world: &mut World) -> Self {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::from_size(Vec3::ONE));
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let blocks = vec![
            materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0., 0.8, 0.),
                ..Default::default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0.88, 0.57, 0.39),
                ..Default::default()
            }),
            materials.add(StandardMaterial {
                base_color: Color::linear_rgb(0.4, 0.4, 0.4),
                ..Default::default()
            }),
        ];
        let mut noise = Fbm::new(0);
        noise = noise.set_frequency(0.005);
        noise = noise.set_persistence(0.7);
        MapDescriptor {
            noise,
            blocks,
            mesh,
        }
    }
}

impl MapDescriptor {
    fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    fn get_y(&self, x: i32, z: i32) -> i32 {
        let n = self.noise.get([x as f64, z as f64]) * 16. + 16.;
        n as i32
    }

    #[allow(dead_code)]
    fn min_max_y(&self) {
        let mut min = f64::MAX;
        let mut max = f64::MIN;
        for x in -1000..1000 {
            for z in -1000..1000 {
                let n = self.noise.get([x as f64, z as f64]);
                min = n.min(min);
                max = n.max(max);
            }
        }
        for x in -1000..1000 {
            for z in -1000..1000 {
                let n = self.noise.get([x as f64 + 0.5, z as f64 - 0.5]);
                min = n.min(min);
                max = n.max(max);
            }
        }
        println!("min: {min}\nmax: {max}");
    }
}
