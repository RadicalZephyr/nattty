#![allow(dead_code)]

use std::fmt;

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
    let board = Board::new();
    println!("{}", board);
    let board = board.mark(2, Mark::X);
    println!("{}", board);
    let board = board.mark(4, Mark::X);
    println!("{}", board);
    let board = board.mark(6, Mark::X);
    println!("{}", board);
}
