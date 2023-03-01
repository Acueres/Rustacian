use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod action;
mod coord;
mod dir;
mod gene;
mod grid;
mod models;
mod ns;
mod organism;
mod sim;
mod systems;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(sim::SimPlugin)
        .run();
}
