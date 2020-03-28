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
    pub fn new(xstate: u64, ystate: u64) -> Self {
        return Self { xstate, ystate };
    }

    pub fn range(&mut self, min: usize, max: usize) -> usize {
        return ((self.next_u64() as usize) % (max - min)) + min;
    }

    pub fn new_from_u64(seed: u64) -> Self {
        let mut res = Self::new(seed, seed ^ 0xec77152282650854);
        for _ in 0..4 {
            res.next_u64();
        }
        return res;
    }

    pub fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    pub fn next_u64(&mut self) -> u64 {
        let xp = self.xstate;
        self.xstate = 15241094284759029579u64.wrapping_mul(self.ystate);
        self.ystate = self.ystate.wrapping_sub(xp);
        self.ystate = self.ystate.rotate_left(27);
        return xp;
    }
}

pub struct RegexScript {
    rng: RomuPrng,
    remaining: usize,
}

impl RegexScript {
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
        return (self.rng.next_u32() as usize) % val;
    }

    pub fn get_range(&mut self, min: usize, max: usize) -> usize {
        return self.get_mod(max - min) + min;
    }
}

fn append_char(res: &mut Vec<u8>, chr: char) {
    let mut buf = [0; 4];
    res.extend_from_slice(chr.encode_utf8(&mut buf).as_bytes())
}

fn append_lit(res: &mut Vec<u8>, lit: &Literal) {
    use regex_syntax::hir::Literal::*;

    match lit {
        Unicode(chr) => append_char(res, *chr),
        Byte(b) => res.push(*b),
    }
}

fn append_unicode_range(res: &mut Vec<u8>, scr: &mut RegexScript, cls: &ClassUnicodeRange) {
    let mut chr_a_buf = [0; 4];
    let mut chr_b_buf = [0; 4];
    cls.start().encode_utf8(&mut chr_a_buf);
    cls.end().encode_utf8(&mut chr_b_buf);
    let a = u32::from_le_bytes(chr_a_buf);
    let b = u32::from_le_bytes(chr_b_buf);
    let c = scr.get_range(a as usize, (b + 1) as usize) as u32;
    append_char(res, std::char::from_u32(c).unwrap());
}

fn append_byte_range(res: &mut Vec<u8>, scr: &mut RegexScript, cls: &ClassBytesRange) {
    res.push(scr.get_range(cls.start() as usize, (cls.end() + 1) as usize) as u8);
}

fn append_class(res: &mut Vec<u8>, scr: &mut RegexScript, cls: &Class) {
    use regex_syntax::hir::Class::*;
    match cls {
        Unicode(cls) => {
            let rngs = cls.ranges();
            let rng = rngs[scr.get_mod(rngs.len())];
            append_unicode_range(res, scr, &rng);
        }
        Bytes(cls) => {
            let rngs = cls.ranges();
            let rng = rngs[scr.get_mod(rngs.len())];
            append_byte_range(res, scr, &rng);
        }
    }
}

fn get_length(scr: &mut RegexScript) -> usize {
    let bits = scr.get_mod(8);
    return scr.get_mod(2 << bits);
}

fn get_repetition_range(rep: &RepetitionRange, scr: &mut RegexScript) -> usize {
    use regex_syntax::hir::RepetitionRange::*;
    match rep {
        Exactly(a) => return *a as usize,
        AtLeast(a) => return get_length(scr) + (*a as usize),
        Bounded(a, b) => return scr.get_range(*a as usize, *b as usize),
    }
}

fn get_repetitions(rep: &RepetitionKind, scr: &mut RegexScript) -> usize {
    use regex_syntax::hir::RepetitionKind::*;
    match rep {
        ZeroOrOne => return scr.get_mod(2),
        ZeroOrMore => return get_length(scr),
        OneOrMore => return 1 + get_length(scr),
        Range(rng) => get_repetition_range(rng, scr),
    }
}

pub fn generate(hir: &Hir, seed: u64) -> Vec<u8> {
    use regex_syntax::hir::HirKind::*;
    let mut scr = RegexScript::new(seed);
    let mut stack = vec![hir];
    let mut res = vec![];
    while stack.len() > 0 {
        match stack.pop().unwrap().kind() {
            Empty => {}
            Literal(lit) => append_lit(&mut res, lit),
            Class(cls) => append_class(&mut res, &mut scr, cls),
            Anchor(_) => {}
            WordBoundary(_) => {}
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
    return res;
}
