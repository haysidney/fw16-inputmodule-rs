use crate::control::*;
use crate::games::game_of_life::*;
use crate::games::pong_animation::*;
use crate::games::snake_animation::*;
use crate::matrix::Grid;
use crate::matrix::*;
use crate::patterns::*;

// TODO
// - [ ] Is there a cancellable Iterator? I think Java/Kotlin has one
// - [ ] Each one has a number of frames
// - [ ] Each one might have a different frame-rate

#[allow(clippy::large_enum_variant)]
pub enum Animation {
    ZigZag(ZigZagIterator),
    Gof(GameOfLifeIterator),
    Percentage(StartupPercentageIterator),
    Breathing(BreathingIterator),
    Snake(SnakeIterator),
    Pong(PongIterator),
    Plasma(PlasmaIterator),
}
impl Iterator for Animation {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Animation::ZigZag(x) => x.next(),
            Animation::Gof(x) => x.next(),
            Animation::Percentage(x) => x.next(),
            Animation::Breathing(x) => x.next(),
            Animation::Snake(x) => x.next(),
            Animation::Pong(x) => x.next(),
            Animation::Plasma(x) => x.next(),
        }
    }
}

pub struct ZigZagIterator {
    frames: usize,
    current_frame: usize,
}

impl ZigZagIterator {
    pub fn new(frames: usize) -> Self {
        Self {
            frames,
            current_frame: 0,
        }
    }
}

impl Default for ZigZagIterator {
    fn default() -> Self {
        Self::new(34)
    }
}

impl Iterator for ZigZagIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame < self.frames {
            let mut next = zigzag();
            next.rotate(self.current_frame);
            self.current_frame += 1;
            Some(next)
        } else {
            None
        }
    }
}

pub struct StartupPercentageIterator {
    frames: usize,
    current_frame: usize,
}

impl Default for StartupPercentageIterator {
    fn default() -> Self {
        Self {
            frames: 34,
            current_frame: 0,
        }
    }
}

impl Iterator for StartupPercentageIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame < self.frames {
            self.current_frame += 1;
            Some(rows(self.current_frame))
        } else {
            None
        }
    }
}

pub struct GameOfLifeIterator {
    state: GameOfLifeState,
    frames_remaining: usize,
}

impl GameOfLifeIterator {
    pub fn new(start_param: GameOfLifeStartParam, frames: usize) -> Self {
        Self {
            // Could start with a custom grid
            state: GameOfLifeState::new(start_param, &Grid::default()),
            frames_remaining: frames,
        }
    }
}

impl Iterator for GameOfLifeIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frames_remaining > 0 {
            self.frames_remaining -= 1;
            // Only update every 8th frame, otherwise the animation is too fast
            if self.frames_remaining % 8 == 0 {
                self.state.tick();
            }
            Some(self.state.draw_matrix())
        } else {
            None
        }
    }
}

pub struct BreathingIterator {
    frames_remaining: usize,
    current_brightness: u8,
}

impl BreathingIterator {
    pub fn new(frames: usize) -> Self {
        Self {
            frames_remaining: frames,
            current_brightness: 0,
        }
    }
}
impl Default for BreathingIterator {
    fn default() -> Self {
        Self::new(64)
    }
}

impl Iterator for BreathingIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        if self.frames_remaining > 0 {
            let mut grid = Grid::default();
            let breath_step = 4;
            // TODO: Make it cycle up and down
            self.current_brightness = (self.current_brightness + breath_step) % 255;
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    grid.0[x][y] = self.current_brightness;
                }
            }
            self.frames_remaining -= 1;
            Some(grid)
        } else {
            None
        }
    }
}

/// Four interfering sine fields, ported from the host's `fwled ambient plasma`
/// (FW16/leds/ambient.py). Never ends: it runs until a drawing command
/// replaces it. Integer only: the M0+ has no FPU.
pub struct PlasmaIterator {
    /// Phase of each sine term, 65536 = one turn
    t: [u16; 4],
}

/// Phase steps per frame at the default 32 FPS (host: t += 0.36 rad/s,
/// terms at t, -0.7t, 0.5t, -1.3t)
const PLASMA_STEP: [i16; 4] = [117, -82, 59, -152];

