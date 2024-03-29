use std::{io, thread, time::Duration};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use nattty::{Board, Mark, TicTacToe};
use sodium as na;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Widget},
    Frame, Terminal,
};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let ctx = na::SodiumCtx::new();

    let (start_game, kb_input, game) = ctx.transaction(|| {
        let start_game: na::StreamSink<()> = ctx.new_stream_sink();
        let kb_input: na::StreamSink<String> = ctx.new_stream_sink();

        let game = TicTacToe::new(&ctx, &kb_input.stream());

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
    let playing_cell_sink = ctx.new_cell_sink(true);
    let playing_cell = playing_cell_sink.cell();

    thread::spawn(move || dispatch_events(kb_input, playing_cell_sink));

    while playing_cell.sample() {
        terminal.draw(|f| ui.draw(f))?;
        thread::sleep(Duration::from_millis(30));
    }

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

fn dispatch_events(
    kb_input: na::StreamSink<String>,
    playing_cell_sink: na::CellSink<bool>,
) -> io::Result<()> {
    loop {
        match event::read()? {
            event::Event::Key(ev) => match ev.code {
                event::KeyCode::Esc => playing_cell_sink.send(false),
                event::KeyCode::Char(c) if ('1'..='9').contains(&c) => {
                    kb_input.send(String::from(c));
                }
                _ => (),
            },
            event::Event::FocusGained => (),
            event::Event::FocusLost => (),
            event::Event::Mouse(_) => (),
            event::Event::Paste(_) => (),
            event::Event::Resize(_, _) => (),
        }
    }
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
            Constraint::Percentage(5),
            Constraint::Percentage(29),
            Constraint::Percentage(34),
            Constraint::Percentage(30),
            Constraint::Percentage(5),
        ];
        let vchunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(thirds.as_ref())
            .split(area);

        let hchunks0 = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .horizontal_margin(3)
            .constraints(thirds.as_ref())
            .split(vchunks[1]);
        let hchunks1 = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .horizontal_margin(3)
            .constraints(thirds.as_ref())
            .split(vchunks[2]);
        let hchunks2 = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .horizontal_margin(3)
            .constraints(thirds.as_ref())
            .split(vchunks[3]);

        let board = self.board.sample();
        let squares = [
            (
                board.squares[6],
                hchunks0[1],
                Borders::RIGHT | Borders::BOTTOM,
            ),
            (board.squares[7], hchunks0[2], Borders::ALL ^ Borders::TOP),
            (
                board.squares[8],
                hchunks0[3],
                Borders::LEFT | Borders::BOTTOM,
            ),
            (board.squares[3], hchunks1[1], Borders::ALL ^ Borders::LEFT),
            (board.squares[4], hchunks1[2], Borders::ALL),
            (board.squares[5], hchunks1[3], Borders::ALL ^ Borders::RIGHT),
            (board.squares[0], hchunks2[1], Borders::RIGHT | Borders::TOP),
            (
                board.squares[1],
                hchunks2[2],
                Borders::ALL ^ Borders::BOTTOM,
            ),
            (board.squares[2], hchunks2[3], Borders::LEFT | Borders::TOP),
        ];

        for (mark, chunk, borders) in squares {
            let block = Block::default().borders(borders);
            f.render_widget(RenderMark(mark), block.inner(chunk));
            f.render_widget(block, chunk);
        }
    }
}

struct RenderMark(Option<Mark>);

impl Widget for RenderMark {
    fn render(self, area: Rect, buf: &mut tui::buffer::Buffer) {
        if let Some(mark) = self.0 {
            match mark {
                Mark::X => render_x(area, buf),
                Mark::O => render_o(area, buf),
            }
        }
    }
}

fn render_x(area: Rect, buf: &mut tui::buffer::Buffer) {
    const LEFT: &str = "\\@\\";
    const RIGHT: &str = "/@/";

    // In theory this should be the number of characters to go over for each line
    // -2 because the line is 3 characters wide
    let inv_slope = (area.width).div_euclid(area.height);
    for y in 0..area.height {
        let x_left = area.width - 5 - (y * inv_slope);
        buf.set_string(area.x + x_left, area.y + y, RIGHT, Style::default());

        let x_right = y * inv_slope;
        buf.set_string(area.x + x_right, area.y + y, LEFT, Style::default());
    }
}

fn render_o(area: Rect, buf: &mut tui::buffer::Buffer) {
    let x_offset;
    let y_offset;
    if area.width >= area.height {
        x_offset = (area.width - area.height).div_euclid(2);
        y_offset = 0;
    } else {
        x_offset = 0;
        y_offset = (area.height - area.width).div_euclid(2);
    }

    let diameter = area.width.min(area.height);
    let radius = diameter.div_euclid(2);
    let center_x = (area.x + radius + x_offset) as i32;
    let center_y = (area.y + radius + y_offset) as i32;

    let radius = radius as f32;
    let min_angle = (1.0 - 1.0 / radius).acos().to_radians();

    let mut angle = 0.0;

    while angle < std::f32::consts::PI {
        let (sin, cos) = angle.sin_cos();
        let x_offset: i32 = unsafe { (radius * cos).to_int_unchecked() };
        let y_offset: i32 = unsafe { (radius * sin).to_int_unchecked() };

        let x = center_x + x_offset;
        let y = center_y + y_offset;
        buf.get_mut(x as u16, y as u16).set_char('@');
        let y = center_y - y_offset;
        buf.get_mut(x as u16, y as u16).set_char('@');
        angle += min_angle;
    }
}
