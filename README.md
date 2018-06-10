# Remote volume mixer written in Rust

voler *fr* (“to steal”)

I got tired of alt-tabbing out of games to adjust the volume relative to VoIP applications when playing with friends. This is currently **abandoned**, as I don't have the time to learn Rust properly.

## What works

* Receiving notifications from PulseAudio about volume changes
* Sending updates through [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
* Reacting to slider events and receiving them at the server via POST

Based on https://github.com/kchmck/uhttp_sse.rs

## Usage

Run `tsc index.ts` to compile the TypeScript down to plain JavaScript.

`cargo run` will pull all the necessary dependencies the first time you run it and launch the server.

Visit http://localhost:7878/ and change the volume of an application (e.g. by using *PulseAudio Volume Control* or the volume slider in Spotify) to make the corresponding slider appear.
