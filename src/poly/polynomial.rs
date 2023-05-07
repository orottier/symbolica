use ahash::{HashMap, HashMapExt};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::fmt;
use std::fmt::Display;
use std::mem;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::representations::Identifier;
use crate::rings::finite_field::FiniteField;
use crate::rings::{EuclideanDomain, Field, Ring};

use super::{Exponent, INLINED_EXPONENTS};
use smallvec::{smallvec, SmallVec};

/// Multivariate polynomial with a sparse degree and variable dense representation.
// TODO: implement EuclideanDomain for MultivariatePolynomial
#[derive(Clone, Hash)]
pub struct MultivariatePolynomial<F: Ring, E: Exponent> {
    // Data format: the i-th monomial is stored as coefficients[i] and
    // exponents[i * nvars .. (i + 1) * nvars]. Keep coefficients.len() == nterms and
    // exponents.len() == nterms * nvars. Terms are always expanded and sorted by the exponents via
    // cmp_exponents().
    pub coefficients: Vec<F::Element>,
    pub exponents: Vec<E>,
    pub nterms: usize,
    pub nvars: usize,
    pub field: F,
    pub var_map: Option<SmallVec<[Identifier; INLINED_EXPONENTS]>>,
}

impl<F: Ring, E: Exponent> MultivariatePolynomial<F, E> {
    /// Constructs a zero polynomial.
    #[inline]
    pub fn new(
        nvars: usize,
        field: F,
        cap: Option<usize>,
        var_map: Option<SmallVec<[Identifier; INLINED_EXPONENTS]>>,
    ) -> Self {
        Self {
            coefficients: Vec::with_capacity(cap.unwrap_or(0)),
            exponents: Vec::with_capacity(cap.unwrap_or(0) * nvars),
            nterms: 0,
            nvars,
            field,
            var_map,
        }
    }

    /// Constructs a zero polynomial with the given number of variables and capacity,
    /// inheriting the field and variable map from `self`.
    #[inline]
    pub fn new_from(&self, cap: Option<usize>) -> Self {
        Self {
            coefficients: Vec::with_capacity(cap.unwrap_or(0)),
            exponents: Vec::with_capacity(cap.unwrap_or(0) * self.nvars),
            nterms: 0,
            nvars: self.nvars,
            field: self.field,
            var_map: self.var_map.clone(),
        }
    }

    /// Constructs a constant polynomial with the given number of variables.
    #[inline]
    pub fn from_constant(constant: F::Element, nvars: usize, field: F) -> Self {
        if F::is_zero(&constant) {
            return Self::new(nvars, field, None, None);
        }
        Self {
            coefficients: vec![constant],
            exponents: vec![E::zero(); nvars],
            nterms: 1,
            nvars,
            field,
            var_map: None,
        }
    }

    /// Constructs a polynomial with a single term.
    #[inline]
    pub fn from_monomial(coefficient: F::Element, exponents: Vec<E>, field: F) -> Self {
        if F::is_zero(&coefficient) {
            return Self::new(exponents.len(), field, None, None);
        }
        Self {
            coefficients: vec![coefficient],
            nvars: exponents.len(),
            exponents,
            nterms: 1,
            field,
            var_map: None,
        }
    }

    /// Get the ith monomial
    pub fn to_monomial_view(&self, i: usize) -> MonomialView<F, E> {
        assert!(i < self.nterms);

        MonomialView {
            coefficient: &self.coefficients[i],
            exponents: &self.exponents(i),
        }
    }

    #[inline]
    pub fn reserve(&mut self, cap: usize) -> &mut Self {
        self.coefficients.reserve(cap);
        self.exponents.reserve(cap * self.nvars);
        self
    }

    #[inline]
    pub fn zero(field: F) -> Self {
        Self::new(0, field, None, None)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.nterms == 0
    }

    #[inline]
    pub fn one(field: F) -> Self {
        MultivariatePolynomial::from_constant(field.one(), 0, field)
    }

    #[inline]
    pub fn is_one(&self) -> bool {
        self.nterms == 1
            && self.field.is_one(&self.coefficients[0])
            && self.exponents.iter().all(|x| x.is_zero())
    }

    /// Returns the number of terms in the polynomial.
    #[inline]
    pub fn nterms(&self) -> usize {
        return self.nterms;
    }

    /// Returns the number of variables in the polynomial.
    #[inline]
    pub fn nvars(&self) -> usize {
        return self.nvars;
    }

    /// Returns true if the polynomial is constant.
    #[inline]
    pub fn is_constant(&self) -> bool {
        if self.is_zero() {
            return true;
        }
        if self.nterms >= 2 {
            return false;
        }
        debug_assert!(!F::is_zero(self.coefficients.first().unwrap()));
        return self.exponents.iter().all(|e| e.is_zero());
    }

    /// Returns the `index`th monomial, starting from the back.
    #[inline]
    pub fn coefficient_back(&self, index: usize) -> &F::Element {
        &self.coefficients[self.nterms - index - 1]
    }

    /// Returns the slice for the exponents of the specified monomial.
    #[inline]
    pub fn exponents(&self, index: usize) -> &[E] {
        &self.exponents[index * self.nvars..(index + 1) * self.nvars]
    }

    /// Returns the slice for the exponents of the specified monomial
    /// starting from the back.
    #[inline]
    pub fn exponents_back(&self, index: usize) -> &[E] {
        let index = self.nterms - index - 1;
        &self.exponents[index * self.nvars..(index + 1) * self.nvars]
    }

    pub fn last_exponents(&self) -> &[E] {
        assert!(self.nterms > 0);
        &self.exponents[(self.nterms - 1) * self.nvars..self.nterms * self.nvars]
    }

    /// Returns the mutable slice for the exponents of the specified monomial.
    #[inline]
    fn exponents_mut(&mut self, index: usize) -> &mut [E] {
        &mut self.exponents[index * self.nvars..(index + 1) * self.nvars]
    }

    /// Returns the number of variables in the polynomial.
    #[inline]
    pub fn clear(&mut self) {
        self.nterms = 0;
        self.coefficients.clear();
        self.exponents.clear();
    }

    /// Get the variable map.
    pub fn get_var_map(
        &self,
    ) -> &Option<smallvec::SmallVec<[crate::representations::Identifier; INLINED_EXPONENTS]>> {
        &self.var_map
    }

