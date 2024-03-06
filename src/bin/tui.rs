use std::{io, thread, time::Duration};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nattty::{Board, Mark, TicTacToe};
use sodium as na;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols::block,
    widgets::{Block, Borders, Row, Table},
    Frame, Terminal,
};

fn main() -> io::Result<()> {
    let ctx = na::SodiumCtx::new();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (start_game, kb_input, game) = ctx.transaction(|| {
        let start_game: na::StreamSink<()> = ctx.new_stream_sink();
        let kb_input: na::StreamSink<String> = ctx.new_stream_sink();

        let game = TicTacToe::new(&kb_input, &ctx);

        (start_game, kb_input, game)
    });

    let mut listeners: Vec<na::Listener> = Vec::new();
    let TicTacToe {
        board,
        turn,
        moves,
        winner,
        error,
    } = game;
    let ui = Ui { turn, board };

    terminal.draw(move |f| ui.draw(f))?;

    thread::sleep(Duration::from_millis(5000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

struct Ui {
    turn: na::Cell<Mark>,
    board: na::Cell<Board>,
}

impl Ui {
    fn draw<B: Backend>(&self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(2)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(f.size());
        let block = Block::default().title("Board").borders(Borders::ALL);
        self.draw_board(f, block.inner(chunks[0]));
        f.render_widget(block, chunks[0]);

        let block = Block::default().title("Game Info").borders(Borders::ALL);
        f.render_widget(block, chunks[1]);
    }

    fn draw_board<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let thirds = [
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ];
        let vchunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(thirds.as_ref())
            .split(area);
        let hchunks0 = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(thirds.as_ref())
            .split(vchunks[0]);
        let block = Block::default().borders(Borders::RIGHT | Borders::BOTTOM);
        f.render_widget(block, hchunks0[0]);
        let block = Block::default().borders(Borders::ALL ^ Borders::TOP);
        f.render_widget(block, hchunks0[1]);
        let block = Block::default().borders(Borders::LEFT | Borders::BOTTOM);
        f.render_widget(block, hchunks0[2]);

        let hchunks1 = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(thirds.as_ref())
            .split(vchunks[1]);
        let block = Block::default().borders(Borders::ALL ^ Borders::LEFT);
        f.render_widget(block, hchunks1[0]);
        let block = Block::default().borders(Borders::ALL);
        f.render_widget(block, hchunks1[1]);
        let block = Block::default().borders(Borders::ALL ^ Borders::RIGHT);
        f.render_widget(block, hchunks1[2]);

        let hchunks2 = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints(thirds.as_ref())
            .split(vchunks[2]);
        let block = Block::default().borders(Borders::RIGHT | Borders::TOP);
        f.render_widget(block, hchunks2[0]);
        let block = Block::default().borders(Borders::ALL ^ Borders::BOTTOM);
        f.render_widget(block, hchunks2[1]);
        let block = Block::default().borders(Borders::LEFT | Borders::TOP);
        f.render_widget(block, hchunks2[2]);
    }
}
