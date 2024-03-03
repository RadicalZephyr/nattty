#![allow(dead_code)]

use std::{fmt, io::BufRead, num::ParseIntError};

use sodium::{SodiumCtx, StreamSink};

#[derive(Copy, Clone, Debug)]
enum Mark {
    X,
    O,
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
    let mut listeners = Vec::new();
    let ctx = SodiumCtx::new();

    let kb_input: StreamSink<String> = ctx.new_stream_sink();

    let parsed_result_stream = &kb_input.stream().map(|line: &String| line.parse::<usize>());

    // Handle errors in the input!
    let err_stream = parsed_result_stream
        .filter(|res: &Result<usize, ParseIntError>| res.is_err())
        .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap_err());
    listeners.push(err_stream.listen(|err: &ParseIntError| println!("invalid input: {}", err)));

    let ok_stream = parsed_result_stream
        .filter(|res: &Result<usize, ParseIntError>| res.is_ok())
        .map(|res: &Result<usize, ParseIntError>| res.clone().unwrap());

    listeners.push(ok_stream.listen(|index: &usize| println!("make a move at index {}", index)));

    let stdin = std::io::stdin().lock();
    for line in stdin.lines() {
        kb_input.send(line.unwrap());
    }
}