    /// Unify the variable maps of two polynomials, i.e.
    /// rewrite a polynomial in `x` and one in `y` to a
    /// two polynomial in `x` and `y`.
    pub fn unify_var_map(&mut self, other: &mut Self) {
        assert!(self.var_map.is_some() && other.var_map.is_some());

        let mut new_var_map = self.var_map.clone().unwrap();
        let mut new_var_pos_other = vec![0; other.nvars];
        for (pos, v) in new_var_pos_other
            .iter_mut()
            .zip(other.var_map.as_ref().unwrap())
        {
            if let Some(p) = new_var_map.iter().position(|x| x == v) {
                *pos = p;
            } else {
                *pos = new_var_map.len();
                new_var_map.push(*v);
            }
        }

        let mut newexp = vec![E::zero(); new_var_map.len() * self.nterms];

        for t in 0..self.nterms {
            newexp[t * new_var_map.len()..t * new_var_map.len() + self.nvars]
                .copy_from_slice(self.exponents(t));
        }

        self.var_map = Some(new_var_map.clone());
        self.exponents = newexp;
        self.nvars = new_var_map.len();

        // reconstruct 'other' with correct monomial ordering
        let mut newother = Self::new(
            new_var_map.len(),
            other.field.clone(),
            Some(other.nterms),
            Some(new_var_map.clone()),
        );
        let mut newexp: SmallVec<[E; INLINED_EXPONENTS]> = smallvec![E::zero(); new_var_map.len()];
        for t in other.into_iter() {
            for c in &mut newexp {
                *c = E::zero();
            }

            for (var, e) in t.exponents.iter().enumerate() {
                newexp[new_var_pos_other[var]] = *e;
            }
            newother.append_monomial(t.coefficient.clone(), &newexp);
        }
        *other = newother;
    }

    /// Reverse the monomial ordering in-place.
    fn reverse(&mut self) {
        self.coefficients.reverse();

        let midu = if self.nterms % 2 == 0 {
            self.nvars * (self.nterms / 2)
        } else {
            self.nvars * (self.nterms / 2 + 1)
        };

        let (l, r) = self.exponents.split_at_mut(midu);

        let rend = r.len();
        for i in 0..self.nterms / 2 {
            l[i * self.nvars..(i + 1) * self.nvars]
                .swap_with_slice(&mut r[rend - (i + 1) * self.nvars..rend - i * self.nvars]);
        }
    }

    /// Compares exponent vectors of two monomials.
    #[inline]
    fn cmp_exponents(a: &[E], b: &[E]) -> Ordering {
        debug_assert!(a.len() == b.len());
        // TODO: Introduce other term orders.
        a.cmp(b)
    }

    /// Grow the exponent list so the variable index fits in.
    pub fn grow_to(&mut self, var: usize) {
        if self.nterms() < var {
            // move all the exponents
            self.exponents.resize(var, E::zero());
            unimplemented!()
        }
    }

    /// Check if the polynomial is sorted and has only non-zero coefficients
    pub fn check_consistency(&self) {
        assert_eq!(self.coefficients.len(), self.nterms);
        assert_eq!(self.exponents.len(), self.nterms * self.nvars);

        for c in &self.coefficients {
            if F::is_zero(c) {
                panic!("Inconsistent polynomial (0 coefficient): {}", self);
            }
        }

        for t in 1..self.nterms {
            match MultivariatePolynomial::<F, E>::cmp_exponents(
                self.exponents(t),
                &self.exponents(t - 1),
            ) {
                Ordering::Equal => panic!("Inconsistent polynomial (equal monomials): {}", self),
                Ordering::Less => panic!(
                    "Inconsistent polynomial (wrong monomial ordering): {}",
                    self
                ),
                Ordering::Greater => {}
            }
        }
    }

    /// Append a monomial to the back. It merges with the last monomial if the
    /// exponents are equal.
    #[inline]
    pub fn append_monomial_back(&mut self, coefficient: F::Element, exponents: &[E]) {
        if F::is_zero(&coefficient) {
            return;
        }

        if self.nterms > 0 && exponents == self.last_exponents() {
            self.field
                .add_assign(&mut self.coefficients[self.nterms - 1], &coefficient);

            if F::is_zero(&self.coefficients[self.nterms - 1]) {
                self.coefficients.pop();
                self.exponents.truncate((self.nterms - 1) * self.nvars);
                self.nterms -= 1;
            }
        } else {
            self.coefficients.push(coefficient);
            self.exponents.extend_from_slice(exponents);
            self.nterms += 1;
        }
    }

    /// Appends a monomial to the polynomial.
    pub fn append_monomial(&mut self, coefficient: F::Element, exponents: &[E]) {
        if F::is_zero(&coefficient) {
            return;
        }
        if self.nvars != exponents.len() {
            panic!(
                "nvars mismatched: got {}, expected {}",
                exponents.len(),
                self.nvars
            );
        }

        // should we append to the back?
        if self.nterms == 0 || self.last_exponents() < exponents {
            self.coefficients.push(coefficient);
            self.exponents.extend_from_slice(exponents);
            self.nterms += 1;
            return;
        }

        // Binary search to find the insert-point.
        let mut l = 0;
        let mut r = self.nterms;

        while l <= r {
            let m = (l + r) / 2;
            let c = Self::cmp_exponents(exponents, self.exponents(m)); // note the reversal

            match c {
                Ordering::Equal => {
                    // Add the two coefficients.
                    self.field
                        .add_assign(&mut self.coefficients[m], &coefficient);
                    if F::is_zero(&self.coefficients[m]) {
                        // The coefficient becomes zero. Remove this monomial.
                        self.coefficients.remove(m);
                        let i = m * self.nvars;
                        self.exponents.splice(i..i + self.nvars, Vec::new());
                        self.nterms -= 1;
                    }
                    return;
                }
                Ordering::Greater => {
                    l = m + 1;

                    if l == self.nterms {
                        self.coefficients.push(coefficient);
                        self.exponents.extend_from_slice(exponents);
                        self.nterms += 1;
                        return;
                    }
                }
                Ordering::Less => {
                    if m == 0 {
                        self.coefficients.insert(0, coefficient);
                        self.exponents.splice(0..0, exponents.iter().cloned());
                        self.nterms += 1;
                        return;
                    }

                    r = m - 1;
                }
            }
        }

        self.coefficients.insert(l, coefficient);
        let i = l * self.nvars;
        self.exponents.splice(i..i, exponents.iter().cloned());
        self.nterms += 1;
    }
}

