use crate::resources::*;
use crate::systems::*;
use bevy::prelude::*;

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTime {
            timer: Timer::from_seconds(0.05, TimerMode::Repeating),
        })
        .insert_resource(EpochTime {
            timer: Timer::from_seconds(10., TimerMode::Repeating),
        })
        .insert_resource(SimState {
            paused: false,
            reset: false,
            epoch: 0,
        })
        .add_startup_system(startup_system)
        .add_system(input_system)
        .add_system(sim_step)
        .add_system(advance_epoch)
        .add_system(reset_world)
        .add_system(energy_text_system)
        .add_system(epoch_text_system)
        .add_system(population_text_system);
    }
}
