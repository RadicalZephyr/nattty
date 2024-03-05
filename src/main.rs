#![allow(dead_code)]

use std::{io::BufRead, num::ParseIntError};

use sodium::{Cell, CellLoop, SodiumCtx, Stream, StreamSink};

mod board;

use board::{Board, Mark};

fn set_up_play(
    kb_input: &StreamSink<String>,
    listeners: &mut Vec<sodium::Listener>,
    ctx: &SodiumCtx,
) {
    let board_cell_loop: CellLoop<Board> = ctx.new_cell_loop();
    let board_cell_fwd = board_cell_loop.cell();

    let kb_stream = kb_input.stream();
    let valid_index_stream = validate_index(&kb_stream, &board_cell_fwd);

    // Alternate marks
    let turn_cell = mark_swapping(ctx, &valid_index_stream);

    let index_mark_stream =
        valid_index_stream.snapshot(&turn_cell, |index: &usize, turn: &Mark| (*index, *turn));
    listeners.push(index_mark_stream.listen(|(index, mark): &(usize, Mark)| {
        println!("\nMark an {:?} at index {}:", mark, index)
    }));

    let board_stream = &valid_index_stream.snapshot3(
        &board_cell_fwd,
        &turn_cell,
        |index: &usize, board: &Board, mark: &Mark| board.mark(*index, *mark),
    );
    let board_cell = board_stream.hold(Board::new());
    board_cell_loop.loop_(&board_cell);

    listeners.push(
        board_cell
            .updates()
            .listen(|board: &Board| println!("{}", board)),
    );

    let winner_stream = board_stream
        .map(|board: &Board| board.get_winner())
        .filter_option();
    listeners.push(winner_stream.listen(|mark: &Mark| println!("{:?} has won the game!", mark)));
}

fn validate_index(input_stream: &Stream<String>, board_cell: &Cell<Board>) -> Stream<usize> {
    let parsed_result_stream = &input_stream.map(|line: &String| line.parse::<usize>());

    // Handle errors in the input!
    let _err_stream = parsed_result_stream
        .filter(|res: &Result<usize, ParseIntError>| res.is_err())
        .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap_err());
    // listeners.push(err_stream.listen(|err: &ParseIntError| println!("invalid input: {}", err)));

    let index_stream = parsed_result_stream
        .filter(|res: &Result<usize, ParseIntError>| res.is_ok())
        .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap());

    let valid_index_stream = index_stream
        .filter(|index: &usize| (1..=9).contains(index))
        .map(|index: &usize| index - 1);
    // listeners.push(
    //     index_stream
    //         .filter(|index: &usize| !(0..9).contains(index))
    //         .listen(|index: &usize| println!("invalid index: {}! try again", index)),
    // );

    let board_cell = board_cell.clone();
    valid_index_stream.filter(move |index: &usize| {
        let board = board_cell.sample();
        board.is_valid_move(*index)
    })
}

fn mark_swapping(ctx: &SodiumCtx, index_stream: &Stream<usize>) -> Cell<Mark> {
    ctx.transaction(|| {
        let turn_cell_loop: CellLoop<Mark> = ctx.new_cell_loop();

        let turn_cell_fwd = turn_cell_loop.cell();
        let turn_cell = index_stream
            .snapshot(&turn_cell_fwd, |_index: &usize, turn: &Mark| turn.swap())
            .hold(Mark::X);

        turn_cell_loop.loop_(&turn_cell);
        turn_cell
    })
}

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