impl<F: Ring + fmt::Debug, E: Exponent + fmt::Debug> fmt::Debug for MultivariatePolynomial<F, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_zero() {
            return write!(f, "[]");
        }
        let mut first = true;
        write!(f, "[ ")?;
        for monomial in self {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(
                f,
                "{{ {:?}, {:?} }}",
                monomial.coefficient, monomial.exponents
            )?;
        }
        write!(f, " ]")
    }
}

impl<F: Ring + Display, E: Exponent> Display for MultivariatePolynomial<F, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first_term = true;
        for monomial in self {
            let mut is_first_factor = true;
            if self.field.is_one(&monomial.coefficient) {
                if !is_first_term {
                    write!(f, "+")?;
                }
            } else if monomial.coefficient.eq(&self.field.neg(&self.field.one())) {
                write!(f, "-")?;
            } else {
                if is_first_term {
                    write!(f, "{}", monomial.coefficient)?;
                } else {
                    write!(f, "{:+}", monomial.coefficient)?;
                }
                is_first_factor = false;
            }
            is_first_term = false;
            for (i, e) in monomial.exponents.into_iter().enumerate() {
                if e.is_zero() {
                    continue;
                }
                if is_first_factor {
                    is_first_factor = false;
                } else {
                    write!(f, "*")?;
                }
                write!(f, "x{}", i)?;
                if e.to_u32() != 1 {
                    write!(f, "^{}", e)?;
                }
            }
            if is_first_factor {
                write!(f, "1")?;
            }
        }
        if is_first_term {
            write!(f, "0")?;
        }

        Display::fmt(&self.field, f)
    }
}

impl<F: Ring + PartialEq, E: Exponent> PartialEq for MultivariatePolynomial<F, E> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.nvars != other.nvars {
            if self.is_zero() && other.is_zero() {
                // Both are 0.
                return true;
            }
            if self.is_zero() || other.is_zero() {
                // One of them is 0.
                return false;
            }
            panic!("nvars mismatched");
        }
        if self.nterms != other.nterms {
            return false;
        }
        self.exponents.eq(&other.exponents) && self.coefficients.eq(&other.coefficients)
    }
}

impl<F: Ring + Eq, E: Exponent> Eq for MultivariatePolynomial<F, E> {}

impl<F: Ring, E: Exponent> Add for MultivariatePolynomial<F, E> {
    type Output = Self;

    fn add(mut self, mut other: Self) -> Self::Output {
        debug_assert_eq!(self.field, other.field);
        debug_assert!(other.var_map.is_none() || self.var_map == other.var_map);

        if self.is_zero() {
            return other;
        }
        if other.is_zero() {
            return self;
        }
        if self.nvars != other.nvars {
            panic!("nvars mismatched");
        }

        // Merge the two polynomials, which are assumed to be already sorted.

        let mut new_coefficients = vec![F::zero(); self.nterms + other.nterms];
        let mut new_exponents: Vec<E> = vec![E::zero(); self.nvars * (self.nterms + other.nterms)];
        let mut new_nterms = 0;
        let mut i = 0;
        let mut j = 0;

        macro_rules! insert_monomial {
            ($source:expr, $index:expr) => {
                mem::swap(
                    &mut new_coefficients[new_nterms],
                    &mut $source.coefficients[$index],
                );

                new_exponents[new_nterms * $source.nvars..(new_nterms + 1) * $source.nvars]
                    .clone_from_slice($source.exponents($index));
                new_nterms += 1;
            };
        }

        while i < self.nterms && j < other.nterms {
            let c = Self::cmp_exponents(self.exponents(i), other.exponents(j));
            match c {
                Ordering::Less => {
                    insert_monomial!(self, i);
                    i += 1;
                }
                Ordering::Greater => {
                    insert_monomial!(other, j);
                    j += 1;
                }
                Ordering::Equal => {
                    self.field
                        .add_assign(&mut self.coefficients[i], &other.coefficients[j]);
                    if !F::is_zero(&self.coefficients[i]) {
                        insert_monomial!(self, i);
                    }
                    i += 1;
                    j += 1;
                }
            }
        }

        while i < self.nterms {
            insert_monomial!(self, i);
            i += 1;
        }

        while j < other.nterms {
            insert_monomial!(other, j);
            j += 1;
        }

        new_coefficients.truncate(new_nterms);
        new_exponents.truncate(self.nvars * new_nterms);

        Self {
            coefficients: new_coefficients,
            exponents: new_exponents,
            nterms: new_nterms,
            nvars: self.nvars,
            field: self.field,
            var_map: self.var_map,
        }
    }
}

impl<'a, 'b, F: Ring, E: Exponent> Add<&'a MultivariatePolynomial<F, E>>
    for &'b MultivariatePolynomial<F, E>
{
    type Output = MultivariatePolynomial<F, E>;

    fn add(self, other: &'a MultivariatePolynomial<F, E>) -> Self::Output {
        (self.clone()).add(other.clone())
    }
}

impl<F: Ring, E: Exponent> Sub for MultivariatePolynomial<F, E> {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        self.add(other.neg())
    }
}

impl<'a, 'b, F: Ring, E: Exponent> Sub<&'a MultivariatePolynomial<F, E>>
    for &'b MultivariatePolynomial<F, E>
{
    type Output = MultivariatePolynomial<F, E>;

    fn sub(self, other: &'a MultivariatePolynomial<F, E>) -> Self::Output {
        (self.clone()).add(other.clone().neg())
    }
}

impl<F: Ring, E: Exponent> Neg for MultivariatePolynomial<F, E> {
    type Output = Self;
    fn neg(mut self) -> Self::Output {
        // Negate coefficients of all terms.
        for c in &mut self.coefficients {
            *c = self.field.neg(c);
        }
        self
    }
}

impl<'a, 'b, F: Ring, E: Exponent> Mul<&'a MultivariatePolynomial<F, E>>
    for &'b MultivariatePolynomial<F, E>
{
    type Output = MultivariatePolynomial<F, E>;

    fn mul(self, other: &'a MultivariatePolynomial<F, E>) -> Self::Output {
        self.heap_mul(other)
    }
}

