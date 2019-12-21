use crate::state::*;
use crate::utils::*;
use serde_json::json;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

pub fn color_callback(stamp: u64, message: &[u8], state: &mut State) {}

pub struct ColorRamp;

impl Callback for ColorRamp {
    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &huemanity::Bridge) {}
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

pub struct NodeAppColorRamp;

impl Callback for NodeAppColorRamp {
    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &huemanity::Bridge) {
        let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

        // check how many seconds have passed
        let threshold: u8 = 2; // seconds
        let threshold_met: bool = (stamp - state.start_timestamp) / 1000000 > threshold as u64;

        match (hit, threshold_met) {
            (_, true) => {
                let hpm = (state.hits as u64) * (60 / threshold) as u64;
                let color = apm_to_hex(hpm);
                state.last_hpm = hpm;
                state.hits = 0;
                state.start_timestamp = stamp;
                println!("{} -> {}", hpm, color);
                Command::new("hue")
                    .arg("lights")
                    .arg("1,2,3")
                    .arg(color)
                    .output();
            }
            (true, _) => {
                state.hits += 1;
            }
            (_, _) => (),
        }
    }
}

pub trait Callback: std::marker::Send + 'static {
    fn execute(&self, stamp: u64, message: &[u8], state: &mut State, bridge: &huemanity::Bridge);
}
