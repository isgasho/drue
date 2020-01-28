<p align="center"><img align="left" src="meta/logo.png" width="350px"></p>

## Hey, heads up!

This tool is in heavy development and it is in a state where it works for me
`¯\_(ツ)\_/¯`. I will update this as it goes along, but I would really appreciate
pull requests and issues posted.

# What is it?

[Demo](https://www.youtube.com/watch?v=fEK2DofSwEE)

This crate and executable link a midi input to the hue lights in a house. It
is able to use one of the several implemented 'algorithms' to adjust the lights.
It uses a simple crate I wrote called
[huemanity](https://github.com/finnkauski/huemanity) as the interface the
lights.

# Currently implemented behaviour:

- `HPM` -> This uses a tier system and swaps colors relative to the number of
  hits you are making. Default behaviour is

  1. Start counting hits
  1. Check how many hits have been made in `0.7` seconds
  1. Estimate how many hits you would have made if you kept the same pace
  1. Reset counter
  1. Check where you are in the `hpm` to `color` mapping
  1. Send the new color to the lights

- `Blink` -> Currently it has 2 modes. If the `-p <PAD>` parameter is provided,
  the lights will blink only when you hit that note. If you don't provide a pad
  mapping it will blink on all hits. To figure out which pad is which, use the
  `Debug` mode and hit some of the pads.

- `Variety` -> This currently works similarly to the `HPM` method, with the
  exception that it measures the number of distinct drum pads hit. If you more
  than 3 different pads it will trigger a color change.

- `Debug` -> This executes the `Blink` method, but also prints information about
  what you hit in the terminal

## Ambitions

- Expand the capability of the
  [huemanity](https://github.com/finnkauski/huemanity) crate to allow easier
  registration and async request sending or find a more sustainable alternative.
- A drum kit explorer function for users to see what their drums are mapped to.
  At the moment it can only be done with `Debug`
- Configuration files that are able to be parsed into settings for algorithms
- Potential settings for individual songs stored and cached in a database like
  storage

For more see the [IDEAS](IDEAS.org) file.

## Issues

- Tons of issues, help appreciated although I realise that drummers who have
  E-Drums and are into coding Rust aren't that widespread. EDIT: AND have HUE
  lights!

- No ability to set which lights act on command, the implementation of the light
  behaviours updates the states of all lights.

- The [huemanity](https://github.com/finnkauski/huemanity) crate is rudimentary.

- Documentation is rubbish and the API design is something that I'm adding to
  rather than thinking this out in advance if someone more skilled in Rust can
  knock some sense into me, that would be much appreciated.

- I don't have any other things on my bridge so I can't test if this works the
  same when you have weird lights connected to the bridge.

- No tests or CI

and so on...

## How to run

Ok, if you're still interested and want to run this yourself then read on.

1. Clone the repo and CD into the directory
2. Run using the following command (assuming `cargo` is installed):

```sh
cargo run
```