impl<'a, F: Ring, E: Exponent> Mul<&'a MultivariatePolynomial<F, E>>
    for MultivariatePolynomial<F, E>
{
    type Output = MultivariatePolynomial<F, E>;

    fn mul(self, other: &'a MultivariatePolynomial<F, E>) -> Self::Output {
        (&self).heap_mul(other)
    }
}

impl<'a, 'b, F: EuclideanDomain, E: Exponent> Div<&'a MultivariatePolynomial<F, E>>
    for &'b MultivariatePolynomial<F, E>
{
    type Output = MultivariatePolynomial<F, E>;

    fn div(self, other: &'a MultivariatePolynomial<F, E>) -> Self::Output {
        let (r, q) = self.quot_rem(&other);
        if q.is_zero() {
            r
        } else {
            panic!(
                "No clean division of {} by {} possible: rest = {}",
                self, other, r
            );
        }
    }
}

impl<'a, F: EuclideanDomain, E: Exponent> Div<&'a MultivariatePolynomial<F, E>>
    for MultivariatePolynomial<F, E>
{
    type Output = MultivariatePolynomial<F, E>;

    fn div(
        self: MultivariatePolynomial<F, E>,
        other: &'a MultivariatePolynomial<F, E>,
    ) -> Self::Output {
        (&self).div(other)
    }
}

// FIXME: cannot implement Add<F::Element> because F::Element could be MultivariatePolynomial<F, E>
impl<F: Ring, E: Exponent> MultivariatePolynomial<F, E> {
    /// Multiply every coefficient with `other`.
    pub fn mul_coeff(mut self, other: F::Element) -> Self {
        for c in &mut self.coefficients {
            self.field.mul_assign(c, &other);
        }
        self
    }

    /// Add a new monomial with coefficient `other` and exponent one.
    pub fn add_monomial(mut self, other: F::Element) -> Self {
        let nvars = self.nvars;
        self.append_monomial(other, &vec![E::zero(); nvars]);
        self
    }

    #[inline]
    fn mul_monomial(mut self, coefficient: &F::Element, exponents: &[E]) -> Self {
        debug_assert_eq!(self.nvars, exponents.len());
        debug_assert!(self.nterms > 0);
        debug_assert!(!F::is_zero(coefficient));
        for c in &mut self.coefficients {
            self.field.mul_assign(c, coefficient);
        }
        for i in 0..self.nterms {
            let ee = self.exponents_mut(i);
            for (e1, e2) in ee.iter_mut().zip(exponents) {
                *e1 = e1.checked_add(e2).expect("overflow in adding exponents");
            }
        }
        self
    }

    /// Get the degree of the variable `x`.
    /// This operation is O(n).
    pub fn degree(&self, x: usize) -> E {
        let mut max = E::zero();
        for t in 0..self.nterms {
            if max < self.exponents(t)[x] {
                max = self.exponents(t)[x];
            }
        }
        max
    }

    // Get the highest degree of a variable in the leading monomial.
    pub fn ldegree(&self, v: usize) -> E {
        if self.is_zero() {
            return E::zero();
        }
        self.last_exponents()[v].clone()
    }

    /// Get the highest degree of the leading monomial.
    pub fn ldegree_max(&self) -> E {
        if self.is_zero() {
            return E::zero();
        }
        self.last_exponents()
            .iter()
            .max()
            .unwrap_or(&E::zero())
            .clone()
    }

    /// Get the leading coefficient.
    pub fn lcoeff(&self) -> F::Element {
        if self.is_zero() {
            return F::zero();
        }
        self.coefficients.last().unwrap().clone()
    }

