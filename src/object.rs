use constants::*;
use gui::*;
use tcod::colors::{self, Color};
use tcod::console::*;
use tcod::input::Mouse;
use tcod::map::Map as FovMap;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ai;

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
  tcod: &mut Tcod,
) {
  use Item::*;
  if let Some(item) = inventory[inventory_id].item {
    let on_use = match item {
      Heal => cast_heal,
      Lightning => cast_lightning,
    };
    match on_use(inventory_id, objects, messages, tcod) {
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
