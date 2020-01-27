#[macro_use]
use drue::*;
use crate::core::*;
use algorithms::*;
use huemanity::Bridge;

// TODO: Implement default algorithms
/// Main entrypoint for the CLI
fn main() {
    let bridge = Bridge::link();
    println!("Sucessfully linked to a bridge");

    // Collect command line arguments
    let matches = create_app().get_matches();

    match matches.value_of("METHOD") {
        Some("blink") => {
            if let Some(pad) = matches.value_of("PAD") {
                if let Ok(ipad) = pad.parse::<u8>() {
                    let new_blinkbug = Blink {
                        duration: 1,
                        midi_notes: Some(vec![ipad]),
                    };
                    go!(bridge, [new_blinkbug, Debug]);
                } else {
                    panic!("Can not parse the pads")
                }
            } else {
                go!(bridge, [Blink::default(), Debug]);
            }
        }
        Some("hpm") => go!(bridge, HPM::default()),
        Some("variety") => go!(bridge, Variety::default()),
        None => {
            println!("Incorrect method passed!");
        }
        _ => (),
    }
}
