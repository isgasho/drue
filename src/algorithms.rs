use crate::state::*;
use huemanity::Bridge;
use serde_json::json;
use std::collections::BTreeMap;
use std::thread::sleep;
use std::time::Duration;

pub struct VarietyThreshold {
    pub below: [f32; 2],
    pub above: [f32; 2],
    pub variety_threshold: u8,
    pub measurement_seconds: f32,
    pub transition_milliseconds: u8,
}

impl Callback for VarietyThreshold {
    fn setup(&self, bridge: &Bridge) {
        // set up starting light state
        bridge.state_all(&json!({"on":true, "xy":self.below}));
        bridge.state_all(&json!({"alert":"select"}));
    }
    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge) {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

        // check if enough time has passed
        let time_passed: bool = state.time_since_last(stamp) > self.measurement_seconds as f64;

        match (hit, time_passed) {
            (_, true) => {
                // main algorithm
                let variety = state.hit_tracker.len();
                let thresh_reached = variety >= self.variety_threshold as usize;
                if thresh_reached {
                    bridge.state_all(&json!({"bri":254, "xy":self.above, "transitiontime":self.transition_milliseconds}));
                } else {
                    bridge
                            .state_all(&json!({"bri":254, "xy":self.below, "transitiontime":self.transition_milliseconds}));
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

pub struct TieredThreshold {
    pub base_color: [f32; 2],
    pub tiers: BTreeMap<usize, [f32; 2]>, // todo: implement btreemap instead of vector
    pub measurement_seconds: f32,
    pub transition_milliseconds: u8,
}

impl Callback for TieredThreshold {
    fn setup(&self, bridge: &Bridge) {
        println!("Found these tiers:");
        for (thresh, color) in self.tiers.iter() {
            println!("Threshold: {} ({:?})", thresh, color);
        }
        sleep(Duration::from_secs(3));
        bridge.state_all(&json!({"bri":254, "transitiontime":10, "xy":self.base_color}));
    }

    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge) {
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

impl Callback for Threshold {
    fn setup(&self, bridge: &Bridge) {
        // set up starting light state
        bridge.state_all(&json!({"on":true, "xy":self.below}));
        bridge.state_all(&json!({"alert":"select"}));
    }

    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge) {
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
                        bridge.state_all(&json!({"bri":254, "xy":self.above, "transitiontime":self.transition_milliseconds}));
                    }
                    false => {
                        bridge
                            .state_all(&json!({"bri":254, "xy":self.below, "transitiontime":self.transition_milliseconds}));
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

pub struct Blink {
    pub duration: u8,
}

impl Callback for Blink {
    fn execute(&self, _stamp: u64, message: &[u8], _state: &mut State, bridge: &Bridge) {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));
        if hit {
            bridge.state_all(&json!({"on":false, "bri":254, "transitiontime":self.duration}));
            sleep(Duration::from_millis(10));
            bridge.state_all(&json!({"on":true, "bri":254, "transitiontime":self.duration}));
        }
    }
}

pub struct SpecialisedBlink {
    pub duration: u8,
    pub midi_notes: Vec<u8>,
}

impl Callback for SpecialisedBlink {
    fn execute(&self, _stamp: u64, message: &[u8], _state: &mut State, bridge: &Bridge) {
        if message.len() == 3 {
            let hit: bool = (message.get(2) != Some(&0)) & self.midi_notes.contains(&message[1]);
            if hit {
                bridge.state_all(&json!({"on":false, "bri":254, "transitiontime":self.duration}));
                sleep(Duration::from_millis(10));
                bridge.state_all(&json!({"on":true, "bri":254, "transitiontime":self.duration}));
            }
        }
    }
}

pub struct DummyPrint;

impl Callback for DummyPrint {
    fn execute(&self, _stamp: u64, message: &[u8], _state: &mut State, _bridge: &Bridge) {
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

pub trait Callback: std::marker::Send + 'static {
    /// The default initalisation procedure for a callback
    fn setup(&self, bridge: &Bridge) {
        // set up starting light state
        bridge.state_all(&json!({"on":true}));
        bridge.state_all(&json!({"alert":"select"}));
    }
    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &Bridge);
}

pub struct AlgoFactory {
    pub bridge: Bridge,
}

// TODO: set it up so the returned closures are composible
impl<'a> AlgoFactory {
    pub fn get_debug(self) -> impl Fn(u64, &[u8], &mut State) -> () {
        return |_stamp: u64, msg: &[u8], _state: &mut State| println!("{:?}", msg);
    }
    pub fn get_spec_blink(
        self,
        duration: u8,
        midi_notes: Vec<u8>,
    ) -> impl Fn(u64, &[u8], &mut State) -> () {
        return move |_stamp: u64, msg: &[u8], _state: &mut State| {
            if msg.len() == 3 {
                let hit: bool = (msg.get(2) != Some(&0)) & midi_notes.contains(&msg[1]);
                if hit {
                    self.bridge
                        .state_all(&json!({"on":false, "bri":254, "transitiontime":duration}));
                    sleep(Duration::from_millis(10));
                    self.bridge
                        .state_all(&json!({"on":true, "bri":254, "transitiontime":duration}));
                }
            }
        };
    }

    pub fn get_blink(self, duration: u8) -> impl Fn(u64, &[u8], &mut State) -> () {
        return move |_stamp: u64, msg: &[u8], _state: &mut State| {
            let hit: bool = (msg.len() == 3) & (msg.get(2) != Some(&0));
            if hit {
                self.bridge
                    .state_all(&json!({"on":false, "bri":254, "transitiontime":duration}));
                sleep(Duration::from_millis(10));
                self.bridge
                    .state_all(&json!({"on":true, "bri":254, "transitiontime":duration}));
            }
        };
    }

    pub fn threshold(
        &self,
        below: [f32; 2],
        above: [f32; 2],
        hpm_threshold: usize,
        secs: f32,
        transition: u8,
    ) -> impl Fn(u64, &[u8], &mut State) -> () {
        return move |stamp: u64, msg: &[u8], state: &mut State| {
            let hit: bool = (msg.len() == 3) & (msg.get(2) != Some(&0));

            // check if enough time has passed
            let time_passed: bool = state.time_since_last(stamp) > secs as f64;

            match (hit, time_passed) {
                (_, true) => {
                    // main algorithm
                    // TODO: implement common computations on the state itself
                    let hpm = state.calculate_hpm(secs);
                    let thresh_reached = hpm >= hpm_threshold as usize;
                    match thresh_reached {
                        true => {
                            self.bridge.state_all(
                                &json!({"bri":254, "xy":above, "transitiontime":transition}),
                            );
                        }
                        false => {
                            self.bridge.state_all(
                                &json!({"bri":254, "xy":below, "transitiontime":transition}),
                            );
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
        };
    }
}

pub struct Threshold {
    pub below: [f32; 2],
    pub above: [f32; 2],
    pub hpm_threshold: usize,
    pub measurement_seconds: f32,
    pub transition_milliseconds: u8,
}
