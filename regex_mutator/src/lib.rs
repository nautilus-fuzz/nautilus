// Nautilus
// Copyright (C) 2020  Daniel Teuchert, Cornelius Aschermann, Sergej Schumilo

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate regex_syntax;

use regex_syntax::hir::{
    Class, ClassBytesRange, ClassUnicodeRange, Hir, Literal, RepetitionKind, RepetitionRange,
};

pub struct RomuPrng {
    xstate: u64,
    ystate: u64,
}

impl RomuPrng {
    #[must_use]
    pub fn new(xstate: u64, ystate: u64) -> Self {
        Self { xstate, ystate }
    }

    pub fn range(&mut self, min: usize, max: usize) -> usize {
        ((self.next_u64() as usize) % (max - min)) + min
    }

    #[must_use]
    pub fn new_from_u64(seed: u64) -> Self {
        let mut res = Self::new(seed, seed ^ 0xec77_1522_8265_0854);
        for _ in 0..4 {
            res.next_u64();
        }
        res
    }

    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    pub fn next_u64(&mut self) -> u64 {
        let xp = self.xstate;
        self.xstate = 15_241_094_284_759_029_579_u64.wrapping_mul(self.ystate);
        self.ystate = self.ystate.wrapping_sub(xp);
        self.ystate = self.ystate.rotate_left(27);
        xp
    }
}

pub struct RegexScript {
    rng: RomuPrng,
    remaining: usize,
}

impl RegexScript {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let mut rng = RomuPrng::new_from_u64(seed);

        let len = if rng.next_u64() % 256 == 0 {
            rng.next_u64() % 0xffff
        } else {
            let len = 1 << (rng.next_u64() % 8);
            rng.next_u64() % len
        };
        RegexScript {
            rng,
            remaining: len as usize,
        }
    }

    pub fn get_mod(&mut self, val: usize) -> usize {
        if self.remaining == 0 {
            return 0;
        }
        (self.rng.next_u32() as usize) % val
    }

    pub fn get_range(&mut self, min: usize, max: usize) -> usize {
        self.get_mod(max - min) + min
    }
}

fn append_char(res: &mut Vec<u8>, chr: char) {
    let mut buf = [0; 4];
    res.extend_from_slice(chr.encode_utf8(&mut buf).as_bytes());
}

fn append_lit(res: &mut Vec<u8>, lit: &Literal) {
    use regex_syntax::hir::Literal::{Byte, Unicode};

    match lit {
        Unicode(chr) => append_char(res, *chr),
        Byte(b) => res.push(*b),
    }
}

fn append_unicode_range(res: &mut Vec<u8>, scr: &mut RegexScript, cls: ClassUnicodeRange) {
    let mut chr_a_buf = [0; 4];
    let mut chr_b_buf = [0; 4];
    cls.start().encode_utf8(&mut chr_a_buf);
    cls.end().encode_utf8(&mut chr_b_buf);
    let a = u32::from_le_bytes(chr_a_buf);
    let b = u32::from_le_bytes(chr_b_buf);
    let c = scr.get_range(a as usize, (b + 1) as usize) as u32;
    append_char(res, std::char::from_u32(c).unwrap());
}

fn append_byte_range(res: &mut Vec<u8>, scr: &mut RegexScript, cls: ClassBytesRange) {
    res.push(scr.get_range(cls.start() as usize, (cls.end() + 1) as usize) as u8);
}

fn append_class(res: &mut Vec<u8>, scr: &mut RegexScript, cls: &Class) {
    use regex_syntax::hir::Class::{Bytes, Unicode};
    match cls {
        Unicode(cls) => {
            let rngs = cls.ranges();
            let rng = rngs[scr.get_mod(rngs.len())];
            append_unicode_range(res, scr, rng);
        }
        Bytes(cls) => {
            let rngs = cls.ranges();
            let rng = rngs[scr.get_mod(rngs.len())];
            append_byte_range(res, scr, rng);
        }
    }
}

fn get_length(scr: &mut RegexScript) -> usize {
    let bits = scr.get_mod(8);
    scr.get_mod(2 << bits)
}

fn get_repetition_range(rep: &RepetitionRange, scr: &mut RegexScript) -> usize {
    use regex_syntax::hir::RepetitionRange::{AtLeast, Bounded, Exactly};
    match rep {
        Exactly(a) => *a as usize,
        AtLeast(a) => get_length(scr) + (*a as usize),
        Bounded(a, b) => scr.get_range(*a as usize, *b as usize),
    }
}

fn get_repetitions(rep: &RepetitionKind, scr: &mut RegexScript) -> usize {
    use regex_syntax::hir::RepetitionKind::{OneOrMore, Range, ZeroOrMore, ZeroOrOne};
    match rep {
        ZeroOrOne => scr.get_mod(2),
        ZeroOrMore => get_length(scr),
        OneOrMore => 1 + get_length(scr),
        Range(rng) => get_repetition_range(rng, scr),
    }
}

#[must_use]
pub fn generate(hir: &Hir, seed: u64) -> Vec<u8> {
    use regex_syntax::hir::HirKind::{
        Alternation, Anchor, Class, Concat, Empty, Group, Literal, Repetition, WordBoundary,
    };
    let mut scr = RegexScript::new(seed);
    let mut stack = vec![hir];
    let mut res = vec![];
    while stack.is_empty() {
        match stack.pop().unwrap().kind() {
            Anchor(_) | WordBoundary(_) | Empty => {}
            Literal(lit) => append_lit(&mut res, lit),
            Class(cls) => append_class(&mut res, &mut scr, cls),
            Repetition(rep) => {
                let num = get_repetitions(&rep.kind, &mut scr);
                for _ in 0..num {
                    stack.push(&rep.hir);
                }
            }
            Group(grp) => stack.push(&grp.hir),
            Concat(hirs) => hirs.iter().rev().for_each(|h| stack.push(h)),
            Alternation(hirs) => stack.push(&hirs[scr.get_mod(hirs.len())]),
        }
    }
    res
}
