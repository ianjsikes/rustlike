use constants::*;
use gui::*;
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::Mouse;
use tcod::map::Map as FovMap;

#[derive(Clone, Copy, Debug)]
pub struct Tile {
  pub blocked: bool,
  pub block_sight: bool,
  pub explored: bool,
}

impl Tile {
  pub fn empty() -> Self {
    Tile {
      blocked: false,
      block_sight: false,
      explored: false,
    }
  }

  pub fn wall() -> Self {
    Tile {
      blocked: true,
      block_sight: true,
      explored: false,
    }
  }
}

pub type Map = Vec<Vec<Tile>>;

pub struct Tcod {
  pub root: Root,
  pub con: Offscreen,
  pub panel: Offscreen,
  pub fov: FovMap,
  pub mouse: Mouse,
}

// combat-related properties and methods (monster, player, NPC)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fighter {
  pub max_hp: i32,
  pub hp: i32,
  pub defense: i32,
  pub power: i32,
  pub on_death: DeathCallback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Item {
  Heal,
  Lightning,
  Confuse,
  Fireball,
}

enum UseResult {
  UsedUp,
  Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathCallback {
  Player,
  Monster,
}

impl DeathCallback {
  fn callback(self, object: &mut Object, messages: &mut Messages) {
    use self::DeathCallback::*;
    let callback: fn(&mut Object, &mut Messages) = match self {
      Player => player_death,
      Monster => monster_death,
    };
    callback(object, messages);
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ai {
  Basic,
  Confused {
    previous_ai: Box<Ai>,
    num_turns: i32,
  },
}

#[derive(Debug)]
pub struct Object {
  pub x: i32,
  pub y: i32,
  pub name: String,
  pub blocks: bool,
  pub alive: bool,
  pub char: char,
  pub color: Color,
  pub fighter: Option<Fighter>,
  pub ai: Option<Ai>,
  pub item: Option<Item>,
}

impl Object {
  pub fn new(x: i32, y: i32, char: char, name: &str, color: Color, blocks: bool) -> Self {
    Object {
      x: x,
      y: y,
      char: char,
      color: color,
      name: name.into(),
      blocks: blocks,
      alive: false,
      fighter: None,
      ai: None,
      item: None,
    }
  }

  pub fn draw(&self, con: &mut Console) {
    con.set_default_foreground(self.color);
    con.put_char(self.x, self.y, self.char, BackgroundFlag::None);
  }

  pub fn clear(&self, con: &mut Console) {
    con.put_char(self.x, self.y, ' ', BackgroundFlag::None);
  }

  pub fn pos(&self) -> (i32, i32) {
    (self.x, self.y)
  }

  pub fn set_pos(&mut self, x: i32, y: i32) {
    self.x = x;
    self.y = y;
  }

  pub fn distance_to(&self, other: &Object) -> f32 {
    let dx = other.x - self.x;
    let dy = other.y - self.y;
    ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
  }

  pub fn take_damage(&mut self, damage: i32, messages: &mut Messages) {
    if let Some(fighter) = self.fighter.as_mut() {
      if damage > 0 {
        fighter.hp -= damage;
      }
    }

    if let Some(fighter) = self.fighter {
      if fighter.hp <= 0 {
        self.alive = false;
        fighter.on_death.callback(self, messages);
      }
    }
  }

  pub fn attack(&mut self, target: &mut Object, messages: &mut Messages) {
    let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
    if damage > 0 {
      message(
        messages,
        format!(
          "{} attacks {} for {} hit points.",
          self.name, target.name, damage
        ),
        colors::DESATURATED_FUCHSIA,
      );
      target.take_damage(damage, messages);
    } else {
      message(
        messages,
        format!(
          "{} attacks {} but it has no effect!",
          self.name, target.name
        ),
        colors::DESATURATED_FUCHSIA,
      );
    }
  }

  pub fn heal(&mut self, amount: i32) {
    if let Some(ref mut fighter) = self.fighter {
      fighter.hp += amount;
      if fighter.hp > fighter.max_hp {
        fighter.hp = fighter.max_hp;
      }
    }
  }

  pub fn distance(&self, x: i32, y: i32) -> f32 {
    (((x - self.x).pow(2) + (y - self.y).pow(2)) as f32).sqrt()
  }
}

pub fn pick_item_up(
  object_id: usize,
  objects: &mut Vec<Object>,
  inventory: &mut Vec<Object>,
  messages: &mut Messages,
) {
  if inventory.len() >= 26 {
    message(
      messages,
      format!(
        "Your inventory is full, cannot pick up {}.",
        objects[object_id].name
      ),
      colors::RED,
    );
  } else {
    let item = objects.swap_remove(object_id);
    message(
      messages,
      format!("You picked up a {}!", item.name),
      colors::GREEN,
    );
    inventory.push(item);
  }
}

fn player_death(player: &mut Object, messages: &mut Messages) {
  message(messages, "You died!", colors::DARK_RED);

  player.char = '%';
  player.color = colors::DARK_RED;
}

fn monster_death(monster: &mut Object, messages: &mut Messages) {
  message(messages, format!("{} is dead!", monster.name), colors::RED);
  monster.char = '%';
  monster.color = colors::DARK_RED;
  monster.blocks = false;
  monster.fighter = None;
  monster.ai = None;
  monster.name = format!("remains of {}", monster.name);
}

pub fn use_item(
  inventory_id: usize,
  inventory: &mut Vec<Object>,
  objects: &mut [Object],
  messages: &mut Messages,
  map: &mut Map,
  tcod: &mut Tcod,
) {
  use Item::*;
  if let Some(item) = inventory[inventory_id].item {
    let on_use = match item {
      Heal => cast_heal,
      Lightning => cast_lightning,
      Confuse => cast_confuse,
      Fireball => cast_fireball,
    };
    match on_use(inventory_id, objects, messages, map, tcod) {
      UseResult::UsedUp => {
        inventory.remove(inventory_id);
      }
      UseResult::Cancelled => {
        message(messages, "Cancelled", colors::WHITE);
      }
    }
  } else {
    message(
      messages,
      format!("The {} cannot be used.", inventory[inventory_id].name),
      colors::WHITE,
    );
  }
}

fn cast_heal(
  _inventory_id: usize,
  objects: &mut [Object],
  messages: &mut Messages,
  map: &mut Map,
  tcod: &mut Tcod,
) -> UseResult {
  if let Some(fighter) = objects[PLAYER].fighter {
    if fighter.hp == fighter.max_hp {
      message(messages, "You are already at full health.", colors::RED);
      return UseResult::Cancelled;
    }
    message(
      messages,
      "Your wounds start to feel better!",
      colors::LIGHT_VIOLET,
    );
    objects[PLAYER].heal(HEAL_AMOUNT);
    return UseResult::UsedUp;
  }
  UseResult::Cancelled
}

fn cast_lightning(
  _inventory_id: usize,
  objects: &mut [Object],
  messages: &mut Messages,
  map: &mut Map,
  tcod: &mut Tcod,
) -> UseResult {
  let monster_id = closest_monster(LIGHTNING_RANGE, objects, tcod);
  if let Some(monster_id) = monster_id {
    message(
      messages,
      format!(
        "A lightning bolt strikes the {} with a loud thunder! \
         The damage is {} hit points.",
        objects[monster_id].name, LIGHTNING_DAMAGE
      ),
      colors::LIGHT_BLUE,
    );
    objects[monster_id].take_damage(LIGHTNING_DAMAGE, messages);
    UseResult::UsedUp
  } else {
    message(messages, "No enemy is close enough to strike.", colors::RED);
    UseResult::Cancelled
  }
}

fn cast_confuse(
  _inventory_id: usize,
  objects: &mut [Object],
  messages: &mut Messages,
  map: &mut Map,
  tcod: &mut Tcod,
) -> UseResult {
  let monster_id = closest_monster(CONFUSE_RANGE, objects, tcod);
  if let Some(monster_id) = monster_id {
    let old_ai = objects[monster_id].ai.take().unwrap_or(Ai::Basic);
    objects[monster_id].ai = Some(Ai::Confused {
      previous_ai: Box::new(old_ai),
      num_turns: CONFUSE_NUM_TURNS,
    });
    message(
      messages,
      format!(
        "The eyes of {} look vacant, as it starts to stumble around!",
        objects[monster_id].name
      ),
      colors::LIGHT_GREEN,
    );
    UseResult::UsedUp
  } else {
    message(messages, "No enemy is close enough to strike.", colors::RED);
    UseResult::Cancelled
  }
}

fn cast_fireball(
  _inventory_id: usize,
  objects: &mut [Object],
  messages: &mut Messages,
  map: &mut Map,
  tcod: &mut Tcod,
) -> UseResult {
  message(
    messages,
    "Left-click a target tile for the fireball, or right-click to cancel.",
    colors::LIGHT_CYAN,
  );
  let (x, y) = match target_tile(tcod, objects, map, messages, None) {
    Some(tile_pos) => tile_pos,
    None => return UseResult::Cancelled,
  };
  message(
    messages,
    format!(
      "The fireball explodes, burning everything within {} tiles!",
      FIREBALL_RADIUS
    ),
    colors::ORANGE,
  );

  for obj in objects {
    if obj.distance(x, y) <= FIREBALL_RADIUS as f32 && obj.fighter.is_some() {
      message(
        messages,
        format!(
          "The {} gets burned for {} hit points.",
          obj.name, FIREBALL_DAMAGE
        ),
        colors::ORANGE,
      );
      obj.take_damage(FIREBALL_DAMAGE, messages);
    }
  }

  UseResult::UsedUp
}

fn closest_monster(max_range: i32, objects: &mut [Object], tcod: &Tcod) -> Option<usize> {
  let mut closest_enemy = None;
  let mut closest_dist = (max_range + 1) as f32;

  for (id, object) in objects.iter().enumerate() {
    if (id != PLAYER)
      && object.fighter.is_some()
      && object.ai.is_some()
      && tcod.fov.is_in_fov(object.x, object.y)
    {
      let dist = objects[PLAYER].distance_to(object);
      if dist < closest_dist {
        closest_enemy = Some(id);
        closest_dist = dist;
      }
    }
  }
  closest_enemy
}

fn target_tile(
  tcod: &mut Tcod,
  objects: &[Object],
  map: &mut Map,
  messages: &Messages,
  max_range: Option<f32>,
) -> Option<(i32, i32)> {
  use tcod::input::KeyCode::Escape;
  use tcod::input::{self, Event};
  loop {
    tcod.root.flush();
    let event = input::check_for_event(input::KEY_PRESS | input::MOUSE).map(|e| e.1);
    let mut key = None;
    match event {
      Some(Event::Mouse(m)) => tcod.mouse = m,
      Some(Event::Key(k)) => key = Some(k),
      None => {}
    }
    render_all(tcod, objects, map, messages, false);

    let (x, y) = (tcod.mouse.cx as i32, tcod.mouse.cy as i32);

    let in_fov = (x < MAP_WIDTH) && (y < MAP_HEIGHT) && tcod.fov.is_in_fov(x, y);
    let in_range = max_range.map_or(true, |range| objects[PLAYER].distance(x, y) <= range);
    if tcod.mouse.lbutton_pressed && in_fov && in_range {
      return Some((x, y));
    }

    let escape = key.map_or(false, |k| k.code == Escape);
    if tcod.mouse.rbutton_pressed || escape {
      return None;
    }
  }
}

pub fn render_all(
  tcod: &mut Tcod,
  objects: &[Object],
  map: &mut Map,
  messages: &Messages,
  fov_recompute: bool,
) {
  if fov_recompute {
    let player = &objects[PLAYER];
    tcod
      .fov
      .compute_fov(player.x, player.y, TORCH_RADIUS, FOV_LIGHT_WALLS, FOV_ALGO);

    for y in 0..MAP_HEIGHT {
      for x in 0..MAP_WIDTH {
        let visible = tcod.fov.is_in_fov(x, y);
        let wall = map[x as usize][y as usize].block_sight;
        let color = match (visible, wall) {
          (false, true) => COLOR_DARK_WALL,
          (false, false) => COLOR_DARK_GROUND,
          (true, true) => COLOR_LIGHT_WALL,
          (true, false) => COLOR_LIGHT_GROUND,
        };
        if visible {
          map[x as usize][y as usize].explored = true;
        }
        if map[x as usize][y as usize].explored {
          tcod
            .con
            .set_char_background(x, y, color, BackgroundFlag::Set);
        }
      }
    }
  }

  // Sort list of objects so non-blocking objects come first
  let mut to_draw: Vec<_> = objects
    .iter()
    .filter(|o| tcod.fov.is_in_fov(o.x, o.y))
    .collect();
  to_draw.sort_by(|o1, o2| o1.blocks.cmp(&o2.blocks));

  for object in &to_draw {
    object.draw(&mut tcod.con);
  }

  // Copy the contents of con to root
  blit(
    &mut tcod.con,
    (0, 0),
    (MAP_WIDTH, MAP_HEIGHT),
    &mut tcod.root,
    (0, 0),
    1.0,
    1.0,
  );

  tcod.panel.set_default_background(colors::BLACK);
  tcod.panel.clear();

  let hp = objects[PLAYER].fighter.map_or(0, |f| f.hp);
  let max_hp = objects[PLAYER].fighter.map_or(0, |f| f.max_hp);
  render_bar(
    &mut tcod.panel,
    1,
    1,
    BAR_WIDTH,
    "HP",
    hp,
    max_hp,
    colors::LIGHT_RED,
    colors::DARKER_RED,
  );

  tcod.panel.set_default_foreground(colors::LIGHT_GREY);
  tcod.panel.print_ex(
    1,
    0,
    BackgroundFlag::None,
    TextAlignment::Left,
    get_names_under_mouse(tcod.mouse, objects, &mut tcod.fov),
  );

  render_messages(&messages, &mut tcod.panel);

  blit(
    &mut tcod.panel,
    (0, 0),
    (SCREEN_WIDTH, PANEL_HEIGHT),
    &mut tcod.root,
    (0, PANEL_Y),
    1.0,
    1.0,
  );
}

fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
  let (x, y) = (mouse.cx as i32, mouse.cy as i32);

  let names = objects
    .iter()
    .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y))
    .map(|obj| obj.name.clone())
    .collect::<Vec<_>>();

  names.join(", ")
}