    /// Get the leading coefficient under a given variable ordering.
    /// This operation is O(n) if the variables are out of order.
    pub fn lcoeff_varorder(&self, vars: &[usize]) -> F::Element {
        if vars.windows(2).all(|s| s[0] < s[1]) {
            return self.lcoeff();
        }

        let mut highest = vec![E::zero(); self.nvars];
        let mut highestc = &F::zero();

        'nextmon: for m in self.into_iter() {
            let mut more = false;
            for &v in vars {
                if more {
                    highest[v] = m.exponents[v];
                } else {
                    match m.exponents[v].cmp(&highest[v]) {
                        Ordering::Less => {
                            continue 'nextmon;
                        }
                        Ordering::Greater => {
                            highest[v] = m.exponents[v];
                            more = true;
                        }
                        Ordering::Equal => {}
                    }
                }
            }
            highestc = &m.coefficient;
        }
        debug_assert!(!F::is_zero(highestc));
        highestc.clone()
    }

    /// Get the leading coefficient viewed as a polynomial
    /// in all variables except the last variable `n`.
    pub fn lcoeff_last(&self, n: usize) -> MultivariatePolynomial<F, E> {
        if self.is_zero() {
            return MultivariatePolynomial::zero(self.field);
        }
        // the last variable should have the least sorting priority,
        // so the last term should still be the lcoeff
        let last = self.last_exponents();

        let mut res = self.new_from(None);
        let mut e = vec![E::zero(); self.nvars];
        for t in (0..self.nterms()).rev() {
            if (0..self.nvars - 1).all(|i| self.exponents(t)[i] == last[i] || i == n) {
                e[n] = self.exponents(t)[n];
                res.append_monomial(self.coefficients[t].clone(), &e);
                e[n] = E::zero();
            } else {
                break;
            }
        }

        res
    }

    /// Get the leading coefficient viewed as a polynomial
    /// in all variables with order as described in `vars` except the last variable in `vars`.
    /// This operation is O(n) if the variables are out of order.
    pub fn lcoeff_last_varorder(&self, vars: &[usize]) -> MultivariatePolynomial<F, E> {
        if self.is_zero() {
            return MultivariatePolynomial::zero(self.field);
        }

        if vars.windows(2).all(|s| s[0] < s[1]) {
            return self.lcoeff_last(*vars.last().unwrap());
        }

        let (vars, lastvar) = vars.split_at(vars.len() - 1);

        let mut highest = vec![E::zero(); self.nvars];
        let mut indices = Vec::with_capacity(10);

        'nextmon: for (i, m) in self.into_iter().enumerate() {
            let mut more = false;
            for &v in vars {
                if more {
                    highest[v] = m.exponents[v];
                } else {
                    match m.exponents[v].cmp(&highest[v]) {
                        Ordering::Less => {
                            continue 'nextmon;
                        }
                        Ordering::Greater => {
                            highest[v] = m.exponents[v];
                            indices.clear();
                            more = true;
                        }
                        Ordering::Equal => {}
                    }
                }
            }
            indices.push(i);
        }

        let mut res = self.new_from(None);
        let mut e = vec![E::zero(); self.nvars];
        for i in indices {
            e[lastvar[0]] = self.exponents(i)[lastvar[0]];
            res.append_monomial(self.coefficients[i].clone(), &e);
            e[lastvar[0]] = E::zero();
        }
        res
    }

    /// Change the order of the variables in the polynomial, using `varmap`.
    /// The map can also be reversed, by setting `inverse` to `true`.
    pub fn rearrange(&self, varmap: &[usize], inverse: bool) -> MultivariatePolynomial<F, E> {
        let mut res = self.new_from(None);
        let mut newe = vec![E::zero(); self.nvars];
        for m in self.into_iter() {
            for x in 0..varmap.len() {
                if !inverse {
                    newe[x] = m.exponents[varmap[x]];
                } else {
                    newe[varmap[x]] = m.exponents[x];
                }
            }

            res.append_monomial(m.coefficient.clone(), &newe);
        }
        res
    }

    /// Replace a variable `n' in the polynomial by an element from
    /// the ring `v'.
    pub fn replace(&self, n: usize, v: F::Element) -> MultivariatePolynomial<F, E> {
        let mut res = self.new_from(Some(self.nterms));
        let mut e = vec![E::zero(); self.nvars];
        for t in 0..self.nterms {
            let c = self.field.mul(
                &self.coefficients[t],
                &self.field.pow(&v, self.exponents(t)[n].to_u32() as u64),
            );

            for (i, ee) in self.exponents(t).iter().enumerate() {
                e[i] = *ee;
            }

            e[n] = E::zero();
            res.append_monomial(c, &e);
        }

        res
    }

    /// Replace all variables except `v` in the polynomial by elements from
    /// the ring.
    pub fn replace_all_except(
        &self,
        v: usize,
        r: &[(usize, F::Element)],
        cache: &mut [Vec<F::Element>],
    ) -> MultivariatePolynomial<F, E> {
        let mut tm: HashMap<E, F::Element> = HashMap::new();

        for t in 0..self.nterms {
            let mut c = self.coefficients[t].clone();
            for (n, vv) in r {
                let p = self.exponents(t)[*n].to_u32() as usize;
                if p > 0 {
                    if p < cache[*n].len() {
                        if F::is_zero(&cache[*n][p]) {
                            cache[*n][p] = self.field.pow(vv, p as u64);
                        }

                        self.field.mul_assign(&mut c, &cache[*n][p]);
                    } else {
                        self.field.mul_assign(&mut c, &self.field.pow(vv, p as u64));
                    }
                }
            }

            tm.entry(self.exponents(t)[v])
                .and_modify(|e| self.field.add_assign(e, &c))
                .or_insert(c);
        }

        let mut res = self.new_from(None);
        let mut e = vec![E::zero(); self.nvars];
        for (k, c) in tm {
            e[v] = k;
            res.append_monomial(c, &e);
            e[v] = E::zero();
        }

        res
    }

    /// Create a univariate polynomial out of a multivariate one.
    // TODO: allow a MultivariatePolynomial as a coefficient
    pub fn to_univariate_polynomial_list(
        &self,
        x: usize,
    ) -> Vec<(MultivariatePolynomial<F, E>, E)> {
        if self.coefficients.is_empty() {
            return vec![];
        }

        // get maximum degree for variable x
        let mut maxdeg = E::zero();
        for t in 0..self.nterms {
            let d = self.exponents(t)[x];
            if d > maxdeg {
                maxdeg = d.clone();
            }
        }

        // construct the coefficient per power of x
        let mut result = vec![];
        let mut e = vec![E::zero(); self.nvars];
        for d in 0..maxdeg.to_u32() + 1 {
            // TODO: add bounds estimate
            let mut a = self.new_from(None);
            for t in 0..self.nterms {
                if self.exponents(t)[x].to_u32() == d {
                    for (i, ee) in self.exponents(t).iter().enumerate() {
                        e[i] = *ee;
                    }
                    e[x] = E::zero();
                    a.append_monomial(self.coefficients[t].clone(), &e);
                }
            }

            if !a.is_zero() {
                result.push((a, E::from_u32(d)));
            }
        }

        result
    }

    /// Split the polynomial as a polynomial in `xs` if include is true,
    /// else excluding `xs`.
    pub fn to_multivariate_polynomial_list(
        &self,
        xs: &[usize],
        include: bool,
    ) -> HashMap<Vec<E>, MultivariatePolynomial<F, E>> {
        if self.coefficients.is_empty() {
            return HashMap::new();
        }

        let mut tm: HashMap<Vec<E>, MultivariatePolynomial<F, E>> = HashMap::new();
        let mut e = vec![E::zero(); self.nvars];
        let mut me = vec![E::zero(); self.nvars];
        for t in 0..self.nterms {
            for (i, ee) in self.exponents(t).iter().enumerate() {
                e[i] = *ee;
                me[i] = E::zero();
            }

            for x in xs {
                me[*x] = e[*x].clone();
                e[*x] = E::zero();
            }

            if include {
                let add = match tm.get_mut(&me) {
                    Some(x) => {
                        x.append_monomial(self.coefficients[t].clone(), &e);
                        false
                    }
                    None => true,
                };

                if add {
                    tm.insert(
                        me.clone(),
                        // TODO: add nterms estimate
                        MultivariatePolynomial::from_monomial(
                            self.coefficients[t].clone(),
                            e.clone(),
                            self.field,
                        ),
                    );
                }
            } else {
                let add = match tm.get_mut(&e) {
                    Some(x) => {
                        x.append_monomial(self.coefficients[t].clone(), &me);
                        false
                    }
                    None => true,
                };

                if add {
                    tm.insert(
                        e.clone(),
                        MultivariatePolynomial::from_monomial(
                            self.coefficients[t].clone(),
                            me.clone(),
                            self.field,
                        ),
                    );
                }
            }
        }

        tm
    }

    /// Multiplication for multivariate polynomials using a custom variation of the heap method
    /// described in "Sparse polynomial division using a heap" by Monagan, Pearce (2011).
    /// It uses a heap to obtain the next monomial of the result in an ordered fashion.
    /// Additionally, this method uses a hashmap with the monomial exponent as a key and a vector of all pairs
    /// of indices in `self` and `other` that have that monomial exponent when multiplied together.
    /// When a multiplication of two monomials is considered, its indices are added to the hashmap,
    /// but they are only added to the heap if the monomial exponent is new. As a result, the heap
    /// only has new monomials, and by taking (and removing) the corresponding entry from the hashmap, all
    /// monomials that have that exponent can be summed. Then, new monomials combinations are added that
    /// should be considered next as they are smaller than the current monomial.
    pub fn heap_mul(&self, other: &MultivariatePolynomial<F, E>) -> MultivariatePolynomial<F, E> {
        let mut res = self.new_from(Some(self.nterms));

        let mut cache: HashMap<SmallVec<[E; INLINED_EXPONENTS]>, SmallVec<[(usize, usize); 5]>> =
            HashMap::with_capacity(self.nterms);

        // create a min-heap since our polynomials are sorted smallest to largest
        let mut h: BinaryHeap<Reverse<SmallVec<[E; INLINED_EXPONENTS]>>> =
            BinaryHeap::with_capacity(self.nterms);

        let monom: SmallVec<[E; INLINED_EXPONENTS]> = self
            .exponents(0)
            .iter()
            .zip(other.exponents(0))
            .map(|(e1, e2)| *e1 + *e2)
            .collect();
        cache.insert(monom.clone(), smallvec![(0, 0)]);
        h.push(Reverse(monom));

        while h.len() > 0 {
            let cur_mon = h.pop().unwrap();

            let mut coefficient = F::zero();

            for (i, j) in cache.remove(&cur_mon.0).unwrap() {
                self.field.add_assign(
                    &mut coefficient,
                    &self
                        .field
                        .mul(&self.coefficients[i], &other.coefficients[j]),
                );

                if j + 1 < other.nterms {
                    let monom: SmallVec<[E; INLINED_EXPONENTS]> = self
                        .exponents(i)
                        .iter()
                        .zip(other.exponents(j + 1))
                        .map(|(e1, e2)| *e1 + *e2)
                        .collect();

                    cache
                        .entry(monom.clone())
                        .or_insert_with(|| {
                            h.push(Reverse(monom)); // only add when new
                            smallvec![]
                        })
                        .push((i, j + 1));
                }

                // only increment i when (i, 0) has been extracted since this
                // new term is necessarily smaller
                if j == 0 && i + 1 < self.nterms {
                    let monom: SmallVec<[E; INLINED_EXPONENTS]> = self
                        .exponents(i + 1)
                        .iter()
                        .zip(other.exponents(j))
                        .map(|(e1, e2)| *e1 + *e2)
                        .collect();
                    cache
                        .entry(monom.clone())
                        .or_insert_with(|| {
                            h.push(Reverse(monom)); // only add when new
                            smallvec![]
                        })
                        .push((i + 1, 0));
                }
            }

            if !F::is_zero(&coefficient) {
                res.coefficients.push(coefficient);
                res.exponents.extend_from_slice(&cur_mon.0);
                res.nterms += 1;
            }
        }
        res
    }
}

