use crate::acquire_midi_input;
use crate::algorithms::*;
use crate::state::State;
use huemanity::Bridge;

/// Macro to instantiate a vector of boxes. Convenient for use with [run].
#[macro_export]
macro_rules! boxvec {
    [$($a:expr), *] => {
        vec![$(Box::new($a) ), *]
    };
}

// TODO: add a go! expression pattern for singular list
/// Run a set of algorithms on  a given bridge, convenience wrapper around
/// [run].
///
/// ## Usage
/// ```rust
/// // -- snip --
///
/// let alg = crate::algorithms::Blink {duration: 1, midi_notes: None};
/// go!(bridge, [alg,  crate::algorithms::Debug]);
///
/// // -- snip --
/// ```
#[macro_export]
macro_rules! go {
    ($b:ident, [$($a:expr), *]) => {
        run(boxvec![$($a), *], $b)
    };
    ($b:ident, $a:expr) => {
        run(boxvec![$a], $b)
    };
}

/// The run function. This sends the callback to be executed on the bridge.
///
/// Note:
/// Special thanks to @Marli, @seri and @shepmaster on the Rust discord for
/// explaining `trait objects`.
pub fn run(algo: Vec<Box<dyn Algorithm>>, bridge: Bridge) {
    let (in_port, midi_in) = acquire_midi_input().unwrap();

    // create a state
    let state = State {
        hits: 0,
        start_timestamp: 0,
        last_hpm: 0,
        hit_tracker: Vec::new(),
    };

    let cb = move |stmp: u64, msg: &[u8], st: &mut State| {
        if msg[0] != 169 {
            for alg in algo.iter() {
                alg.go(stmp, msg, st, &bridge);
            }
        }
    };

    let _conn_in = midi_in
        .connect(in_port, "Connection from Rust", cb, state)
        .unwrap();

    // main loop
    loop {}
}
