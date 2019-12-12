extern crate clap;

pub fn create_app() -> clap::App<'static, 'static> {
    let app = clap_app!(drumosphere =>
                            (version: "0.1")
                            (author: "Art Eidukas <iwiivi@gmail.com>")
                            (about: "This app allows midi input to fire of shell commands.")
                            (@arg REFRESH_INTERVAL: -r --refresh +takes_value "Specify how often the actions per minute are calculated")
    );
    app
}
