use constants::*;
use object::*;
use tcod::colors::{self, Color};
use tcod::console::*;

pub trait MessageLog {
  fn add<T: Into<String>>(&mut self, message: T, color: Color);
}

pub type Messages = Vec<(String, Color)>;

impl MessageLog for Vec<(String, Color)> {
  fn add<T: Into<String>>(&mut self, message: T, color: Color) {
    self.push((message.into(), color));
  }
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

pub fn menu<T: AsRef<str>>(
  header: &str,
  options: &[T],
  width: i32,
  root: &mut Root,
) -> Option<usize> {
  assert!(
    options.len() <= 26,
    "Cannot have a menu with more than 26 options."
  );

  let header_height = if header.is_empty() {
    0
  } else {
    root.get_height_rect(0, 0, width, SCREEN_HEIGHT, header)
  };
  let height = options.len() as i32 + header_height;

  let mut window = Offscreen::new(width, height);

  window.set_default_foreground(colors::WHITE);
  window.print_rect_ex(
    0,
    0,
    width,
    height,
    BackgroundFlag::None,
    TextAlignment::Left,
    header,
  );

  for (index, option_text) in options.iter().enumerate() {
    let menu_letter = (b'a' + index as u8) as char;
    let text = format!("({}) {}", menu_letter, option_text.as_ref());
    window.print_ex(
      0,
      header_height + index as i32,
      BackgroundFlag::None,
      TextAlignment::Left,
      text,
    );
  }

  let x = SCREEN_WIDTH / 2 - width / 2;
  let y = SCREEN_HEIGHT / 2 - height / 2;
  blit(&mut window, (0, 0), (width, height), root, (x, y), 1.0, 0.7);

  root.flush();
  let key = root.wait_for_keypress(true);

  if key.printable.is_alphabetic() {
    let index = key.printable.to_ascii_lowercase() as usize - 'a' as usize;
    if index < options.len() {
      Some(index)
    } else {
      None
    }
  } else {
    None
  }
}

pub fn inventory_menu(inventory: &[Object], header: &str, root: &mut Root) -> Option<usize> {
  let options = if inventory.len() == 0 {
    vec!["Inventory is empty.".into()]
  } else {
    inventory.iter().map(|item| item.name.clone()).collect()
  };

  let inventory_index = menu(header, &options, INVENTORY_WIDTH, root);

  if inventory.len() > 0 {
    inventory_index
  } else {
    None
  }
}

pub fn drop_item(inventory_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
  let mut item = game.inventory.remove(inventory_id);
  item.set_pos(objects[PLAYER].x, objects[PLAYER].y);
  game
    .log
    .add(format!("You dropped a {}.", item.name), colors::YELLOW);
  objects.push(item);
}
