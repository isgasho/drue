#[macro_use]
extern crate clap;
use crate::core::*;
use algorithms::*;
use drue::*;
use huemanity::Bridge;

// TODO: Implement default algorithms
/// Main entrypoint for the CLI
fn main() {
    println!(
        "
________________________________________________________________________________

          ██████╗ ██████╗ ██╗   ██╗███████╗
          ██╔══██╗██╔══██╗██║   ██║██╔════╝
          ██║  ██║██████╔╝██║   ██║█████╗
          ██║  ██║██╔══██╗██║   ██║██╔══╝
          ██████╔╝██║  ██║╚██████╔╝███████╗
          ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚══════╝
________________________________________________________________________________


"
    );

    // Collect command line arguments
    let matches = create_app().get_matches();
    let bridge = Bridge::link();
    println!("Sucessfully linked to a bridge");

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

/// Simply creates a `clap` app with certain parsing capabilities and which is the main entry
/// point for the tool.
pub fn create_app() -> clap::App<'static, 'static> {
    let app = clap_app!(drue =>
                            (version: "0.1")
                            (author: "Art Eidukas <iwiivi@gmail.com>")
                            (@arg METHOD: -m --method +takes_value "Pick a method: blink|variety|hpm|debug")
                            (@arg PAD: -p --pad +takes_value "Pick which pad gets mapped to specialised blinking")
    );

    // TODO: make sure this is consistent with the way the API is designed
    app
}
