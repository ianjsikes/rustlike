use constants::*;
use tcod::colors::{self, Color};
use tcod::console::*;

pub type Messages = Vec<(String, Color)>;

// This is really annoying to pass around everywhere. Consider using lazy_static
pub fn message<T: Into<String>>(messages: &mut Messages, message: T, color: Color) {
  if messages.len() == MSG_HEIGHT {
    messages.remove(0);
  }

  messages.push((message.into(), color));
}

pub fn render_messages(messages: &Messages, panel: &mut Offscreen) {
  let mut y = MSG_HEIGHT as i32;
  for &(ref msg, color) in messages.iter().rev() {
    let msg_height = panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    y -= msg_height;
    if y < 0 {
      break;
    }
    panel.set_default_foreground(color);
    panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
  }
}

pub fn render_bar(
  panel: &mut Offscreen,
  x: i32,
  y: i32,
  total_width: i32,
  name: &str,
  value: i32,
  maximum: i32,
  bar_color: Color,
  back_color: Color,
) {
  let bar_width = (value as f32 / maximum as f32 * total_width as f32) as i32;

  panel.set_default_background(back_color);
  panel.rect(x, y, total_width, 1, false, BackgroundFlag::Screen);

  panel.set_default_background(bar_color);
  if bar_width > 0 {
    panel.rect(x, y, bar_width, 1, false, BackgroundFlag::Screen);
  }

  panel.set_default_foreground(colors::WHITE);
  panel.print_ex(
    x + total_width / 2,
    y,
    BackgroundFlag::None,
    TextAlignment::Center,
    &format!("{}: {}/{}", name, value, maximum),
  );
}
