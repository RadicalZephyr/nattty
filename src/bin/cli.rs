use std::io::BufRead;

use sodium::{SodiumCtx, StreamSink};

use nattty::{Board, Error, Mark, TicTacToe};

fn main() {
    let ctx = SodiumCtx::new();

    let (start_game, kb_input, _listeners) = ctx.transaction(|| {
        let mut listeners = Vec::new();

        let start_game: StreamSink<()> = ctx.new_stream_sink();
        let kb_input: StreamSink<String> = ctx.new_stream_sink();

        let game = TicTacToe::new(&kb_input, &ctx);

        listeners.push(start_game.stream().listen({
            let turn = game.turn.clone();
            let board = game.board.clone();
            move |_: &()| {
                println!("Welcome to Tic Tac Toe!");
                println!("{:?} plays first!\n", turn.sample());
                println!("{}", board.sample());
            }
        }));

        listeners.push(game.error.listen(|err: &Error| println!("{}", err)));

        listeners.push(game.moves.listen(|(index, mark): &(usize, Mark)| {
            println!("\n{:?}s took space {}:", mark, index)
        }));

        listeners.push(
            game.board
                .updates()
                .listen(|board: &Board| println!("{}", board)),
        );

        listeners.push(
            game.winner
                .listen(|mark: &Mark| println!("{:?} has won the game!", mark)),
        );

        (start_game, kb_input, listeners)
    });

    let stdin = std::io::stdin().lock();

    start_game.send(());
    for line in stdin.lines() {
        kb_input.send(line.unwrap());
    }
}
