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
pub fn blink<'a>(
    duration: u8,
    midi_notes: Option<Vec<u8>>,
) -> impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge)
{
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

        (stamp, message, state, bridge)
    }
}

/// # Debug
/// Constructs the debug callback which is just a way to print out information
/// about every hit.
pub fn debug<'a>(
) -> impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge)
{
    move |stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge| {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));
        if hit {
            println!(
                "| drum code: {} | state code: {} | full message: {:?}|",
                message[1], message[0], message
            );
            println!("------------------------------------------------------");
        };

        (stamp, message, state, bridge)
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
) -> impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge)
{
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
        };

        (stamp, message, state, bridge)
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
pub fn variety_threshold<'a>(
    below: [f32; 2],
    above: [f32; 2],
    variety_threshold: u8,
    measurement_seconds: f32,
    transition_milliseconds: u8,
) -> impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge)
{
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
        (stamp, message, state, bridge)
    }
}

/// A test drum mapping constructor, mainly there to debug composition macros.
fn test_algo<'a>(
) -> impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge)
{
    move |stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge| {
        (stamp, message, state, bridge)
    }
}

/// Composes two functions one after another.
///
/// Refer, to the type signatures to see which functions are supported by this composition.
/// But in general this intends to provide a way to compose several drum-light behaviour mappings
///
/// Note: This function is only really useful when you have 2 algorithms (mappings) to compose. Use
/// [compose] when you want to compose more than 2.
pub fn compose_fns<'a>(
    f1: impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge),
    f2: impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge),
) -> impl Fn(u64, &'a [u8], &'a mut State, &'a Bridge) -> (u64, &'a [u8], &'a mut State, &'a Bridge)
{
    move |a1: u64, a2: &'a [u8], a3: &'a mut State, a4: &'a Bridge| {
        let args = f1(a1, a2, a3, a4);
        f2(args.0, args.1, args.2, args.3)
    }
}

/// Macro that composes created algorithms. It uses [compose2] in a recursive way
/// to fold the provided functions.
///
/// This is intended to be used to construct consequtively execute several mappings
/// of drumkit pads to lights.
#[macro_export]
macro_rules! compose {
    ($e:expr) => { $e };
    ($e:expr,  $($es:expr),+) => { compose_fns($e, compose!($($es),*)) };
}
