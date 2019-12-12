#[macro_use]
extern crate clap;
extern crate midir;

use std::error::Error;
use std::fmt;
use std::io::{stdin, stdout, Write};
use std::process::Command;
// use std::thread::sleep;
// use std::time::Duration;

// midi related
use midir::{Ignore, MidiInput};

// get the cli app imported
mod cli;
mod hue;

#[derive(Debug)]
struct State {
    hits: u8,
    start_timestamp: u64,
    last_hpm: u64,
}
// TODO: expand the state and allow custom states

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timestamp: {} hits: {} last_hpm: {} ",
            self.start_timestamp, self.hits, self.last_hpm
        )
    }
}

fn main() {
    let args = cli::create_app().get_matches();

    match run() {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err.description()),
    }
}
fn run() -> Result<(), Box<dyn Error>> {
    // acquire an input
    let (in_port, midi_in) = acquire_midi_input().unwrap();

    // create a state
    let mut state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
    };

    // connect
    let conn_in = midi_in.connect(
        in_port,
        "Connection from Rust",
        |stamp, message, data| {
            callback(stamp, message, data);
        },
        state,
    )?;

    // main loop
    while true {
        continue;
    }
    // TODO: figure out if this gets closed
    let (midi_in_, log_all_bytes) = conn_in.close();

    Ok(())
}
fn acquire_midi_input() -> Result<(usize, MidiInput), Box<dyn Error>> {
    let mut input = String::new();
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Check which ports are available
    println!("Availabe input ports:");
    for input in 0..midi_in.port_count() {
        println!("{}: {}", input, midi_in.port_name(input).unwrap());
    }

    print!("Please select input port: ");
    stdout().flush()?;
    stdin().read_line(&mut input)?;
    let in_port: usize = input.trim().parse()?;
    Ok((in_port, midi_in))
}
// logic & algorithms
// turn this into a struct potentially
fn callback(stamp: u64, message: &[u8], state: &mut State) {
    let hit: bool = (message.len() == 3) & (message.get(2) != Some(&0));

    // check how many seconds have passed
    let threshold: u8 = 5; // seconds
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
// colors mappers
// parts of the code responsible for mapping outputs to colours.
fn apm_to_hex(value: u64) -> String {
    // this will need tweaking over time
    let constrained_hexed = (value as f64) / 4.0 / 16.0;
    let (first, second) = (constrained_hexed.floor(), constrained_hexed.fract() * 16.0);
    return format!("{:x}{:x}0000", first as i8, second as i8);
}
