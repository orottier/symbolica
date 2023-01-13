use byteorder::{LittleEndian, WriteBytesExt};
use bytes::{Buf, BufMut};
use std::{cmp::Ordering, io::Cursor};

use crate::{representations::tree::Number, state::ResettableBuffer, utils};

use super::{
    number::{RationalNumberReader, RationalNumberWriter},
    tree::Atom,
    AtomT, AtomView, FunctionT, Identifier, ListIteratorT, NumberT, OwnedAtomT, OwnedFunctionT,
    OwnedNumberT, OwnedTermT, OwnedVarT, TermT, VarT,
};

const NUM_ID: u8 = 1;
const VAR_ID: u8 = 2;
const FN_ID: u8 = 3;
const TERM_ID: u8 = 4;
const TYPE_MASK: u8 = 0b00000111;
const DIRTY_FLAG: u8 = 0b10000000;

#[derive(Debug, Copy, Clone)]
pub struct DefaultRepresentation {}

#[derive(Debug, Clone)]
pub struct OwnedAtom {
    data: Vec<u8>,
}

impl OwnedAtomT for OwnedAtom {
    type P = DefaultRepresentation;

    fn from_num(source: <Self::P as AtomT>::ON) -> Self {
        OwnedAtom { data: source.data }
    }

    fn write<'a>(&mut self, source: &AtomView<'a, Self::P>) {
        // TODO: does not work yet, as an upgrade to Term is needed and sizes need to be changed
        self.data.extend(source.get_data());
    }

    fn write_tree(&mut self, source: &Atom) {
        // TODO: does not work yet, as an upgrade to Term is needed and sizes need to be changed
        self.linearize(source);
    }

    fn from_tree(a: &Atom) -> Self {
        let mut owned_data = OwnedAtom { data: vec![] };
        owned_data.linearize(a);
        owned_data
    }

    fn to_tree(&self) -> Atom {
        OwnedAtom::write_to_tree(self.data.as_slice()).0
    }

    fn to_view<'a>(&'a self) -> AtomView<'a, Self::P> {
        match self.data[0] & TYPE_MASK {
            VAR_ID => AtomView::Var(VarView { data: &self.data }),
            FN_ID => AtomView::Function(FunctionView { data: &self.data }),
            NUM_ID => AtomView::Number(NumberView { data: &self.data }),
            TERM_ID => AtomView::Term(TermView { data: &self.data }),
            x => unreachable!("Bad id: {}", x),
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl ResettableBuffer for OwnedAtom {
    fn new() -> Self {
        OwnedAtom { data: vec![] }
    }

    fn reset(&mut self) {
        self.data.clear();
    }
}

#[derive(Debug, Clone)]
pub struct OwnedNumber {
    data: Vec<u8>,
}

impl OwnedNumberT for OwnedNumber {
    type P = DefaultRepresentation;

    fn from_view<'a>(a: NumberView<'a>) -> Self {
        OwnedNumber {
            data: a.data.to_vec(),
        }
    }

    fn add<'a>(&mut self, other: &NumberView<'a>) {
        let a = self.to_num_view().get_numden();
        let b = other.get_numden();

        let c = (a.0 * b.0, a.1 * b.1);
        let gcd = utils::gcd_unsigned(c.0 as u64, c.1 as u64);

        self.data.truncate(1);
        (c.0 as u64 / gcd).write_frac(c.1 as u64 / gcd, &mut self.data);
    }

    fn to_num_view(&self) -> NumberView {
        assert!(self.data[0] & TYPE_MASK == NUM_ID);
        NumberView { data: &self.data }
    }
}

impl ResettableBuffer for OwnedNumber {
    fn new() -> Self {
        let mut data = Vec::new();
        data.put_u8(NUM_ID);
        0u64.write_num(&mut data);

        OwnedNumber { data }
    }

    fn reset(&mut self) {
        self.data.clear();
        self.data.put_u8(NUM_ID);
        0u64.write_num(&mut self.data);
    }
}

#[derive(Debug, Clone)]
pub struct OwnedVar {
    data: Vec<u8>,
}

impl OwnedVarT for OwnedVar {
    type P = DefaultRepresentation;

    fn from_id_pow(&mut self, id: Identifier, pow: OwnedNumber) {
        self.data.put_u8(VAR_ID);
        (id.to_u32() as u64).write_frac(1, &mut self.data);
        self.data.extend(pow.data);
    }

    fn to_var_view<'a>(&'a self) -> <Self::P as AtomT>::V<'a> {
        VarView { data: &self.data }
    }

    fn to_atom(&mut self, out: &mut OwnedAtom) {
        out.data.clone_from(&self.data);
    }
}

