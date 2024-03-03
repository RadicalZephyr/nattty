#![allow(dead_code)]

use std::{fmt, io::BufRead, num::ParseIntError};

use sodium::{Cell, CellLoop, SodiumCtx, Stream, StreamSink};

const WIN_SEQUENCES: [[usize; 3]; 8] = [
    // Horizontal
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    // Vertical
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    // Diagonal
    [0, 4, 8],
    [2, 4, 6],
];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Mark {
    X,
    O,
}
impl Mark {
    fn swap(&self) -> Mark {
        match self {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Board {
    squares: [Option<Mark>; 9],
}

impl Board {
    fn new() -> Self {
        let squares = [None; 9];
        Self { squares }
    }

    fn mark(&self, index: usize, mark: Mark) -> Board {
        let mut new_board = *self;
        new_board.squares[index] = Some(mark);
        new_board
    }

    fn display_squares(&self) -> [&'static str; 9] {
        let mut display = [""; 9];
        for (dsquare, square) in display.iter_mut().zip(self.squares.iter()) {
            match square {
                Some(Mark::X) => *dsquare = "X",
                Some(Mark::O) => *dsquare = "O",
                None => *dsquare = " ",
            }
        }
        display
    }

    fn is_valid_move(&self, index: usize) -> bool {
        self.squares[index].is_none()
    }

    fn get_winner(&self) -> Option<Mark> {
        for seq in WIN_SEQUENCES {
            let first = self.squares[seq[0]];
            if first.is_some() && seq.iter().map(|i| self.squares[*i]).all(|x| x == first) {
                return first;
            }
        }
        None
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ds = self.display_squares();
        writeln!(f, " {} | {} | {}", ds[6], ds[7], ds[8])?;
        f.write_str("---+---+---\n")?;
        writeln!(f, " {} | {} | {}", ds[3], ds[4], ds[5])?;
        f.write_str("---+---+---\n")?;
        writeln!(f, " {} | {} | {}", ds[0], ds[1], ds[2])?;
        Ok(())
    }
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
