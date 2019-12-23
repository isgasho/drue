use crate::midi::*;
use crate::state::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;

pub struct TieredColorSwap {
    pub base_color: [f32; 2],
    pub tiers: BTreeMap<usize, [f32; 2]>, // TODO: implement btreemap instead of vector
    pub measurement_seconds: f32,
    pub transition_milliseconds: u8,
}

impl Callback for TieredColorSwap {
    fn setup(&self, bridge: &huemanity::Bridge) {
        // set up starting light state
        bridge.state_all(&json!({"on":false, "bri":254, "transitiontime":1}));

        println!("Found these tiers:");
        for (thresh, color) in self.tiers.iter() {
            println!("Threshold: {} ({:?})", thresh, color);
        }
        sleep(Duration::from_secs(3));
        bridge.state_all(&json!({"on":true, "bri":254, "transitiontime":10, "xy":self.base_color}));
    }

    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &huemanity::Bridge) {
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
        }
    }
}

pub struct SimpleColorSwap {
    pub normal: [f32; 2],
    pub fast: [f32; 2],
    pub hpm_threshold: usize,
    pub measurement_seconds: f32,
    pub transition_milliseconds: u8,
}

impl Callback for SimpleColorSwap {
    fn setup(&self, bridge: &huemanity::Bridge) {
        // set up starting light state
        bridge.state_all(&json!({"on":true, "bri":254, "transitiontime":1, "xy":self.fast}));
        sleep(Duration::from_secs(1));
        bridge.state_all(&json!({"on":true, "bri":254, "transitiontime":1, "xy":self.normal}));
    }

    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &huemanity::Bridge) {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

        // check if enough time has passed
        let time_passed: bool = state.time_since_last(stamp) > self.measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                // TODO: implement common computations on the state itself
                let hpm = state.calculate_hpm(self.measurement_seconds);
                let thresh_reached = hpm >= self.hpm_threshold as usize;
                match thresh_reached {
                    true => {
                        bridge.state_all(&json!({"bri":254, "xy":self.fast, "transitiontime":self.transition_milliseconds}));
                    }
                    false => {
                        bridge
                            .state_all(&json!({"bri":254, "xy":self.normal, "transitiontime":self.transition_milliseconds}));
                    }
                };
                state.reset(stamp);
                println!("HPM: {} -> {:?}", hpm, thresh_reached);
            }
            (true, _) => {
                state.hits += 1;
            }

            (_, _) => (),
        }
    }
}

pub struct Blink;

impl Callback for Blink {
    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &huemanity::Bridge) {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));
        if hit {
            bridge.state_all(&json!({"on":false, "bri":254, "transitiontime":1}));
            sleep(Duration::from_millis(10));
            bridge.state_all(&json!({"on":true, "bri":254, "transitiontime":1}));
        }
    }
}

pub trait Callback: std::marker::Send + 'static {
    fn setup(&self, bridge: &huemanity::Bridge) {}
    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &huemanity::Bridge);
}

pub fn run(bridge: huemanity::Bridge, callback: impl Callback) {
    // TODO: Take in a function here and use that as a callback
    // acquire an input
    let (in_port, midi_in) = acquire_midi_input().unwrap();

    // create a state
    let mut state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
    };

    // do the setup for the algorithm
    callback.setup(&bridge);

    // connect
    let conn_in = midi_in
        .connect(
            in_port,
            "Connection from Rust",
            move |stamp, message, data| {
                callback.execute(stamp, message, data, &bridge);
            },
            state,
        )
        .unwrap();

    // main loop
    loop {}
}