impl ResettableBuffer for OwnedVar {
    fn new() -> Self {
        OwnedVar { data: vec![] }
    }

    fn reset(&mut self) {
        self.data.clear();
    }
}

#[derive(Debug, Clone)]
pub struct OwnedFunction {
    data: Vec<u8>,
}

impl OwnedFunctionT for OwnedFunction {
    type P = DefaultRepresentation;

    fn from_name(&mut self, id: Identifier) {
        self.data.put_u8(VAR_ID);
        (id.to_u32() as u64).write_num(&mut self.data);
    }

    fn set_dirty(&mut self, dirty: bool) {
        if dirty {
            self.data[0] &= DIRTY_FLAG;
        } else {
            self.data[0] &= !DIRTY_FLAG;
        }
    }

    fn add_arg(&mut self, other: AtomView<Self::P>) {
        // TODO: update the size
        self.data.extend(other.get_data());
        todo!()
    }

    fn to_func_view<'a>(&'a self) -> <Self::P as AtomT>::F<'a> {
        FunctionView { data: &self.data }
    }

    fn to_atom(&mut self, out: &mut OwnedAtom) {
        out.data.clone_from(&self.data);
    }
}

impl ResettableBuffer for OwnedFunction {
    fn new() -> Self {
        OwnedFunction { data: vec![] }
    }

    fn reset(&mut self) {
        self.data.clear();
    }
}

pub struct OwnedTerm {
    data: Vec<u8>,
}

impl OwnedTermT for OwnedTerm {
    type P = DefaultRepresentation;

    fn extend<'a>(&mut self, other: AtomView<'a, DefaultRepresentation>) {
        // may increase size of the num of args
        let c = &self.data[1 + 4..];

        let buf_pos = 1 + 4;

        let mut n_args;
        (n_args, _, _) = c.get_frac_u64();

        match other {
            AtomView::Term(_t) => {
                todo!();
            }
            _ => {
                n_args += 1;
                self.data.extend(other.get_data());
            }
        }

        // FIXME: this may overwrite the rest of the term
        // assume for now it does not
        n_args.write_frac_fixed(1, &mut self.data[1 + 4..]);

        let new_buf_pos = self.data.len();

        let mut cursor = &mut self.data[1..];
        cursor
            .write_u32::<LittleEndian>((new_buf_pos - buf_pos) as u32)
            .unwrap();
    }

    fn to_term_view<'a>(&'a self) -> <Self::P as AtomT>::T<'a> {
        TermView { data: &self.data }
    }

    fn to_atom(&mut self, out: &mut OwnedAtom) {
        out.data.clone_from(&self.data);
    }
}

impl ResettableBuffer for OwnedTerm {
    fn new() -> Self {
        let mut data = Vec::new();
        data.put_u8(TERM_ID);
        data.put_u32_le(0 as u32);
        0u64.write_num(&mut data);

        OwnedTerm { data }
    }

    fn reset(&mut self) {
        self.data.clear();
        self.data.put_u8(TERM_ID);
        self.data.put_u32_le(0 as u32);
        0u64.write_num(&mut self.data);
    }
}

impl AtomT for DefaultRepresentation {
    type N<'a> = NumberView<'a>;
    type V<'a> = VarView<'a>;
    type F<'a> = FunctionView<'a>;
    type T<'a> = TermView<'a>;
    type O = OwnedAtom;
    type ON = OwnedNumber;
    type OV = OwnedVar;
    type OF = OwnedFunction;
    type OT = OwnedTerm;
}

impl<'a> VarT<'a> for VarView<'a> {
    type P = DefaultRepresentation;

    fn get_name(&self) -> Identifier {
        Identifier::from((&self.data[1..]).get_frac_u64().0 as u32)
    }

    fn get_pow(&self) -> NumberView<'a> {
        NumberView {
            data: (&self.data[1..]).skip_rational(),
        }
    }
}