impl<F: EuclideanDomain, E: Exponent> MultivariatePolynomial<F, E> {
    /// Get the content from the coefficients.
    pub fn content(&self) -> F::Element {
        if self.coefficients.is_empty() {
            return F::zero();
        }
        let mut c = self.coefficients.first().unwrap().clone();
        for cc in self.coefficients.iter().skip(1) {
            c = self.field.gcd(&c, cc);
        }
        c
    }

    /// Synthetic division for univariate polynomials
    // TODO: create UnivariatePolynomial?
    pub fn synthetic_division(
        &self,
        div: &MultivariatePolynomial<F, E>,
    ) -> (MultivariatePolynomial<F, E>, MultivariatePolynomial<F, E>) {
        let mut dividendpos = self.nterms - 1; // work from the back
        let norm = div.coefficients.last().unwrap();

        let mut q = self.new_from(Some(self.nterms));
        let mut r = self.new_from(None);

        // determine the variable
        let mut var = 0;
        for (i, x) in self.last_exponents().iter().enumerate() {
            if !x.is_zero() {
                var = i;
                break;
            }
        }

        let m = div.ldegree_max();
        let mut pow = self.ldegree_max();

        loop {
            // find the power in the dividend if it exists
            let mut coeff = loop {
                if self.exponents(dividendpos)[var] == pow {
                    break self.coefficients[dividendpos].clone();
                }
                if dividendpos == 0 || self.exponents(dividendpos)[var] < pow {
                    break F::zero();
                }
                dividendpos -= 1;
            };

            let mut qindex = 0; // starting from highest
            let mut bindex = 0; // starting from lowest
            while bindex < div.nterms && qindex < q.nterms {
                while bindex + 1 < div.nterms
                    && div.exponents(bindex)[var] + q.exponents(qindex)[var] < pow
                {
                    bindex += 1;
                }

                if div.exponents(bindex)[var] + q.exponents(qindex)[var] == pow {
                    self.field.add_assign(
                        &mut coeff,
                        &self.field.neg(
                            &self
                                .field
                                .mul(&div.coefficients[bindex], &q.coefficients[qindex]),
                        ),
                    );
                }

                qindex += 1;
            }

            if !F::is_zero(&coeff) {
                // can the division be performed? if not, add to rest

                let (quot, div) = if pow >= m {
                    let (quot, rem) = self.field.quot_rem(&coeff, &norm);
                    if F::is_zero(&rem) {
                        (quot, true)
                    } else {
                        (coeff, false)
                    }
                } else {
                    (coeff, false)
                };

                if div {
                    q.coefficients.push(quot);
                    q.exponents.resize((q.nterms + 1) * q.nvars, E::zero());
                    q.exponents[q.nterms * q.nvars + var] = pow - m;
                    q.nterms += 1;
                } else {
                    r.coefficients.push(quot);
                    r.exponents.resize((r.nterms + 1) * r.nvars, E::zero());
                    r.exponents[r.nterms * r.nvars + var] = pow;
                    r.nterms += 1;
                }
            }

            if pow.is_zero() {
                break;
            }

            pow = pow - E::from_u32(1);
        }

        q.reverse();
        r.reverse();

        #[cfg(debug_assertions)]
        {
            if !(&q * &div + r.clone() - self.clone()).is_zero() {
                panic!("Division failed: ({})/({}): q={}, r={}", self, div, q, r);
            }
        }

        (q, r)
    }

