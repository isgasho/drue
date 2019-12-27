# Drue

## Polite notice

This tool is in heavy development and it is in a state where it works for me
`¯\_(ツ)\_/¯`. I will update this as it goes along, but I would really appreciate
pull requests and issues posted.

# What is it?

This crate links a midi input to the hue lights in my house. It is able to use
one of the several implemented 'algorithms' to adjust the lights. It does so by
estimating the `HPM` or `hits-per-minute` and using that to adjust the colour or
behaviour of the lights. It uses a simple crate I wrote called
[huemanity](https://github.com/finnkauski/huemanity) as the interface the
lights.

Currently implemented behaviour:

- `SimpleColorSwap` -> provide 2 colors and a `HPM` threshold and it will change
  the lights once the threshold has been reached.

- `TieredColorSwap` -> provide a mapping of thresholds and colours and it will
  use those to pick the colour as you play

- `Blink` -> this is cool, but unfortunately not well implemented. It blink the
  lights with each hit. However, due to the non-async implementation of the
  state change for the lights and the delay/inability to stop the signals being
  sent to the lights, you end up spamming them and then the actions get queued
  up.

## Ambitions

- Expand the capability of the
  [huemanity](https://github.com/finnkauski/huemanity) crate to allow easier
  registration and async request sending.
- A drum kit explorer function for users to see what their drums are mapped to.
  This will help with the next point.
- The ability to map more complex interactions, such as per drum pad specific
  behaviours
- Configuration files that are able to be parsed into settings for algorithms
- Potential settings for individual songs stored and cached in a database like
  storage

For more see the [IDEAS](IDEAS.org) file.

## Issues

- Tons of issues, help appreciated although I realise that drummers who have
  E-Drums and are into coding Rust aren't that widespread.

- No ability to set which lights act on command, the implementation of the light
  behaviours updates the states of all lights.

- Currently the state of the code means you have to compile from source to pick
  the desired behaviour. This is mainly because I am developing it, but soon I
  will enable a CLI interface to alleviate the problem.

- The [huemanity](https://github.com/finnkauski/huemanity) crate is rudimentary
  and currently doesn't support registration to the router/bridge. So in order
  to use this crate you need to register with the router as detailed
  [here](https://developers.meethue.com/develop/get-started-2/) in the
  `/newdeveloper` workflow and store the key provided as well as the bridge ip
  in a `.env` file in the root directory or in your environment under `HUE_IP`
  and `HUE_KEY` variables.

- Documentation is rubbish and the API design is something that I'm adding to
  rather than thinking this out in advance if someone more skilled in Rust can
  knock some sense into me, that would be much appreciated.

- I don't have any other things on my bridge so I can't test if this works the
  same when you have weird lights connected to the bridge.

## How to run

Ok, if you're still interested and want to run this yourself then read on.

1. Clone the repo
2. Follow [this guide ](https://developers.meethue.com/develop/get-started-2/)
   until you've registered your app and gotten a key.
3. Note down this key and store it in an `.env` file in the root of the repo or
   somehow just set it into your environment variables. You need these
   variables:
   - `HUE_IP` - router IP on your network
   - `HUE_KEY` - the key received after the new developer registration with your
     bridge.