impl OwnedAtom {
    pub fn linearize(&mut self, atom: &Atom) {
        match atom {
            Atom::Var(name, pow) => {
                self.data.put_u8(VAR_ID);
                (name.to_u32() as u64).write_frac(1, &mut self.data);

                self.data.put_u8(NUM_ID);
                pow.num.write_frac(pow.den, &mut self.data);
            }
            Atom::Fn(name, args) => {
                self.data.put_u8(FN_ID);
                let size_pos = self.data.len();
                self.data.put_u32_le(0 as u32); // length of entire fn without flag
                let buf_pos = self.data.len();

                // pack name and args
                (name.to_u32() as u64).write_frac(args.len() as u64, &mut self.data);

                for a in args {
                    self.linearize(a);
                }
                let new_buf_pos = self.data.len();

                let mut cursor: Cursor<&mut [u8]> = Cursor::new(&mut self.data[size_pos..]);

                cursor
                    .write_u32::<LittleEndian>((new_buf_pos - buf_pos) as u32)
                    .unwrap();
            }
            Atom::Number(n) => {
                self.data.put_u8(NUM_ID);

                n.num.write_frac(n.den, &mut self.data);
            }
            Atom::Term(args) => {
                self.data.put_u8(TERM_ID);

                let size_pos = self.data.len();
                self.data.put_u32_le(0 as u32); // length of entire fn without flag
                let buf_pos = self.data.len();

                (args.len() as u64).write_num(&mut self.data);

                for a in args {
                    self.linearize(a);
                }
                let new_buf_pos = self.data.len();

                let mut cursor: Cursor<&mut [u8]> = Cursor::new(&mut self.data[size_pos..]);

                cursor
                    .write_u32::<LittleEndian>((new_buf_pos - buf_pos) as u32)
                    .unwrap();
            }
        }
    }

    fn write_to_tree(mut source: &[u8]) -> (Atom, &[u8]) {
        match source.get_u8() & TYPE_MASK {
            VAR_ID => {
                let name;
                (name, _, source) = source.get_frac_u64();

                source.get_u8(); // num tag
                let (num, den);
                (num, den, source) = source.get_frac_u64();

                (
                    Atom::Var(Identifier::from(name as u32), Number::new(num, den)),
                    source,
                )
            }
            FN_ID => {
                source.get_u32_le(); // size

                let (name, n_args);
                (name, n_args, source) = source.get_frac_u64();

                let mut args = Vec::with_capacity(n_args as usize);
                for _ in 0..n_args {
                    let (a, s) = OwnedAtom::write_to_tree(source);
                    source = s;
                    args.push(a);
                }

                (Atom::Fn(Identifier::from(name as u32), args), source)
            }
            NUM_ID => {
                let (num, den);
                (num, den, source) = source.get_frac_u64();
                (Atom::Number(Number::new(num, den)), source)
            }
            TERM_ID => {
                source.get_u32_le(); // size

                let n_args;
                (n_args, _, source) = source.get_frac_u64();

                let mut args = Vec::with_capacity(n_args as usize);
                for _ in 0..n_args {
                    let (a, s) = OwnedAtom::write_to_tree(source);
                    source = s;
                    args.push(a);
                }

                (Atom::Term(args), source)
            }
            x => unreachable!("Bad id: {}", x),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct VarView<'a> {
    pub data: &'a [u8],
}

#[derive(Debug, Copy, Clone)]
pub struct FunctionView<'a> {
    pub data: &'a [u8],
}

impl<'a> FunctionT<'a> for FunctionView<'a> {
    type P = DefaultRepresentation;
    type I = ListIterator<'a>;

    fn get_name(&self) -> Identifier {
        Identifier::from((&self.data[1 + 4..]).get_frac_u64().0 as u32)
    }

    fn get_nargs(&self) -> usize {
        (&self.data[1 + 4..]).get_frac_u64().1 as usize
    }

    fn is_dirty(&self) -> bool {
        (self.data[0] & DIRTY_FLAG) != 0
    }

    fn cmp(&self, other: &Self) -> Ordering {
        self.get_name().cmp(&other.get_name())
    }

    #[inline]
    fn into_iter(&self) -> Self::I {
        let mut c = self.data;
        c.get_u8();
        c.get_u32_le(); // size

        let n_args;
        (_, n_args, c) = c.get_frac_u64(); // name

        ListIterator {
            data: c,
            length: n_args as u32,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct NumberView<'a> {
    pub data: &'a [u8],
}

impl<'a> NumberT<'a> for NumberView<'a> {
    type P = DefaultRepresentation;

    fn is_one(&self) -> bool {
        self.data.is_one()
    }

    fn add<'b>(&self, other: &Self, out: &mut OwnedAtom) {
        let a = self.get_numden();
        let b = other.get_numden();

        let c = (a.0 * b.0, a.1 * b.1);
        let gcd = utils::gcd_unsigned(c.0 as u64, c.1 as u64);

        out.data.put_u8(NUM_ID);

        (c.0 as u64 / gcd).write_frac(c.1 as u64 / gcd, &mut out.data);
    }

