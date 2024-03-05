use std::fmt;

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
pub enum Mark {
    X,
    O,
}

impl Mark {
    pub fn swap(&self) -> Mark {
        match self {
            Mark::X => Mark::O,
            Mark::O => Mark::X,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Board {
    squares: [Option<Mark>; 9],
}

impl Board {
    pub fn new() -> Self {
        let squares = [None; 9];
        Self { squares }
    }

    pub fn mark(&self, index: usize, mark: Mark) -> Board {
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

    pub fn is_valid_move(&self, index: usize) -> bool {
        self.squares[index].is_none()
    }

    pub fn get_winner(&self) -> Option<Mark> {
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
