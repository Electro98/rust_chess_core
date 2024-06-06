#[derive(Debug)]
pub struct BetweenIterator {
    current: u8,
    target: u8,
    step: u8,
}

impl Iterator for BetweenIterator {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.current = self.current.wrapping_add(self.step);
        if self.current == self.target || !is_valid_coord(self.current) {
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
    let step = if is_in_diagonal_line(from, to) {
        let (file, rank) = (diff & 0x08 == 0, diff & 0x80 == 0);
        match (file, rank) {
            (true, true) => 0x11,
            (true, false) => 0xf1,
            (false, true) => 0x0f,
            (false, false) => 0xef,
        }
    } else {
        let (x_or_y, positive) = (
            diff & 0x0f != 0,
            (diff.wrapping_shr(4) | diff & 0x0f) & 0x08 == 0,
        );
        match (x_or_y, positive) {
            (true, true) => 0x01,
            (true, false) => 0xff,
            (false, true) => 0x10,
            (false, false) => 0xf0,
        }
    };
    BetweenIterator {
        current: from,
        target: to,
        step,
    }
}

pub struct DirectionIterator {
    position: u8,
    direction: u8,
}

impl Iterator for DirectionIterator {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.position = self.position.wrapping_add(self.direction);
        if is_valid_coord(self.position) {
            Some(self.position)
        } else {
            None
        }
    }
}

pub fn in_direction(position: u8, direction: u8) -> DirectionIterator {
    DirectionIterator {
        position,
        direction,
    }
}

pub fn distance(from: u8, to: u8) -> u8 {
    (from & 0x0f).abs_diff(to & 0x0f) + ((from & 0xf0) >> 4).abs_diff((to & 0xf0) >> 4)
}

pub fn is_in_straight_line(a: u8, b: u8) -> bool {
    let diff = a.abs_diff(b);
    diff & 0x0f == 0 || diff & 0xf0 == 0
}

pub fn is_in_diagonal_line(a: u8, b: u8) -> bool {
    let diff = a.abs_diff(b);
    diff & 0x0f == (diff & 0xf0) >> 4
}

#[inline]
pub fn is_valid_coord(coord: u8) -> bool {
    coord & 0x88 == 0x00
}

#[inline]
pub fn compact_pos(rank: u8, file: u8) -> u8 {
    rank << 4 | file
}

#[inline]
pub fn unpack_pos<T: From<u8>, V: Into<u8>>(pos: V) -> (T, T) {
    let pos: u8 = pos.into();
    (((pos & 0xf0) >> 4).into(), (pos & 0x0f).into())
}
