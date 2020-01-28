use crate::state::*;
use huemanity::Bridge;
use serde_json::json;
use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;
// TODO: look into adding a parameters option and a string/enum mapper to closure using this https://users.rust-lang.org/t/default-and-optional-parameter/27693/4

/// # Blink algorithm => blink on hit
///
/// Constructs the blink callback. If you pass a midi note vector it will only trigger when these
/// are activated. Otherwise pass in [None] and it will trigger regardless of what pad is hit.
pub struct Blink {
    pub duration: u8,
    pub midi_notes: Option<Vec<u8>>,
}

impl Algorithm for Blink {
    fn go(&self, _stamp: u64, message: &[u8], _state: &mut State, bridge: &Bridge) {
        if message.len() == 3 {
            let hit: bool = (message.get(2) != Some(&0));
            match &self.midi_notes {
                Some(vec) => vec.contains(&message[1]),
                None => true,
            }; // NOTE: Not sure if this way adds overhead.
            if hit {
                bridge.state_all(&json!({
                    "on":false,
                    "bri":254,
                    "transitiontime":self.duration}));
                sleep(Duration::from_millis(10));
                bridge.state_all(&json!({
                    "on":true,
                    "bri":254,
                    "transitiontime":self.duration}));
            }
        }
    }
}

impl Default for Blink {
    fn default() -> Self {
        Self {
            duration: 1,
            midi_notes: None,
        }
    }
}

/// # Debug => prints debug info
/// Constructs the debug callback which is just a way to print out information
/// about every hit.
#[derive(Default)]
pub struct Debug;
impl Algorithm for Debug {
    fn go(&self, _stamp: u64, message: &[u8], _state: &mut State, _bridge: &Bridge) {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));
        if hit {
            println!("------------------------------------------------------");
            println!(
                "| drum code: {} | state code: {} | full message: {:?}|",
                message[1], message[0], message
            );
            println!("------------------------------------------------------");
        };
    }
}

/// # HPM (Hits-Per-Minute) => look at HPM
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
pub struct HPM {
    pub base_color: [f32; 2],
    pub tiers: BTreeMap<usize, [f32; 2]>,
    pub measurement_seconds: f32,
    pub transition_milliseconds: u8,
}

impl Algorithm for HPM {
    fn go(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge) {
        // TODO: Abstract this into the callback trait as an associated function
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

        // check if enough time has passed
        let time_passed: bool = state.time_since_last(stamp) > self.measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                // TODO: implement common computations on the state itself
                let hpm = state.calculate_hpm(self.measurement_seconds);

                let mut color = self.base_color;
                for (threshold, _color) in self.tiers.iter() {
                    if hpm >= *threshold {
                        color = *_color
                    }
                }

                bridge.state_all(
                    &json!({"bri":254, "xy":color, "transitiontime":self.transition_milliseconds}),
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

impl Default for HPM {
    fn default() -> Self {
        // default implementation of HPM based thresholding
        // some color tiers for hpm
        let mut default_tiers = BTreeMap::new();
        default_tiers.insert(0, [1.0, 1.0]); // midnight blue
        default_tiers.insert(200, [0.1585, 0.0884]); // midnight blue
        default_tiers.insert(600, [1.0, 0.0]); // redish

        Self {
            base_color: [0.3174, 0.3207],
            tiers: default_tiers,
            measurement_seconds: 1,
            transition_milliseconds: 1,
        }
    }
}

/// # Variety => look the drum variety
///
/// Constructs a midi mapping that tracks the variety of drum pads hit and maps the lights
/// based on variety.
///
/// The parameters to this function specify the color to be used before the threshold is hit,
/// the color to be used once the variety goes above the threshold, the threshold itself and
/// (similarly to [HPM]) the measurement period and transition period for the hpm and
/// color changes respectively.
///
pub struct Variety {
    pub below: [f32; 2],
    pub above: [f32; 2],
    pub variety_threshold: u8,
    pub measurement_seconds: f32,
    pub transition_milliseconds: u8,
}
impl Algorithm for Variety {
    fn go(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge) {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

        // check if enough time has passed
        let time_passed: bool = state.time_since_last(stamp) > self.measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                let variety = state.hit_tracker.len();
                let thresh_reached = variety >= self.variety_threshold as usize;
                if thresh_reached {
                    bridge.state_all(
                        &json!({"bri":254, "xy":self.above, "transitiontime":self.transition_milliseconds}),
                    );
                } else {
                    bridge.state_all(
                        &json!({"bri":254, "xy":self.below, "transitiontime":self.transition_milliseconds}),
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

impl Default for Variety {
    fn default() -> Self {
        Self {
            below: [1.0, 1.0],
            above: [1.0, 0.0],
            variety_threshold: 3,
            measurement_seconds: 0.7,
            transition_milliseconds: 1,
        }
    }
}

/// The algorithm trait. It is essential that any new struct that wants to
/// be used as a custom algorithm implements this.
pub trait Algorithm: std::marker::Send + std::marker::Sync + 'static {
    fn go(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge) {}
}
