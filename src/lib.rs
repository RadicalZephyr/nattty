use std::num::ParseIntError;

use sodium::{Cell, CellLoop, SodiumCtx, Stream};
use thiserror::Error;

mod board;
pub use board::{Board, Mark};

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("invalid move: square {0} is already taken!")]
    InvalidMove(usize),

    #[error("invalid index: {0}!")]
    InvalidIndex(usize),

    #[error("invalid input: {0}!")]
    InvalidInteger(ParseIntError),
}

pub struct Players {}

#[derive(Copy, Clone, Debug)]
pub enum AppState {
    RegisterPlayer,
    Playing,
}

pub struct SequenceOfGames {
    pub app_state: Cell<AppState>,
    pub prompt_player_name: Stream<()>,
    pub start_game: Stream<()>,
}

pub struct TicTacToe {
    pub board: Cell<Board>,
    pub turn: Cell<Mark>,
    pub moves: Stream<(usize, Mark)>,
    pub winner: Stream<Mark>,
    pub error: Stream<Error>,
}

struct IndexValidator {
    valid_move_stream: Stream<usize>,
    error_stream: Stream<Error>,
}

impl AppState {
    pub fn is_playing(&self) -> bool {
        matches!(self, AppState::Playing)
    }
}

impl SequenceOfGames {
    pub fn new(
        ctx: &SodiumCtx,
        new_matchup: &Stream<()>,
        kb_input: &Stream<String>,
    ) -> SequenceOfGames {
        let start_game = ctx.new_stream();

        let app_state_cell = start_game
            .map(|_: &()| AppState::Playing)
            .hold(AppState::RegisterPlayer);
        let not_playing_cell = app_state_cell.map(|app_state: &AppState| !app_state.is_playing());

        let input = kb_input.gate(&not_playing_cell);

        let prompt_player_name = new_matchup.gate(&not_playing_cell);

        let start_game = ctx.new_stream();
        SequenceOfGames {
            app_state: app_state_cell,
            prompt_player_name,
            start_game,
        }
    }
}

impl TicTacToe {
    pub fn new(ctx: &SodiumCtx, kb_input: &Stream<String>) -> TicTacToe {
        let board_cell_loop: CellLoop<Board> = ctx.new_cell_loop();
        let board_cell_fwd = board_cell_loop.cell();

        let IndexValidator {
            valid_move_stream,
            error_stream,
        } = IndexValidator::new(kb_input, &board_cell_fwd);

        // Alternate marks
        let turn_cell = mark_swapping(ctx, &valid_move_stream);

        let index_mark_stream =
            valid_move_stream.snapshot(&turn_cell, |index: &usize, turn: &Mark| (*index, *turn));

        let board_stream = &valid_move_stream.snapshot3(
            &board_cell_fwd,
            &turn_cell,
            |index: &usize, board: &Board, mark: &Mark| board.mark(*index, *mark),
        );
        let board_cell = board_stream.hold(Board::new());
        board_cell_loop.loop_(&board_cell);

        let winner_stream = board_stream
            .map(|board: &Board| board.get_winner())
            .filter_option();

        TicTacToe {
            board: board_cell,
            turn: turn_cell,
            moves: index_mark_stream,
            winner: winner_stream,
            error: error_stream,
        }
    }
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

        let error_stream = parse_int_err_stream
            .map(|e: &ParseIntError| Error::InvalidInteger(e.clone()))
            .or_else(&invalid_index_stream.map(|i: &usize| Error::InvalidIndex(*i)))
            .or_else(&invalid_move_stream.map(|i: &usize| Error::InvalidMove(*i)));

        IndexValidator {
            valid_move_stream,
            error_stream,
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
