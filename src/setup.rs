use std::num::ParseIntError;

use sodium::{Cell, CellLoop, SodiumCtx, Stream, StreamSink};

use super::board::{Board, Mark};

struct IndexValidator {
    valid_move_stream: Stream<usize>,
    invalid_move_stream: Stream<usize>,
    invalid_index_stream: Stream<usize>,
    parse_int_err_stream: Stream<ParseIntError>,
}

pub fn set_up_play(
    kb_input: &StreamSink<String>,
    listeners: &mut Vec<sodium::Listener>,
    ctx: &SodiumCtx,
) {
    let board_cell_loop: CellLoop<Board> = ctx.new_cell_loop();
    let board_cell_fwd = board_cell_loop.cell();

    let kb_stream = kb_input.stream();
    let IndexValidator {
        valid_move_stream,
        invalid_move_stream: _,
        invalid_index_stream: _,
        parse_int_err_stream: _,
    } = IndexValidator::new(&kb_stream, &board_cell_fwd);
    // listeners.push(parse_int_err_stream.listen(|err: &ParseIntError| println!("invalid input: {}", err)));
    // listeners.push(
    //     invalid_index_stream
    //         .listen(|index: &usize| println!("invalid index: {}! try again", index)),
    // );

    // Alternate marks
    let turn_cell = mark_swapping(ctx, &valid_move_stream);

    let index_mark_stream =
        valid_move_stream.snapshot(&turn_cell, |index: &usize, turn: &Mark| (*index, *turn));
    listeners.push(index_mark_stream.listen(|(index, mark): &(usize, Mark)| {
        println!("\nMark an {:?} at index {}:", mark, index)
    }));

    let board_stream = &valid_move_stream.snapshot3(
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

impl IndexValidator {
    fn new(input_stream: &Stream<String>, board_cell: &Cell<Board>) -> IndexValidator {
        let (index_stream, parse_int_err_stream) = input_stream
            .map(|line: &String| line.parse::<usize>())
            .split_res();

        let valid_index_stream = index_stream
            .filter(|index: &usize| (1..=9).contains(index))
            .map(|index: &usize| index - 1);
        let invalid_index_stream = index_stream.filter(|index: &usize| !(1..=9).contains(index));

        let board = board_cell.clone();
        let valid_move_stream = valid_index_stream.filter(move |index: &usize| {
            let board = board.sample();
            board.is_valid_move(*index)
        });
        let board = board_cell.clone();
        let invalid_move_stream = valid_index_stream.filter(move |index: &usize| {
            let board = board.sample();
            !board.is_valid_move(*index)
        });

        IndexValidator {
            valid_move_stream,
            invalid_move_stream,
            invalid_index_stream,
            parse_int_err_stream,
        }
    }
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
