#[derive(Debug)]
pub struct BetweenIterator {
    current: u8,
    target: u8,
    step: u8,
}

impl Iterator for BetweenIterator {
    type Item = u8;

    #[inline(always)]
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
        let (file, rank) = (diff & 0x80 == 0, diff & 0x08 == 0);
        match (rank, file) {
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

pub fn distance(a: u8, b: u8) -> u8 {
    let file_diff = (a & 0xf0).abs_diff(b & 0xf0);
    let rank_diff = (a & 0x0f).abs_diff(b & 0x0f);
    (file_diff >> 4) + rank_diff
}

pub fn is_in_straight_line(a: u8, b: u8) -> bool {
    let file_diff = (a & 0xf0).abs_diff(b & 0xf0);
    let rank_diff = (a & 0x0f).abs_diff(b & 0x0f);
    file_diff == 0 || rank_diff == 0
}

pub fn is_in_diagonal_line(a: u8, b: u8) -> bool {
    let file_diff = (a & 0xf0).abs_diff(b & 0xf0);
    let rank_diff = (a & 0x0f).abs_diff(b & 0x0f);
    file_diff >> 4 == rank_diff
}

#[inline]
pub fn is_valid_coord(coord: u8) -> bool {
    coord & 0x88 == 0x00
}

#[inline]
pub fn compact_pos(file: u8, rank: u8) -> u8 {
    file << 4 | rank
}

#[inline]
pub fn unpack_pos<T: From<u8>, V: Into<u8>>(pos: V) -> (T, T) {
    let pos: u8 = pos.into();
    (((pos & 0xf0) >> 4).into(), (pos & 0x0f).into())
}

const POS_TO_STRING: [&str; 128] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", "XX", "XX", "XX", "XX", "XX", "XX", "XX", "XX",
];

pub fn pos_to_str(pos: u8) -> &'static str {
    POS_TO_STRING[pos as usize]
}
