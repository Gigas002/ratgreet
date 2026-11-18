use std::error::Error;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph},
};

use libratgreet::Greeter;

use crate::ui::{
    Frame,
    common::style::{Theme, Themed},
    strings, themed_text,
    util::*,
};

pub fn draw(
    greeter: &mut Greeter,
    theme: &Theme,
    f: &mut Frame,
) -> Result<(u16, u16), Box<dyn Error>> {
    let size = f.area();

    // Centered and clamped to the terminal, like the other screens.
    let (x, y, width, height) = get_rect_bounds(greeter, size, 1);

    let container = Rect::new(x, y, width, height);
    let container_padding = greeter.container_padding();
    let frame = Rect::new(
        x + container_padding,
        y + container_padding,
        width.saturating_sub(2 * container_padding),
        height.saturating_sub(2 * container_padding),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .style(theme.of(&[Themed::Container]))
        .border_style(theme.of(&[Themed::Border]));

    let constraints = [Constraint::Length(1)];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_ref())
        .split(frame);
    let text = themed_text(theme, strings::get("wait"));
    let paragraph = Paragraph::new(text).alignment(Alignment::Center);

    f.render_widget(paragraph, chunks[0]);
    f.render_widget(block, container);

    Ok((1, 1))
}
