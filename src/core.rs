use crate::acquire_midi_input;
use crate::algorithms;
use crate::state::State;

/// The run function. This sends the callback to be executed on the bridge.
pub fn run(bridge: huemanity::Bridge) {
    let (in_port, midi_in) = acquire_midi_input().unwrap();
    let callback = algorithms::blink(1, Some(vec![49]));

    // create a state
    let mut state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
        hit_tracker: Vec::new(),
    };

    // connect
    let conn_in = midi_in
        .connect(
            in_port,
            "Connection from Rust",
            move |stamp, message, data| {
                if message[0] != 169 {
                    // avoid hi hat issues when using foot pedal. Normally this sends multiple messages
                    callback(stamp, message, data, &bridge);
                }
            },
            state,
        )
        .unwrap();
    // main loop
    loop {}
}
