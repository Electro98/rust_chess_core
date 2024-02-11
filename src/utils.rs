pub struct BetweenIterator {
    current: u8,
    target: u8,
    step: u8,
}

impl Iterator for BetweenIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = self.current.wrapping_add(self.step);
        if self.current == self.target {
            None
        } else {
            Some(self.current)
        }
    }
}

pub fn between(from: u8, to: u8) -> BetweenIterator {
    // TODO: Check if they on one side of spectre
    #[cfg(debug_assertions)]
    if !is_in_diagonal_line(from, to) && !is_in_straight_line(from, to) {
        panic!("Points can't form line to search between them!")
    }
    let diff = to.wrapping_sub(from);
    // TODO: precompute table?
    // const TABLE: [u8; 256] = {
    //     let mut table = [0; 256];
    //     ... something ...
    //     table
    // };
    let dx: u8 = {
        if diff & 0xf0 == 0 {
            0
        } else if diff & 0x80 == 0x80 {
            0xf0
        } else {
            0x10
        }
    };
    let dy: u8 = {
        if diff & 0x0f == 0 {
            0
        } else if diff & 0x08 == 0x08 {
            0x0f
        } else {
            0x01
        }
    };
    let step = dx | dy;
    BetweenIterator {
        current: from,
        target: to,
        step,
    }
}

pub fn distance(from: u8, to: u8) -> u8 {
    (from & 0x0f).abs_diff(to & 0x0f) + (from & 0xf0 >> 4).abs_diff(to & 0xf0 >> 4)
}

pub fn is_in_straight_line(a: u8, b: u8) -> bool {
    let diff = a.abs_diff(b);
    diff & 0x0f == 0 || diff & 0xf0 == 0
}

pub fn is_in_diagonal_line(a: u8, b: u8) -> bool {
    let diff = a.abs_diff(b);
    diff & 0x0f == diff & 0xf0
}

#[inline]
pub fn is_valid_coord(coord: u8) -> bool {
    coord & 0x88 == 0x00
}
