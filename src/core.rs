use crate::acquire_midi_input;
use crate::algorithms::*;
use crate::state::State;
use huemanity::Bridge;

/// The run function. This sends the callback to be executed on the bridge.
pub fn run<'a>(
    callback: impl Fn(u64, &[u8], &mut State, &Bridge) -> ()
        + std::marker::Send
        + std::marker::Sync
        + 'static,
    bridge: Bridge,
) {
    let (in_port, midi_in) = acquire_midi_input().unwrap();

    // create a state
    let mut state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
        hit_tracker: Vec::new(),
    };

    let cb = move |stmp: u64, msg: &[u8], st: &mut State| {
        callback(stmp, msg, st, &bridge);
        callback(stmp, msg, st, &bridge);
    };
    let conn_in = midi_in
        .connect(in_port, "Connection from Rust", cb, state)
        .unwrap();
    // main loop
    loop {}
}

// TODO: remove
// let t1 = test();
// let t2 = test();
// let v = [1, 1];
// let algo = chain(t1, t2)(1, &v, &mut state, &bridge);
// println!("{:?}", algo);

// connect
// let callback = |stamp, message: &[u8], data: &'s mut State| {
//     if message[0] != 169 {
//         // avoid hi hat issues when using foot pedal. Normally this sends multiple messages
//         callback(stamp, message, data, &bridge);
//     }
//     data
// };
//     let conn_in = midi_in
//         .connect(
//             in_port,
//             "Connection from Rust",
//             move |t, msg, st: &mut State| {
//                 callback(t, msg, st, bridge);
//             },
//             state,
//         )
//         .unwrap();
//     // main loop
//     loop {}
// }
