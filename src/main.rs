#![allow(dead_code)]

use std::{fmt, io::BufRead, num::ParseIntError};

use sodium::{CellLoop, SodiumCtx, StreamSink};

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

        let parsed_result_stream = &kb_input.stream().map(|line: &String| line.parse::<usize>());

        // Handle errors in the input!
        let err_stream = parsed_result_stream
            .filter(|res: &Result<usize, ParseIntError>| res.is_err())
            .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap_err());
        listeners.push(err_stream.listen(|err: &ParseIntError| println!("invalid input: {}", err)));

        let index_stream = parsed_result_stream
            .filter(|res: &Result<usize, ParseIntError>| res.is_ok())
            .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap());

        let valid_index_stream = index_stream.filter(|index: &usize| (0..=9).contains(index));
        listeners.push(
            index_stream
                .filter(|index: &usize| !(0..=9).contains(index))
                .listen(|index: &usize| println!("invalid index: {}! try again", index)),
        );

        // Alternate marks
        let turn_cell = mark_swapping(&ctx, &valid_index_stream);

        let index_mark_stream =
            valid_index_stream.snapshot(&turn_cell, |index: &usize, turn: &Mark| (*index, *turn));
        listeners.push(index_mark_stream.listen(|(index, mark): &(usize, Mark)| {
            println!("Mark an {:?} at index {}", mark, index)
        }));

        (kb_input, listeners)
    });

    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        kb_input.send(line.unwrap());
    }
}

fn mark_swapping(ctx: &SodiumCtx, index_stream: &sodium::Stream<usize>) -> sodium::Cell<Mark> {
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