    /// Long division for multivarariate polynomial.
    /// If the ring `F` is not a field, and the coefficient does not cleanly divide,
    /// the division is stopped and the current quotient and rest term are returned.
    #[allow(dead_code)]
    fn long_division(
        &self,
        div: &MultivariatePolynomial<F, E>,
    ) -> (MultivariatePolynomial<F, E>, MultivariatePolynomial<F, E>) {
        if div.is_zero() {
            panic!("Cannot divide by 0 polynomial");
        }

        let mut q = self.new_from(None);
        let mut r = self.clone();
        let divdeg = div.last_exponents();

        while !r.is_zero()
            && r.last_exponents()
                .iter()
                .zip(divdeg.iter())
                .all(|(re, de)| re >= de)
        {
            let (tc, rem) = self.field.quot_rem(
                &r.coefficients.last().unwrap(),
                &div.coefficients.last().unwrap(),
            );

            if !F::is_zero(&rem) {
                // long division failed, return the term as the rest
                return (q, r);
            }

            let tp: Vec<E> = r
                .last_exponents()
                .iter()
                .zip(divdeg.iter())
                .map(|(e1, e2)| e1.clone() - e2.clone())
                .collect();

            q.append_monomial(tc.clone(), &tp);
            r = r - div.clone().mul_monomial(&tc, &tp);
        }

        (q, r)
    }

    /// Divide two multivariate polynomials and return the quotient and remainder.
    pub fn quot_rem(
        &self,
        div: &MultivariatePolynomial<F, E>,
    ) -> (MultivariatePolynomial<F, E>, MultivariatePolynomial<F, E>) {
        if div.is_zero() {
            panic!("Cannot divide by 0 polynomial");
        }

        if self.is_zero() {
            return (self.clone(), self.clone());
        }

        if div.is_one() {
            return (self.clone(), self.new_from(None));
        }

        if div.nterms == 1 {
            let mut q = self.new_from(Some(self.nterms));
            let dive = div.to_monomial_view(0);

            for t in self {
                let (quot, rem) = self.field.quot_rem(t.coefficient, dive.coefficient);
                if !F::is_zero(&rem)
                    || t.exponents
                        .iter()
                        .zip(dive.exponents)
                        .any(|(te, de)| te < de)
                {
                    // TODO: support upgrade to a RationalField
                    return (MultivariatePolynomial::new_from(&self, None), self.clone());
                }

                q.coefficients.push(quot);
                q.exponents.extend_from_slice(
                    &t.exponents
                        .iter()
                        .zip(dive.exponents)
                        .map(|(te, de)| *te - *de)
                        .collect::<SmallVec<[E; INLINED_EXPONENTS]>>(),
                );
                q.nterms += 1;
            }

            return (q, self.new_from(None));
        }

        // TODO: use other algorithm for univariate div
        self.heap_division(div)
    }

    /// Heap division for multivariate polynomials, using a cache so that only unique
    /// monomial exponents appear in the heap.
    /// Reference: "Sparse polynomial division using a heap" by Monagan, Pearce (2011)
    pub fn heap_division(
        &self,
        div: &MultivariatePolynomial<F, E>,
    ) -> (MultivariatePolynomial<F, E>, MultivariatePolynomial<F, E>) {
        let mut q = self.new_from(Some(self.nterms));
        let mut r = self.new_from(None);

        let mut div_monomial_in_heap = vec![false; div.nterms];
        let mut index_of_div_monomial_in_quotient = vec![0; div.nterms];

        let mut cache: HashMap<
            SmallVec<[E; INLINED_EXPONENTS]>,
            SmallVec<[(usize, usize, bool); 5]>,
        > = HashMap::with_capacity(self.nterms);

        let mut h: BinaryHeap<SmallVec<[E; INLINED_EXPONENTS]>> =
            BinaryHeap::with_capacity(self.nterms);

        let mut m: SmallVec<[E; INLINED_EXPONENTS]> = SmallVec::default();
        let mut c;

        let mut k = 0;
        while !h.is_empty() || k < self.nterms {
            m.clear();
            if k < self.nterms && (h.is_empty() || self.exponents_back(k) >= &h.peek().unwrap()) {
                m.extend_from_slice(self.exponents_back(k));
                c = self.coefficient_back(k).clone();
                k += 1;
            } else {
                m.extend_from_slice(h.peek().unwrap().as_slice());
                c = F::zero();
            }

            if let Some(monomial) = h.peek() {
                if &m == monomial {
                    h.pop().unwrap();

                    for (qi, gi, next_in_divisor) in cache.remove(&m).unwrap() {
                        // TODO: use fraction-free routines
                        self.field.sub_assign(
                            &mut c,
                            &self
                                .field
                                .mul(&q.coefficients[qi], &div.coefficient_back(gi)),
                        );

                        if next_in_divisor && gi + 1 < div.nterms {
                            // quotient heap product
                            let next_m: SmallVec<[E; INLINED_EXPONENTS]> = q
                                .exponents(qi)
                                .iter()
                                .zip(div.exponents_back(gi + 1))
                                .map(|(e1, e2)| *e1 + *e2)
                                .collect();

                            cache
                                .entry(next_m.clone())
                                .or_insert_with(|| {
                                    h.push(next_m); // only add when new
                                    smallvec![]
                                })
                                .push((qi, gi + 1, true));
                        } else if !next_in_divisor {
                            index_of_div_monomial_in_quotient[gi] = qi + 1;

                            if qi + 1 < q.nterms {
                                let next_m: SmallVec<[E; INLINED_EXPONENTS]> = q
                                    .exponents(qi + 1)
                                    .iter()
                                    .zip(div.exponents_back(gi))
                                    .map(|(e1, e2)| *e1 + *e2)
                                    .collect();

                                cache
                                    .entry(next_m.clone())
                                    .or_insert_with(|| {
                                        h.push(next_m);
                                        smallvec![]
                                    })
                                    .push((qi + 1, gi, false));
                            } else {
                                div_monomial_in_heap[gi] = false;
                            }

                            // modification from paper: also executed when qi + 1 = #q
                            if gi + 1 < div.nterms && !div_monomial_in_heap[gi + 1] {
                                let t = index_of_div_monomial_in_quotient[gi + 1];
                                if t < q.nterms {
                                    div_monomial_in_heap[gi + 1] = true; // fixed index in paper

                                    let next_elem: SmallVec<[E; INLINED_EXPONENTS]> = q
                                        .exponents(qi)
                                        .iter()
                                        .zip(div.exponents_back(gi + 1))
                                        .map(|(e1, e2)| *e1 + *e2)
                                        .collect();

                                    cache
                                        .entry(next_elem.clone())
                                        .or_insert_with(|| {
                                            h.push(next_elem);
                                            smallvec![]
                                        })
                                        .push((qi, gi + 1, false));
                                }
                            }
                        }
                    }
                }
            }

            if !F::is_zero(&c) && div.last_exponents().iter().zip(&m).all(|(ge, me)| me >= ge) {
                let (quot, rem) = self.field.quot_rem(&c, &div.lcoeff());
                if !F::is_zero(&rem) {
                    // TODO: support upgrade to a RationalField
                    return (MultivariatePolynomial::new_from(&self, None), self.clone());
                }

                q.coefficients.push(quot);
                q.exponents.extend_from_slice(
                    &div.last_exponents()
                        .iter()
                        .zip(&m)
                        .map(|(ge, me)| *me - *ge)
                        .collect::<SmallVec<[E; INLINED_EXPONENTS]>>(),
                );
                q.nterms += 1;

                if div.nterms == 1 {
                    continue;
                }

                let qn_g1: SmallVec<[E; INLINED_EXPONENTS]> = q
                    .last_exponents()
                    .iter()
                    .zip(div.exponents_back(1))
                    .map(|(e1, e2)| *e1 + *e2)
                    .collect();

                if q.nterms < div.nterms {
                    // using quotient heap
                    cache
                        .entry(qn_g1.clone())
                        .or_insert_with(|| {
                            h.push(qn_g1);
                            smallvec![]
                        })
                        .push((q.nterms - 1, 1, true));
                } else if q.nterms > div.nterms {
                    // using divisor heap
                    if !div_monomial_in_heap[1] {
                        div_monomial_in_heap[1] = true;

                        cache
                            .entry(qn_g1.clone())
                            .or_insert_with(|| {
                                h.push(qn_g1);
                                smallvec![]
                            })
                            .push((q.nterms - 1, 1, false));
                    }
                } else {
                    // switch to divisor heap
                    for index in &mut index_of_div_monomial_in_quotient {
                        *index = q.nterms - 1;
                    }
                    debug_assert!(div_monomial_in_heap.iter().any(|c| !c));
                    div_monomial_in_heap[1] = true;

                    cache
                        .entry(qn_g1.clone())
                        .or_insert_with(|| {
                            h.push(qn_g1);
                            smallvec![]
                        })
                        .push((q.nterms - 1, 1, false));
                }
            } else if !F::is_zero(&c) {
                r.coefficients.push(c);
                r.exponents.extend(&m);
                r.nterms += 1;
            }
        }

        // q and r have the highest monomials first
        q.reverse();
        r.reverse();

        #[cfg(debug_assertions)]
        {
            if !(&q * &div + r.clone() - self.clone()).is_zero() {
                panic!("Division failed: ({})/({}): q={}, r={}", self, div, q, r);
            }
        }

        (q, r)
    }
}