    #[inline]
    fn get_numden(&self) -> (u64, u64) {
        let mut c = self.data;
        c.get_u8();

        let num;
        let den;
        (num, den, _) = c.get_frac_u64();

        (num, den)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TermView<'a> {
    pub data: &'a [u8],
}

impl<'a> AtomView<'a, DefaultRepresentation> {
    pub fn from(source: &'a [u8]) -> AtomView<'a, DefaultRepresentation> {
        match source[0] {
            VAR_ID => AtomView::Var(VarView { data: source }),
            FN_ID => AtomView::Function(FunctionView { data: source }),
            NUM_ID => AtomView::Number(NumberView { data: source }),
            TERM_ID => AtomView::Term(TermView { data: source }),
            x => unreachable!("Bad id: {}", x),
        }
    }

    pub fn to_atom(&self) -> Atom {
        OwnedAtom::write_to_tree(self.get_data()).0
    }

    fn get_data(&self) -> &[u8] {
        match self {
            AtomView::Number(n) => n.data,
            AtomView::Var(v) => v.data,
            AtomView::Function(f) => f.data,
            AtomView::Term(t) => t.data,
        }
    }
}

impl<'a> TermT<'a> for TermView<'a> {
    type P = DefaultRepresentation;
    type I = ListIterator<'a>;

    #[inline]
    fn into_iter(&self) -> Self::I {
        let mut c = self.data;
        c.get_u8();
        c.get_u32_le(); // size

        let n_args;
        (n_args, _, c) = c.get_frac_u64();

        ListIterator {
            data: c,
            length: n_args as u32,
        }
    }

    fn get_nargs(&self) -> usize {
        (&self.data[1 + 4..]).get_frac_u64().0 as usize
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ListIterator<'a> {
    data: &'a [u8],
    length: u32,
}

impl<'a> ListIteratorT<'a> for ListIterator<'a> {
    type P = DefaultRepresentation;

    #[inline(always)]
    fn next(&mut self) -> Option<AtomView<'a, Self::P>> {
        if self.length == 0 {
            return None;
        }

        self.length -= 1;

        let start = self.data;

        let cur_id = self.data.get_u8() & TYPE_MASK;

        match cur_id {
            VAR_ID => {
                self.data = self.data.skip_rational();
                self.data.advance(1); // also skip num tag
                self.data = self.data.skip_rational();
            }
            NUM_ID => {
                self.data = self.data.skip_rational();
            }
            FN_ID => {
                let n_size = self.data.get_u32_le();
                self.data.advance(n_size as usize);
            }
            TERM_ID => {
                let n_size = self.data.get_u32_le();
                self.data.advance(n_size as usize);
            }
            //x => unreachable!("Bad id {}", x),
            _ => {
                return None;
            }
        }

        let len = unsafe { self.data.as_ptr().offset_from(start.as_ptr()) } as usize;

        let data = unsafe { start.get_unchecked(..len) };
        match cur_id {
            VAR_ID => {
                return Some(AtomView::Var(VarView { data }));
            }
            NUM_ID => {
                return Some(AtomView::Number(NumberView { data }));
            }
            FN_ID => {
                return Some(AtomView::Function(FunctionView { data }));
            }
            TERM_ID => {
                return Some(AtomView::Term(TermView { data }));
            }
            //x => unreachable!("Bad id {}", x),
            _ => {
                return None;
            }
        }
    }
}

#[test]
pub fn representation_size() {
    let a = Atom::Fn(
        Identifier::from(1),
        vec![
            Atom::Var(Identifier::from(2), Number::new(1, 1)),
            Atom::Fn(
                Identifier::from(3),
                vec![
                    Atom::Term(vec![
                        Atom::Number(Number::new(3, 1)),
                        Atom::Number(Number::new(13, 1)),
                    ]),
                    Atom::Term(vec![
                        Atom::Number(Number::new(3, 1)),
                        Atom::Number(Number::new(13, 1)),
                    ]),
                    Atom::Term(vec![
                        Atom::Number(Number::new(3, 1)),
                        Atom::Number(Number::new(13, 1)),
                    ]),
                    Atom::Term(vec![
                        Atom::Number(Number::new(3, 1)),
                        Atom::Number(Number::new(13, 1)),
                    ]),
                    Atom::Number(Number::new(4, 2)),
                    Atom::Number(Number::new(4, 2)),
                    Atom::Number(Number::new(4, 2)),
                    Atom::Number(Number::new(4, 2)),
                ],
            ),
            Atom::Var(Identifier::from(6), Number::new(1, 1)),
            Atom::Var(Identifier::from(2), Number::new(1, 1)),
            Atom::Var(Identifier::from(2), Number::new(1, 1)),
            Atom::Var(Identifier::from(2), Number::new(1, 1)),
            Atom::Var(Identifier::from(2), Number::new(1, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
            Atom::Number(Number::new(2, 1)),
        ],
    );
    println!("expr={:?}", a);

    let b = OwnedAtom::from_tree(&a);

    println!("lin size: {:?} bytes", b.data.len());

    let c = b.to_tree();

    if a != c {
        panic!("in and out is different: {:?} vs {:?}", a, c);
    }

    b.to_view().dbg_print_tree(0);
}
