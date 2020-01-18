use crate::acquire_midi_input;
use crate::algorithms::{AlgoFactory, Callback};
use crate::state::State;

/// The custom_run function. This sends the callback to be executed on the bridge.
pub fn run(bridge: huemanity::Bridge, callback: impl Callback) {
    // TODO: Take in a function here and use that as a callback
    // acquire an input
    let (in_port, midi_in) = acquire_midi_input().unwrap();

    // create a state
    let mut state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
        hit_tracker: Vec::new(),
    };

    // do the setup for the algorithm
    callback.setup(&bridge);

    // connect
    let conn_in = midi_in
        .connect(
            in_port,
            "Connection from Rust",
            move |stamp, message, data| {
                if message[0] != 169 {
                    // avoid hi hat issues when using foot pedal. Normally this sends multiple messages
                    callback.execute(stamp, message, data, &bridge);
                }
            },
            state,
        )
        .unwrap();
    // main loop
    loop {}
}

/// The run test with algo factory
pub fn run_test(factory: AlgoFactory) {
    // acquire an input
    let (in_port, midi_in) = acquire_midi_input().unwrap();

    // create a state
    let mut state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
        hit_tracker: Vec::new(),
    };

    let callback = factory.threshold([1.0, 1.0], [1.0, 0.0], 600, 0.7, 1);

    // connect
    let conn_in = midi_in
        .connect(
            in_port,
            "Connection from Rust",
            move |stamp, message, data| {
                if message[0] != 169 {
                    // avoid hi hat issues when using foot pedal. Normally this sends multiple messages
                    callback(stamp, message, data);
                }
            },
            state,
        )
        .unwrap();
    // main loop
    loop {}
}
