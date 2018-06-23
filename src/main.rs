//! The main program!

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// #![allow(unused_imports)]
// #![allow(unused_mut)]
// #![allow(unused_variables)]
// #![allow(dead_code)]

extern crate dwarf_term;
pub use dwarf_term::*;

// std
use std::collections::{HashMap, HashSet};

const TILE_GRID_WIDTH: usize = 66;
const TILE_GRID_HEIGHT: usize = 50;

fn main() {
    unsafe {
        let mut term =
            DwarfTerm::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, "Dwarf Term Test").expect("WHOOPS!");
        let default_fg = rgb32!(128, 255, 20);
        let default_bg = 0;

        // Main loop
        let mut running = true;
        let mut tab_held = false;
        let mut watcher_position: (isize, isize) = (5, 5);
        while running {
            // Handle Input
            term.poll_events(|event| match event {
                Event::WindowEvent {
                    event: win_event, ..
                } => match win_event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        running = false;
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    } => match key {
                        VirtualKeyCode::Up => {
                            watcher_position.1 += 1;
                        }
                        VirtualKeyCode::Down => {
                            watcher_position.1 -= 1;
                        }
                        VirtualKeyCode::Left => {
                            watcher_position.0 -= 1;
                        }
                        VirtualKeyCode::Right => {
                            watcher_position.0 += 1;
                        }
                        VirtualKeyCode::Tab => {
                            tab_held = true;
                        }
                        _ => {}
                    },
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Released,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    } => match key {
                        VirtualKeyCode::Tab => {
                            tab_held = false;
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            });

            if tab_held {
                let mut total = 0usize;
                let (_fgs, _bgs, mut ids) = term.layer_slices_mut();
                for (_x, _y, ref_mut) in ids.iter_mut() {
                    *ref_mut = total as u8;
                    total += 1;
                }
            } else {
                term.set_all_ids(b' ');
                term.get_id_mut((watcher_position.0 as usize, watcher_position.1 as usize))
                    .map(|mut_ref| *mut_ref = b'@');
            }

            term.clear_draw_swap()
                .map_err(|err_vec| {
                    for e in err_vec {
                        eprintln!("clear_draw_swap error: {:?}", e);
                    }
                })
                .ok();

            // Error check
        }
    }
}
