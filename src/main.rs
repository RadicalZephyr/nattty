#![allow(dead_code)]

use std::io::BufRead;

use board::{Board, Mark};
use sodium::{SodiumCtx, StreamSink};

pub(crate) mod board;

mod setup;
use setup::{set_up_play, Error};

fn main() {
    let ctx = SodiumCtx::new();

    let (kb_input, _listeners) = ctx.transaction(|| {
        let mut listeners = Vec::new();
        let kb_input: StreamSink<String> = ctx.new_stream_sink();

        let game = set_up_play(&kb_input, &ctx);

        listeners.push(game.error.listen(|err: &Error| println!("{}", err)));

        // listeners.push(index_mark_stream.listen(|(index, mark): &(usize, Mark)| {
        //     println!("\nMark an {:?} at index {}:", mark, index)
        // }));

        listeners.push(
            game.board
                .updates()
                .listen(|board: &Board| println!("{}", board)),
        );

        listeners.push(
            game.winner
                .listen(|mark: &Mark| println!("{:?} has won the game!", mark)),
        );

        (kb_input, listeners)
    });

    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        kb_input.send(line.unwrap());
    }
}
