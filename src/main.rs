#![allow(dead_code)]

use std::{fmt, io::BufRead, num::ParseIntError};

use sodium::{Cell, CellLoop, SodiumCtx, Stream, StreamSink};

#[derive(Copy, Clone, Debug)]
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
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ds = self.display_squares();
        writeln!(f, " {} | {} | {}", ds[0], ds[1], ds[2])?;
        f.write_str("---+---+---\n")?;
        writeln!(f, " {} | {} | {}", ds[3], ds[4], ds[5])?;
        f.write_str("---+---+---\n")?;
        writeln!(f, " {} | {} | {}", ds[6], ds[7], ds[8])?;
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
    let parsed_result_stream = &kb_input.stream().map(|line: &String| line.parse::<usize>());

    // Handle errors in the input!
    let err_stream = parsed_result_stream
        .filter(|res: &Result<usize, ParseIntError>| res.is_err())
        .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap_err());
    listeners.push(err_stream.listen(|err: &ParseIntError| println!("invalid input: {}", err)));

    let index_stream = parsed_result_stream
        .filter(|res: &Result<usize, ParseIntError>| res.is_ok())
        .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap());

    let valid_index_stream = index_stream.filter(|index: &usize| (0..9).contains(index));
    listeners.push(
        index_stream
            .filter(|index: &usize| !(0..9).contains(index))
            .listen(|index: &usize| println!("invalid index: {}! try again", index)),
    );

    // Alternate marks
    let turn_cell = mark_swapping(ctx, &valid_index_stream);

    let index_mark_stream =
        valid_index_stream.snapshot(&turn_cell, |index: &usize, turn: &Mark| (*index, *turn));
    listeners.push(index_mark_stream.listen(|(index, mark): &(usize, Mark)| {
        println!("\nMark an {:?} at index {}:", mark, index)
    }));

    let board_cell = update_board(ctx, &valid_index_stream, &turn_cell);
    listeners.push(
        board_cell
            .updates()
            .listen(|board: &Board| println!("{}", board)),
    );
}

fn update_board(
    ctx: &SodiumCtx,
    index_stream: &Stream<usize>,
    mark_cell: &Cell<Mark>,
) -> Cell<Board> {
    ctx.transaction(|| {
        let board_cell_loop: CellLoop<Board> = ctx.new_cell_loop();
        let board_cell_fwd = board_cell_loop.cell();

        let board_cell = index_stream
            .snapshot3(
                &board_cell_fwd,
                mark_cell,
                |index: &usize, board: &Board, mark: &Mark| board.mark(*index, *mark),
            )
            .hold(Board::new());

        board_cell_loop.loop_(&board_cell);
        board_cell
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
