use std::io::BufRead;

use sodium::{SodiumCtx, StreamSink};

use nattty::{AppState, Board, Error, Mark, SequenceOfGames, TicTacToe};

fn main() {
    let ctx = SodiumCtx::new();

    let (boot, kb_input, _listeners) = ctx.transaction(|| {
        let mut listeners = Vec::new();

        let boot: StreamSink<()> = ctx.new_stream_sink();
        let kb_input: StreamSink<String> = ctx.new_stream_sink();

        let kb_stream = kb_input.stream();
        let game_seq = SequenceOfGames::new(&ctx, &boot.stream(), &kb_stream);

        listeners.push(boot.stream().listen(|_: &()| {
            println!("Welcome to Tic Tac Toe!\n");
        }));
        listeners.push(game_seq.prompt_player_name.listen(|_: &()| {
            println!("Who is playing today?");
        }));

        let playing_cell = game_seq
            .app_state
            .map(|app_state: &AppState| app_state.is_playing());
        let game = TicTacToe::new(&ctx, &kb_stream.gate(&playing_cell));

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
            println!("\n{:?}s took space {}:", mark, index + 1)
        }));

        listeners.push(game.board.updates().listen({
            let players = game_seq.players.clone();
            let turn = game.turn.clone();
            move |board: &Board| {
                let players = players.sample();
                let mark = turn.sample().swap();
                println!("{}", board);
                println!("{}'s turn to play an {:?}", players.get_name(&mark), mark);
            }
        }));

        listeners.push(game.winner.listen({
            let players = game_seq.players.clone();
            move |mark: &Mark| {
                let players = players.sample();
                println!(
                    "{} playing {:?}s has won the game!",
                    players.get_name(mark),
                    mark
                );
            }
        }));

        (boot, kb_input, listeners)
    });

    let stdin = std::io::stdin().lock();

    boot.send(());
    for line in stdin.lines() {
        kb_input.send(line.unwrap());
    }
}
