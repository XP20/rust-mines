use std::io::Result;

mod game;
mod render;

fn main() -> Result<()> {
    let (width, height) = (12, 12);
    let mut renderer = render::Screen::new(60.0)?;
    let mut game = game::Game::new(width, height);

    // TODO: Game start popup with manual size / max window size
    // TODO: Game won / lost popup with restart / exit game

    'game: loop {
        renderer.render_ui(&game).unwrap();

        for signal in renderer.handle_events(&game)? {
            match signal.signal_type {
                render::SignalType::Quit => break 'game,
                render::SignalType::Move => match signal.pos {
                    Some(pos) => game.set_selected(pos),
                    None => (),
                },
                render::SignalType::Click => game.click_tile(),
                render::SignalType::Mark => game.toggle_mark(),
            }
        }
    }

    renderer.cleanup()?;
    Ok(())
}