impl<UField, E: Exponent> MultivariatePolynomial<FiniteField<UField>, E>
where
    FiniteField<UField>: Field,
{
    /// Optimized division routine for the univariate case in a finite field.
    pub fn fast_divmod(
        &self,
        div: &mut MultivariatePolynomial<FiniteField<UField>, E>,
    ) -> (
        MultivariatePolynomial<FiniteField<UField>, E>,
        MultivariatePolynomial<FiniteField<UField>, E>,
    ) {
        if div.nterms == 1 {
            // calculate inverse once
            let inv = self.field.inv(&div.coefficients[0]);

            if div.is_constant() {
                let mut q = self.clone();
                for c in &mut q.coefficients {
                    self.field.mul_assign(c, &inv);
                }

                return (q, self.new_from(None));
            }

            let mut q = self.new_from(Some(self.nterms));
            let mut r = self.new_from(None);
            let dive = div.exponents(0);

            for m in self.into_iter() {
                if m.exponents.iter().zip(dive).all(|(a, b)| a >= b) {
                    q.coefficients.push(self.field.mul(m.coefficient, &inv));

                    for (ee, ed) in m.exponents.iter().zip(dive) {
                        q.exponents.push(*ee - *ed);
                    }
                    q.nterms += 1;
                } else {
                    r.coefficients.push(m.coefficient.clone());
                    r.exponents.extend(m.exponents);
                    r.nterms += 1;
                }
            }
            return (q, r);
        }

        // normalize the lcoeff to 1 to prevent a costly inversion
        if !self.field.is_one(&div.lcoeff()) {
            let o = div.lcoeff();
            let inv = self.field.inv(&div.lcoeff());

            for c in &mut div.coefficients {
                self.field.mul_assign(c, &inv);
            }

            let mut res = self.synthetic_division(div);

            for c in &mut res.0.coefficients {
                self.field.mul_assign(c, &o);
            }

            for c in &mut div.coefficients {
                self.field.mul_assign(c, &o);
            }
            return res;
        }

        // fall back to generic case
        self.synthetic_division(div)
    }
}

/// View object for a term in a multivariate polynomial.
#[derive(Copy, Clone, Debug)]
pub struct MonomialView<'a, F: 'a + Ring, E: 'a + Exponent> {
    pub coefficient: &'a F::Element,
    pub exponents: &'a [E],
}

/// Iterator over terms in a multivariate polynomial.
pub struct MonomialViewIterator<'a, F: Ring, E: Exponent> {
    poly: &'a MultivariatePolynomial<F, E>,
    index: usize,
}

impl<'a, F: Ring, E: Exponent> Iterator for MonomialViewIterator<'a, F, E> {
    type Item = MonomialView<'a, F, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.poly.nterms {
            None
        } else {
            let view = MonomialView {
                coefficient: &self.poly.coefficients[self.index],
                exponents: self.poly.exponents(self.index),
            };
            self.index += 1;
            Some(view)
        }
    }
}

impl<'a, F: Ring, E: Exponent> IntoIterator for &'a MultivariatePolynomial<F, E> {
    type Item = MonomialView<'a, F, E>;
    type IntoIter = MonomialViewIterator<'a, F, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            poly: self,
            index: 0,
        }
    }
}