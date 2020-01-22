use crate::state::*;
use huemanity::Bridge;
use serde_json::json;
use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;
// TODO: look into adding a parameters option and a string/enum mapper to closure using this https://users.rust-lang.org/t/default-and-optional-parameter/27693/4

/// Constructs the blink callback. If you pass a midi note vector it will only trigger when these are
/// activated. Otherwise pass in `None` and it will trigger regardless of what pad is hit.
pub fn blink(
    duration: u8,
    midi_notes: Option<Vec<u8>>,
) -> impl Fn(u64, &[u8], &mut State, &Bridge) -> () {
    move |_stamp: u64, message: &[u8], _state: &mut State, bridge: &Bridge| {
        if message.len() == 3 {
            let hit: bool = (message.get(2) != Some(&0))
                & match &midi_notes {
                    Some(vec) => vec.contains(&message[1]),
                    None => true,
                }; // NOTE: Not sure if this way adds overhead.
            if hit {
                bridge.state_all(&json!({
                    "on":false,
                    "bri":254,
                    "transitiontime":duration}));
                sleep(Duration::from_millis(10));
                bridge.state_all(&json!({
                    "on":true,
                    "bri":254,
                    "transitiontime":duration}));
            }
        }
    }
}

/// Constructs the debug callback which is just a way to print out information
/// about every hit.
pub fn debug() -> impl Fn(u64, &[u8], &mut State, &Bridge) -> () {
    move |_stamp: u64, message: &[u8], _state: &mut State, bridge: &Bridge| {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));
        if hit {
            println!(
                "| drum code: {} | state code: {} | full message: {:?}|",
                message[1], message[0], message
            );
            println!("------------------------------------------------------");
        }
    }
}

/// Constructs the HPM threshold callback to be called at each hit.
pub fn hpm_threshold(
    base_color: [f32; 2],
    tiers: BTreeMap<usize, [f32; 2]>,
    measurement_seconds: f32,
    transition_milliseconds: u8,
) -> impl Fn(u64, &[u8], &mut State, &Bridge) -> () {
    move |stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge| {
        // TODO: Abstract this into the callback trait as an associated function
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

        // check if enough time has passed
        let time_passed: bool = state.time_since_last(stamp) > measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                // TODO: implement common computations on the state itself
                let hpm = state.calculate_hpm(measurement_seconds);

                let mut color = base_color;
                for (threshold, _color) in tiers.iter() {
                    if hpm >= *threshold {
                        color = *_color
                    }
                }

                bridge.state_all(
                    &json!({"bri":254, "xy":color, "transitiontime":transition_milliseconds}),
                );

                state.reset(stamp);
                println!("HPM: {} -> {:?}", hpm, color);
            }
            (true, _) => {
                state.hits += 1;
            }

            (_, _) => (),
        }
    }
}

/// Construct the variety threshold callback.
pub fn variety_threshold(
    below: [f32; 2],
    above: [f32; 2],
    variety_threshold: u8,
    measurement_seconds: f32,
    transition_milliseconds: u8,
) -> impl Fn(u64, &[u8], &mut State, &Bridge) -> () {
    move |stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge| {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

        // check if enough time has passed
        let time_passed: bool = state.time_since_last(stamp) > measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                let variety = state.hit_tracker.len();
                let thresh_reached = variety >= variety_threshold as usize;
                if thresh_reached {
                    bridge.state_all(
                        &json!({"bri":254, "xy":above, "transitiontime":transition_milliseconds}),
                    );
                } else {
                    bridge.state_all(
                        &json!({"bri":254, "xy":below, "transitiontime":transition_milliseconds}),
                    );
                };
                state.reset(stamp);
                println!("Variety: {} -> {:?}", variety, thresh_reached);
            }
            (true, _) => {
                if !state.hit_tracker.contains(&message[1]) {
                    state.hit_tracker.push(message[1]);
                }
            }

            (_, _) => (),
        }
    }
}
