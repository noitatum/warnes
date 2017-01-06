use sdl2::EventPump;
use sdl2::keyboard::{Scancode as S, KeyboardState};

// A, B, Select, Start, Up, Down, Left, Right
const PLAYER_KEYS : [[S; 8]; 2] = [
    [S::Num1, S::Num2, S::Num3, S::Num0, S::Up, S::Down, S::Left, S::Right],
    [S::J, S::K, S::L, S::Return, S::W, S::S, S::A, S::D],
];

// Returns true if user wants to exit, sets controller keys accordingly
pub fn get_keys(event_pump: &mut EventPump, keys: &mut [[u8; 8]; 2]) -> bool {
    event_pump.pump_events();
    let state = KeyboardState::new(event_pump);
    for (p, ps) in keys.iter_mut().zip(PLAYER_KEYS.iter()) {
        for (k, ks) in p.iter_mut().zip(ps.iter()) {
            *k = state.is_scancode_pressed(*ks) as u8;
        }
    }
    return state.is_scancode_pressed(S::Escape);
}
