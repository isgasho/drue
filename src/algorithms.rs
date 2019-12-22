use crate::state::*;
use crate::utils::*;
use serde_json::json;
use std::collections::BTreeMap;
use std::process::Command;
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

        // check how many seconds have passed
        let time_passed: bool = state.time_since_last(stamp) > self.measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                // TODO: implement common computations on the state itself
                let hpm = ((state.hits * 60) as f32 / self.measurement_seconds) as usize;

                let mut color = self.base_color;
                for (threshold, _color) in self.tiers.iter() {
                    if hpm >= *threshold {
                        color = *_color
                    }
                }

                bridge.state_all(
                    &json!({"bri":254, "xy":color, "transitiontime":self.transition_milliseconds}),
                );

                state.hits = 0;
                state.start_timestamp = stamp;
                println!("{} -> {:?}", hpm, color);
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

        // check how many seconds have passed
        let time_passed: bool = state.time_since_last(stamp) > self.measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                // TODO: implement common computations on the state itself
                let hpm = state.hits * 60 / self.measurement_seconds as usize;
                let thresh_reached = hpm >= self.hpm_threshold as usize;
                let color = match thresh_reached {
                    true => {
                        bridge.state_all(&json!({"bri":254, "xy":self.fast, "transitiontime":self.transition_milliseconds}));
                    }
                    false => {
                        bridge
                            .state_all(&json!({"bri":254, "xy":self.normal, "transitiontime":self.transition_milliseconds}));
                    }
                };
                state.hits = 0;
                state.start_timestamp = stamp;
                println!("{} -> {:?}", hpm, thresh_reached);
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
