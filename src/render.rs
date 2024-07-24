use std::io::{stdout, Result, Stdout};
use ratatui::{crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEventKind, MouseButton, MouseEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}}, layout::Rect, prelude::CrosstermBackend, style::Stylize, widgets::{block::Title, Block, Paragraph}, CompletedFrame, Frame, Terminal};

use crate::game::Game;

pub enum SignalType {
    Quit,
    Click,
    Mark,
    Move,
}

pub struct Signal {
    pub pos: Option<(i32, i32)>,
    pub signal_type: SignalType,
}

pub struct Screen {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    interrupt: u64,
}

impl Screen {
    pub fn new(fps: f32) -> Result<Screen> {
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        enable_raw_mode()?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;
        
        Ok(Screen {
            terminal: terminal,
            interrupt: (1000.0 / fps).round() as u64,
        })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn draw<F>(&mut self, f: F) -> Result<CompletedFrame>
    where F: FnOnce(&mut Frame) {
        self.terminal.draw(f)
    }

    pub fn handle_events(&mut self, game: &Game) -> Result<Vec<Signal>> {
        let mut signals: Vec<Signal> = Vec::new();

        if event::poll(std::time::Duration::from_millis(self.interrupt))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press { match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => signals.push(Signal {
                        pos: None,
                        signal_type: SignalType::Quit,
                    }),
                    KeyCode::Left | KeyCode::Char('h') => signals.push(Signal {
                        pos: Some((game.selected.0 as i32 - 1, game.selected.1 as i32)),
                        signal_type: SignalType::Move,
                    }),
                    KeyCode::Right | KeyCode::Char('l') => signals.push(Signal {
                        pos: Some((game.selected.0 as i32 + 1, game.selected.1 as i32)),
                        signal_type: SignalType::Move,
                    }),
                    KeyCode::Up | KeyCode::Char('k') => signals.push(Signal {
                        pos: Some((game.selected.0 as i32, game.selected.1 as i32 - 1)),
                        signal_type: SignalType::Move,
                    }),
                    KeyCode::Down | KeyCode::Char('j') => signals.push(Signal {
                        pos: Some((game.selected.0 as i32, game.selected.1 as i32 + 1)),
                        signal_type: SignalType::Move,
                    }),
                    KeyCode::Char('f') => signals.push(Signal {
                        pos: None,
                        signal_type: SignalType::Mark,
                    }),
                    KeyCode::Char(' ') => signals.push(Signal {
                        pos: None,
                        signal_type: SignalType::Click,
                    }),
                    _ => (),
                } }
            } else if let event::Event::Mouse(mouse) = event::read()? {
                signals.push(Signal {
                    pos: Some((mouse.column as i32 / 3, mouse.row as i32 - 1)),
                    signal_type: SignalType::Move,
                });

                match mouse.kind {
                    MouseEventKind::Up(MouseButton::Left) => signals.push(Signal {
                        pos: None,
                        signal_type: SignalType::Click,
                    }),
                    MouseEventKind::Up(MouseButton::Right) => signals.push(Signal {
                        pos: None,
                        signal_type: SignalType::Mark,
                    }),
                    _ => (),
                }
            }
        }

        Ok(signals)
    }

    pub fn render_ui(&mut self, game: &Game) -> Result<()> {
        let tiles = &game.tiles;
        let (mut widest, mut highest) = (0, 0);
        tiles.iter().for_each(|x| widest = x.x.max(widest));
        tiles.iter().for_each(|x| highest = x.y.max(highest));

        let mut tile_widgets = Vec::new();
        for tile in tiles {
            let widget = match tile.tile_visibility {
                crate::game::TileVisibility::Hidden => Paragraph::new(" □ ")
                    .black()
                    .on_dark_gray(),
                crate::game::TileVisibility::Marked => Paragraph::new(" ■ ")
                    .black()
                    .on_dark_gray(),
                crate::game::TileVisibility::Visible => match tile.tile_type {
                    crate::game::TileType::Mine => Paragraph::new(" ☓ ")
                        .dark_gray()
                        .on_black(),
                    crate::game::TileType::Safe => Paragraph::new(format!(" {} ",
                        if tile.mine_count == 0 { "· ".to_string() }
                        else { tile.mine_count.to_string() }))
                        .dark_gray()
                        .on_black(),
                },
            };
            tile_widgets.push((
                (tile.x * 3) as u16,
                (tile.y) as u16 + 1,
                if game.selected == (tile.x, tile.y) { widget.on_yellow() } else { widget },
            ));
        }
        
        self.draw(|frame| {
            // Title bar
            frame.render_widget(
                Block::new()
                    .title(Title::from(" rust-mines".black().on_white()))
                    .title(Title::from(" X ".black().on_red().bold())
                        .alignment(ratatui::layout::Alignment::Right))
                    .black()
                    .on_white(),
                Rect::new(
                    0,
                    0,
                    widest as u16 * 3 + 3,
                    1,
                )
            );

            // Tiles
            for (x, y, widget) in tile_widgets {
                frame.render_widget(
                    widget,
                    Rect::new(x, y, 3, 1)
                );
            }
        })?;

        Ok(())
    }
}
