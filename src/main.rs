#![allow(dead_code)]

use std::io::BufRead;

use sodium::{SodiumCtx, StreamSink};

pub(crate) mod board;

mod setup;
use setup::set_up_play;

fn main() {
    let ctx = SodiumCtx::new();

    let (kb_input, _listeners) = ctx.transaction(|| {
        let mut listeners = Vec::new();
        let kb_input: StreamSink<String> = ctx.new_stream_sink();

        set_up_play(&kb_input, &mut listeners, &ctx);

        (kb_input, listeners)
    });

    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        kb_input.send(line.unwrap());
    }
}
