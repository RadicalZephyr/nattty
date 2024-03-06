use std::io::BufRead;

use sodium::{SodiumCtx, StreamSink};

use nattty::{Board, Error, Mark, SequenceOfGames, TicTacToe};

fn main() {
    let ctx = SodiumCtx::new();

    let (boot, kb_input, _listeners) = ctx.transaction(|| {
        let mut listeners = Vec::new();

        let boot: StreamSink<()> = ctx.new_stream_sink();
        let kb_input: StreamSink<String> = ctx.new_stream_sink();

        let game_seq = SequenceOfGames::new(&ctx, &boot.stream(), &kb_input.stream());

        listeners.push(boot.stream().listen(|_: &()| {
            println!("Welcome to Tic Tac Toe!\n");
        }));
        listeners.push(game_seq.prompt_player_name.listen(|_: &()| {
            println!("Who is playing today?");
        }));

        let game = TicTacToe::new(&ctx, &kb_input);

        listeners.push(game_seq.start_game.listen({
            let turn = game.turn.clone();
            let board = game.board.clone();
            move |_: &()| {
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

        (boot, kb_input, listeners)
    });

    let stdin = std::io::stdin().lock();

    boot.send(());
    for line in stdin.lines() {
        kb_input.send(line.unwrap());
    }
}