/// sin(2 pi i / 256) * 127
#[rustfmt::skip]
const SIN: [i8; 256] = [
    0, 3, 6, 9, 12, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46,
    49, 51, 54, 57, 60, 63, 65, 68, 71, 73, 76, 78, 81, 83, 85, 88,
    90, 92, 94, 96, 98, 100, 102, 104, 106, 107, 109, 111, 112, 113, 115, 116,
    117, 118, 120, 121, 122, 122, 123, 124, 125, 125, 126, 126, 126, 127, 127, 127,
    127, 127, 127, 127, 126, 126, 126, 125, 125, 124, 123, 122, 122, 121, 120, 118,
    117, 116, 115, 113, 112, 111, 109, 107, 106, 104, 102, 100, 98, 96, 94, 92,
    90, 88, 85, 83, 81, 78, 76, 73, 71, 68, 65, 63, 60, 57, 54, 51,
    49, 46, 43, 40, 37, 34, 31, 28, 25, 22, 19, 16, 12, 9, 6, 3,
    0, -3, -6, -9, -12, -16, -19, -22, -25, -28, -31, -34, -37, -40, -43, -46,
    -49, -51, -54, -57, -60, -63, -65, -68, -71, -73, -76, -78, -81, -83, -85, -88,
    -90, -92, -94, -96, -98, -100, -102, -104, -106, -107, -109, -111, -112, -113, -115, -116,
    -117, -118, -120, -121, -122, -122, -123, -124, -125, -125, -126, -126, -126, -127, -127, -127,
    -127, -127, -127, -127, -126, -126, -126, -125, -125, -124, -123, -122, -122, -121, -120, -118,
    -117, -116, -115, -113, -112, -111, -109, -107, -106, -104, -102, -100, -98, -96, -94, -92,
    -90, -88, -85, -83, -81, -78, -76, -73, -71, -68, -65, -63, -60, -57, -54, -51,
    -49, -46, -43, -40, -37, -34, -31, -28, -25, -22, -19, -16, -12, -9, -6, -3,
];

/// Host's (v/4 + 0.5)^3 then gamma 2.2: 255 * (i / 254)^6.6
#[rustfmt::skip]
const PLASMA_SHAPE: [u8; 255] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3,
    3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6,
    6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 11, 11, 12,
    12, 13, 13, 14, 14, 15, 15, 16, 17, 17, 18, 19, 19, 20, 21, 22,
    23, 24, 24, 25, 26, 27, 28, 29, 30, 31, 33, 34, 35, 36, 38, 39,
    40, 42, 43, 45, 46, 48, 49, 51, 53, 54, 56, 58, 60, 62, 64, 66,
    68, 70, 73, 75, 77, 80, 82, 85, 88, 90, 93, 96, 99, 102, 105, 108,
    111, 115, 118, 121, 125, 129, 132, 136, 140, 144, 148, 153, 157, 161, 166, 171,
    175, 180, 185, 190, 196, 201, 206, 212, 218, 224, 230, 236, 242, 248, 255,
];

impl PlasmaIterator {
    /// Random start phases, so two modules don't show the same picture
    pub fn new(seed: u8) -> Self {
        let s = (seed as u16) << 8;
        Self {
            t: [s, s.wrapping_mul(3), s.wrapping_mul(5), s.wrapping_mul(7)],
        }
    }
}

fn isqrt(n: u32) -> u32 {
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

impl Iterator for PlasmaIterator {
    type Item = Grid;

    fn next(&mut self) -> Option<Self::Item> {
        let mut grid = Grid::default();
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                // Distance from the panel centre, 8.8 fixed point, doubled
                let dx = 2 * x as i32 - WIDTH as i32;
                let dy = 2 * y as i32 - HEIGHT as i32;
                let d2 = isqrt(((dx * dx + dy * dy) as u32) << 16);
                // Spatial frequencies 0.45 x, 0.21 y, 0.17 (x + y), 0.35 d in phase units
                let phases = [
                    (x as u32 * 4694) as u16,
                    (y as u32 * 2190) as u16,
                    ((x + y) as u32 * 1773) as u16,
                    (d2 * 3651 / 512) as u16,
                ];
                let mut v: i16 = 0;
                for (p, t) in phases.iter().zip(self.t.iter()) {
                    v += SIN[(p.wrapping_add(*t) >> 8) as usize] as i16;
                }
                // v / 4 + 0.5 clamped to 0..1, as on the host
                grid.0[x][y] = PLASMA_SHAPE[((v + 254).clamp(0, 508) >> 1) as usize];
            }
        }
        for (t, step) in self.t.iter_mut().zip(PLASMA_STEP.iter()) {
            *t = t.wrapping_add_signed(*step);
        }
        Some(grid)
    }
}
