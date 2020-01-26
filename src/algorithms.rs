use crate::state::*;
use huemanity::Bridge;
use serde_json::json;
use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;
// TODO: look into adding a parameters option and a string/enum mapper to closure using this https://users.rust-lang.org/t/default-and-optional-parameter/27693/4

/// # Blink
///
/// Constructs the blink callback. If you pass a midi note vector it will only trigger when these
/// are activated. Otherwise pass in [None] and it will trigger regardless of what pad is hit.
pub fn blink(
    duration: u8,
    midi_notes: Option<Vec<u8>>,
) -> impl Fn(u64, &[u8], &mut State, &Bridge) -> () {
    move |stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge| {
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

/// # Debug
/// Constructs the debug callback which is just a way to print out information
/// about every hit.
pub fn debug() -> impl Fn(u64, &[u8], &mut State, &Bridge) -> () {
    move |stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge| {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));
        if hit {
            println!(
                "| drum code: {} | state code: {} | full message: {:?}|",
                message[1], message[0], message
            );
            println!("------------------------------------------------------");
        };
    }
}

/// # HPM (Hits-Per-Minute)
///
/// Constructs the HPM threshold callback to be called at each hit.
///
/// In order to clarify the behaviour of this type of mapping, here is a more structured example
/// of how the given parameters affect the outcome.
///
/// - `base_color`              sets up a base color that will linger when none of the thresholds
/// are met
/// - `tiers`                   specifies the mapping of `hpm => colors`, the lights will change
/// to the color when the hpm is reached
/// - `measurement_seconds`     this dictates how often the hpm should be reassesed
/// - `transition_milliseconds` sets how fast the transition from one color to the other should
/// happen
pub fn hpm_threshold<'a>(
    base_color: [f32; 2],
    tiers: &'a BTreeMap<usize, [f32; 2]>,
    measurement_seconds: f32,
    transition_milliseconds: u8,
) -> impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> () + '_ {
    move |stamp: u64, message: &'a [u8], state: &'a mut State, bridge: &'a Bridge| {
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
        };
    }
}

/// # Variety
///
/// Constructs a midi mapping that tracks the variety of drum pads hit and maps the lights
/// based on variety.
///
/// The parameters to this function specify the color to be used before the threshold is hit,
/// the color to be used once the variety goes above the threshold, the threshold itself and
/// (similarly to [hpm_threshold]) the measurement period and transition period for the hpm and
/// color changes respectively.
///
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
        };
    }
}

/// A test drum mapping constructor, mainly there to debug composition macros.
pub fn test_algo() -> impl Fn(u64, &[u8], &mut State, &Bridge) -> () {
    move |stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge| {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));
        if hit {
            println!("Hello");
        }
    }
}

/// Macro that composes created algorithms. It uses [compose2] in a recursive way
/// to fold the provided functions.

/// This is intended to be used to construct consequtively execute several mappings
/// of drumkit pads to lights.
#[macro_export]
macro_rules! algo {
    ($($f:expr),*) => {
        move | a1: u64, a2: &[u8], a3: &mut crate::state::State, a4: &Bridge| {
            $( $f(a1, a2, a3, a4);) *
        };
    };
}
// TODO: WHY NOT JUST DO FUNCTION CREATING MACROS?
