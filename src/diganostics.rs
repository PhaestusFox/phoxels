use bevy::{
    diagnostic::{Diagnostic, DiagnosticsStore, RegisterDiagnostic},
    prelude::*,
};

pub const VOXEL_COUNT: bevy::diagnostic::DiagnosticPath =
    bevy::diagnostic::DiagnosticPath::const_new("Voxel Count");

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_ui)
        .add_systems(Update, (update_fps, update_voxel_count));
}

#[derive(Debug, Component)]
struct FPSText;

fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.),
            left: Val::Px(10.),
            width: Val::VMax(20.),
            height: Val::VMax(5.),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        BorderRadius::all(Val::VMax(5.)),
        BackgroundColor(Color::linear_rgb(0.6, 0.6, 0.6)),
        children![
            (Text::new("NIY"), FPSText),
            (Text::new("NIY"), VoxelCountText),
            (Text::new("NIY"), VoxelMeshText),
        ],
    ));
}

#[derive(Debug, Component)]
struct VoxelCountText;

#[derive(Debug, Component)]
struct VoxelMeshText;

fn update_fps(mut text: Single<&mut Text, With<FPSText>>, diagnostics: Res<DiagnosticsStore>) {
    if let Some(fps) = diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(smooth) = fps.smoothed() {
            text.0 = format!("{:.02} FPS", smooth);
            // println!("{:.02}", smooth);
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct VoxelCount(u32);

impl std::fmt::Display for VoxelCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} Voxels", self.0))
    }
}

impl VoxelCount {
    #[allow(dead_code)]
    pub fn inc(&mut self) {
        self.0 += 1;
    }
    #[allow(dead_code)]
    pub fn dec(&mut self) {
        self.0 -= 1;
    }

    pub fn add(&mut self, by: u32) {
        self.0 += by;
    }
}

fn update_voxel_count(
    mut text_loaded: Single<&mut Text, (With<VoxelCountText>, Without<VoxelMeshText>)>,
    mut text_mesh: Single<&mut Text, (With<VoxelMeshText>, Without<VoxelCountText>)>,
    diagnostics: Res<phoxels::prelude::VoxelCount>,
) {
    if diagnostics.is_changed() {
        text_loaded.0 = diagnostics.loaded.to_string();
        text_mesh.0 = diagnostics.meshed.to_string();
    }
}
