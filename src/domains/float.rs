use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

use rand::Rng;
use wide::{f64x2, f64x4};

use super::rational::Rational;
use rug::{ops::CompleteRound, Float as MultiPrecisionFloat, Rational as MultiPrecisionRational};

pub trait NumericalFloatLike:
    PartialEq
    + Clone
    + std::fmt::Debug
    + std::fmt::Display
    + std::ops::Neg<Output = Self>
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Div<&'a Self, Output = Self>
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
    + for<'a> DivAssign<&'a Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<Self>
    + DivAssign<Self>
{
    fn mul_add(&self, a: &Self, a: &Self) -> Self;
    fn neg(&self) -> Self;
    fn norm(&self) -> Self;
    fn zero(&self) -> Self;
    /// Create a zero that should only be used as a temporary value,
    /// as for some types it may have wrong precision information.
    fn new_zero() -> Self;
    fn one(&self) -> Self;
    fn pow(&self, e: u64) -> Self;
    fn inv(&self) -> Self;

    fn from_usize(&self, a: usize) -> Self;
    fn from_i64(&self, a: i64) -> Self;

    /// Sample a point on the interval [0, 1].
    fn sample_unit<R: Rng + ?Sized>(&self, rng: &mut R) -> Self;
}

pub trait NumericalFloatComparison: NumericalFloatLike + PartialOrd {
    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;
    fn is_finite(&self) -> bool;
    fn max(&self, other: &Self) -> Self;

    fn to_usize_clamped(&self) -> usize;
    fn to_f64(&self) -> f64;
}

/// A float that can be constructed without any parameters, such as f64.
pub trait ConstructibleFloat: NumericalFloatLike {
    fn new_one() -> Self;
    fn new_from_usize(a: usize) -> Self;
    fn new_from_i64(a: i64) -> Self;
    /// Sample a point on the interval [0, 1].
    fn new_sample_unit<R: Rng + ?Sized>(rng: &mut R) -> Self;
}

pub trait Real: NumericalFloatLike {
    fn sqrt(&self) -> Self;
    fn log(&self) -> Self;
    fn exp(&self) -> Self;
    fn sin(&self) -> Self;
    fn cos(&self) -> Self;
    fn tan(&self) -> Self;
    fn asin(&self) -> Self;
    fn acos(&self) -> Self;
    fn atan2(&self, x: &Self) -> Self;
    fn sinh(&self) -> Self;
    fn cosh(&self) -> Self;
    fn tanh(&self) -> Self;
    fn asinh(&self) -> Self;
    fn acosh(&self) -> Self;
    fn atanh(&self) -> Self;
    fn powf(&self, e: Self) -> Self;
}

impl NumericalFloatLike for f64 {
    #[inline(always)]
    fn mul_add(&self, a: &Self, b: &Self) -> Self {
        f64::mul_add(*self, *a, *b)
    }

    #[inline(always)]
    fn neg(&self) -> Self {
        -self
    }

    #[inline(always)]
    fn norm(&self) -> Self {
        f64::abs(*self)
    }

    #[inline(always)]
    fn zero(&self) -> Self {
        0.
    }

    #[inline(always)]
    fn new_zero() -> Self {
        0.
    }

    #[inline(always)]
    fn one(&self) -> Self {
        1.
    }

    #[inline]
    fn pow(&self, e: u64) -> Self {
        // FIXME: use binary exponentiation
        debug_assert!(e <= i32::MAX as u64);
        self.powi(e as i32)
    }

    #[inline(always)]
    fn inv(&self) -> Self {
        1. / self
    }

    #[inline(always)]
    fn from_usize(&self, a: usize) -> Self {
        a as f64
    }

    #[inline(always)]
    fn from_i64(&self, a: i64) -> Self {
        a as f64
    }

    fn sample_unit<R: Rng + ?Sized>(&self, rng: &mut R) -> Self {
        rng.gen()
    }
}

impl NumericalFloatComparison for f64 {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0.
    }

    #[inline(always)]
    fn is_one(&self) -> bool {
        *self == 1.
    }

    #[inline(always)]
    fn is_finite(&self) -> bool {
        (*self).is_finite()
    }

    fn max(&self, other: &Self) -> Self {
        (*self).max(*other)
    }

    fn to_usize_clamped(&self) -> usize {
        *self as usize
    }

    fn to_f64(&self) -> f64 {
        *self
    }
}

impl ConstructibleFloat for f64 {
    fn new_one() -> Self {
        1.
    }

    fn new_from_usize(a: usize) -> Self {
        a as f64
    }

    fn new_from_i64(a: i64) -> Self {
        a as f64
    }

    fn new_sample_unit<R: Rng + ?Sized>(rng: &mut R) -> Self {
        rng.gen()
    }
}

impl Real for f64 {
    #[inline(always)]
    fn sqrt(&self) -> Self {
        (*self).sqrt()
    }

    #[inline(always)]
    fn log(&self) -> Self {
        (*self).ln()
    }

    #[inline(always)]
    fn exp(&self) -> Self {
        (*self).exp()
    }

    #[inline(always)]
    fn sin(&self) -> Self {
        (*self).sin()
    }

    #[inline(always)]
    fn cos(&self) -> Self {
        (*self).cos()
    }

    #[inline(always)]
    fn tan(&self) -> Self {
        (*self).tan()
    }

    #[inline(always)]
    fn asin(&self) -> Self {
        (*self).asin()
    }

    #[inline(always)]
    fn acos(&self) -> Self {
        (*self).acos()
    }

    #[inline(always)]
    fn atan2(&self, x: &Self) -> Self {
        (*self).atan2(*x)
    }

    #[inline(always)]
    fn sinh(&self) -> Self {
        (*self).sinh()
    }

    #[inline(always)]
    fn cosh(&self) -> Self {
        (*self).cosh()
    }

    #[inline(always)]
    fn tanh(&self) -> Self {
        (*self).tanh()
    }

    #[inline(always)]
    fn asinh(&self) -> Self {
        (*self).asinh()
    }

    #[inline(always)]
    fn acosh(&self) -> Self {
        (*self).acosh()
    }

    #[inline(always)]
    fn atanh(&self) -> Self {
        (*self).atanh()
    }

    #[inline]
    fn powf(&self, e: f64) -> Self {
        (*self).powf(e)
    }
}

impl From<&Rational> for f64 {
    fn from(value: &Rational) -> Self {
        match value {
            Rational::Natural(n, d) => *n as f64 / *d as f64,
            Rational::Large(l) => l.to_f64(),
        }
    }
}

impl From<Rational> for f64 {
    fn from(value: Rational) -> Self {
        match value {
            Rational::Natural(n, d) => n as f64 / d as f64,
            Rational::Large(l) => l.to_f64(),
        }
    }
}

impl NumericalFloatLike for MultiPrecisionFloat {
    #[inline(always)]
    fn mul_add(&self, a: &Self, b: &Self) -> Self {
        self.mul_add_ref(a, b).complete(self.prec())
    }

    #[inline(always)]
    fn neg(&self) -> Self {
        -self.clone()
    }

    #[inline(always)]
    fn norm(&self) -> Self {
        self.clone().abs()
    }

    #[inline(always)]
    fn zero(&self) -> Self {
        MultiPrecisionFloat::new(self.prec())
    }

    #[inline(always)]
    fn new_zero() -> Self {
        MultiPrecisionFloat::new(1)
    }

    #[inline(always)]
    fn one(&self) -> Self {
        MultiPrecisionFloat::with_val(self.prec(), 1.)
    }

    #[inline]
    fn pow(&self, e: u64) -> Self {
        rug::ops::Pow::pow(self, e).complete(self.prec())
    }

    #[inline(always)]
    fn inv(&self) -> Self {
        self.clone().recip()
    }

    #[inline(always)]
    fn from_usize(&self, a: usize) -> Self {
        MultiPrecisionFloat::with_val(self.prec(), a)
    }

    #[inline(always)]
    fn from_i64(&self, a: i64) -> Self {
        MultiPrecisionFloat::with_val(self.prec(), a)
    }

    fn sample_unit<R: Rng + ?Sized>(&self, rng: &mut R) -> Self {
        let f: f64 = rng.gen();
        MultiPrecisionFloat::with_val(self.prec(), f)
    }
}

impl NumericalFloatComparison for MultiPrecisionFloat {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0.
    }

    #[inline(always)]
    fn is_one(&self) -> bool {
        *self == 1.
    }

    #[inline(always)]
    fn is_finite(&self) -> bool {
        (*self).is_finite()
    }

    fn max(&self, other: &Self) -> Self {
        (self).max_ref(other).complete(self.prec())
    }

    fn to_usize_clamped(&self) -> usize {
        self.to_integer().unwrap().to_usize().unwrap_or(usize::MAX)
    }

    fn to_f64(&self) -> f64 {
        self.to_f64()
    }
}

impl Real for MultiPrecisionFloat {
    #[inline(always)]
    fn sqrt(&self) -> Self {
        self.clone().sqrt()
    }

    #[inline(always)]
    fn log(&self) -> Self {
        self.clone().ln()
    }

    #[inline(always)]
    fn exp(&self) -> Self {
        self.clone().exp()
    }

    #[inline(always)]
    fn sin(&self) -> Self {
        self.clone().sin()
    }

    #[inline(always)]
    fn cos(&self) -> Self {
        self.clone().cos()
    }

    #[inline(always)]
    fn tan(&self) -> Self {
        self.clone().tan()
    }

    #[inline(always)]
    fn asin(&self) -> Self {
        self.clone().asin()
    }

    #[inline(always)]
    fn acos(&self) -> Self {
        self.clone().acos()
    }

    #[inline(always)]
    fn atan2(&self, x: &Self) -> Self {
        self.clone().atan2(x)
    }

    #[inline(always)]
    fn sinh(&self) -> Self {
        self.clone().sinh()
    }

    #[inline(always)]
    fn cosh(&self) -> Self {
        self.clone().cosh()
    }

    #[inline(always)]
    fn tanh(&self) -> Self {
        self.clone().tanh()
    }

    #[inline(always)]
    fn asinh(&self) -> Self {
        self.clone().asinh()
    }

    #[inline(always)]
    fn acosh(&self) -> Self {
        self.clone().acosh()
    }

    #[inline(always)]
    fn atanh(&self) -> Self {
        self.clone().atanh()
    }

    #[inline]
    fn powf(&self, e: Self) -> Self {
        rug::ops::Pow::pow(self, e)
    }
}

impl Rational {
    // Convert the rational number to a multi-precision float with precision `prec`.
    pub fn to_multi_prec_float(&self, prec: u32) -> MultiPrecisionFloat {
        MultiPrecisionFloat::with_val(
            prec,
            rug::Rational::from((
                self.numerator().to_multi_prec(),
                self.denominator().to_multi_prec(),
            )),
        )
    }
}

macro_rules! simd_impl {
    ($t:ty, $p:ident) => {
        impl NumericalFloatLike for $t {
            #[inline(always)]
            fn mul_add(&self, a: &Self, b: &Self) -> Self {
                *self * *a + b
            }

            #[inline(always)]
            fn neg(&self) -> Self {
                -self
            }

            #[inline(always)]
            fn norm(&self) -> Self {
                (*self).abs()
            }

            #[inline(always)]
            fn zero(&self) -> Self {
                Self::ZERO
            }

            #[inline(always)]
            fn new_zero() -> Self {
                Self::ZERO
            }

            #[inline(always)]
            fn one(&self) -> Self {
                Self::ONE
            }

            #[inline]
            fn pow(&self, e: u64) -> Self {
                // FIXME: use binary exponentiation
                debug_assert!(e <= i32::MAX as u64);
                (*self).powf(e as f64)
            }

            #[inline(always)]
            fn inv(&self) -> Self {
                Self::ONE / *self
            }

            #[inline(always)]
            fn from_usize(&self, a: usize) -> Self {
                Self::from(a as f64)
            }

            #[inline(always)]
            fn from_i64(&self, a: i64) -> Self {
                Self::from(a as f64)
            }

            fn sample_unit<R: Rng + ?Sized>(&self, rng: &mut R) -> Self {
                Self::from(rng.gen::<f64>())
            }
        }

        impl Real for $t {
            #[inline(always)]
            fn sqrt(&self) -> Self {
                (*self).sqrt()
            }

            #[inline(always)]
            fn log(&self) -> Self {
                (*self).ln()
            }

            #[inline(always)]
            fn exp(&self) -> Self {
                (*self).exp()
            }

            #[inline(always)]
            fn sin(&self) -> Self {
                (*self).sin()
            }

            #[inline(always)]
            fn cos(&self) -> Self {
                (*self).cos()
            }

            #[inline(always)]
            fn tan(&self) -> Self {
                (*self).tan()
            }

            #[inline(always)]
            fn asin(&self) -> Self {
                (*self).asin()
            }

            #[inline(always)]
            fn acos(&self) -> Self {
                (*self).acos()
            }

            #[inline(always)]
            fn atan2(&self, x: &Self) -> Self {
                (*self).atan2(*x)
            }

            #[inline(always)]
            fn sinh(&self) -> Self {
                unimplemented!("Hyperbolic geometric functions are not supported with SIMD");
            }

            #[inline(always)]
            fn cosh(&self) -> Self {
                unimplemented!("Hyperbolic geometric functions are not supported with SIMD");
            }

            #[inline(always)]
            fn tanh(&self) -> Self {
                unimplemented!("Hyperbolic geometric functions are not supported with SIMD");
            }

            #[inline(always)]
            fn asinh(&self) -> Self {
                unimplemented!("Hyperbolic geometric functions are not supported with SIMD");
            }

            #[inline(always)]
            fn acosh(&self) -> Self {
                unimplemented!("Hyperbolic geometric functions are not supported with SIMD");
            }

            #[inline(always)]
            fn atanh(&self) -> Self {
                unimplemented!("Hyperbolic geometric functions are not supported with SIMD");
            }

            #[inline(always)]
            fn powf(&self, e: Self) -> Self {
                (*self).$p(e)
            }
        }

        impl From<&Rational> for $t {
            fn from(value: &Rational) -> Self {
                match value {
                    Rational::Natural(n, d) => Self::from(*n as f64 / *d as f64),
                    Rational::Large(l) => Self::from(l.to_f64()),
                }
            }
        }
    };
}

simd_impl!(f64x2, pow_f64x2);
simd_impl!(f64x4, pow_f64x4);

impl NumericalFloatLike for Rational {
    fn mul_add(&self, a: &Self, c: &Self) -> Self {
        &(self * a) + c
    }

    fn neg(&self) -> Self {
        self.neg()
    }

    fn norm(&self) -> Self {
        self.abs()
    }

    fn zero(&self) -> Self {
        Self::zero()
    }

    fn new_zero() -> Self {
        Self::zero()
    }

    fn one(&self) -> Self {
        Self::one()
    }

    fn pow(&self, e: u64) -> Self {
        self.pow(e)
    }

    fn inv(&self) -> Self {
        self.inv()
    }

    fn from_usize(&self, a: usize) -> Self {
        if a < i64::MAX as usize {
            Rational::Natural(a as i64, 1)
        } else {
            Rational::Large(MultiPrecisionRational::from(a))
        }
    }

    fn from_i64(&self, a: i64) -> Self {
        Rational::Natural(a, 1)
    }

    fn sample_unit<R: Rng + ?Sized>(&self, rng: &mut R) -> Self {
        let rng1 = rng.gen::<i64>();
        let rng2 = rng.gen::<i64>();

        if rng1 > rng2 {
            Rational::Natural(rng2, rng1)
        } else {
            Rational::Natural(rng1, rng2)
        }
    }
}

impl NumericalFloatComparison for Rational {
    #[inline(always)]
    fn is_zero(&self) -> bool {
        self.is_zero()
    }

    #[inline(always)]
    fn is_one(&self) -> bool {
        self.is_one()
    }

    #[inline(always)]
    fn is_finite(&self) -> bool {
        true
    }

    fn max(&self, other: &Self) -> Self {
        if self > other {
            self.clone()
        } else {
            other.clone()
        }
    }

    fn to_usize_clamped(&self) -> usize {
        f64::from(self).to_usize_clamped()
    }

    fn to_f64(&self) -> f64 {
        f64::from(self)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Complex<T: Real> {
    pub re: T,
    pub im: T,
}

impl<T: Real> Complex<T> {
    #[inline]
    pub fn new(re: T, im: T) -> Complex<T> {
        Complex { re, im }
    }

    #[inline]
    pub fn i(&self) -> Complex<T> {
        Complex {
            re: self.re.zero(),
            im: self.im.one(),
        }
    }

    #[inline]
    pub fn norm_squared(&self) -> T {
        self.re.clone() * &self.re + self.im.clone() * &self.im
    }

    #[inline]
    pub fn arg(&self) -> T {
        self.im.atan2(&self.re)
    }

    #[inline]
    pub fn to_polar_coordinates(self) -> (T, T) {
        (self.norm_squared().sqrt(), self.arg())
    }

    #[inline]
    pub fn from_polar_coordinates(r: T, phi: T) -> Complex<T> {
        Complex::new(r.clone() * phi.cos(), r.clone() * phi.sin())
    }
}

impl<T: Real> Add<Complex<T>> for Complex<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Complex::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl<T: Real> Add<&Complex<T>> for Complex<T> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: &Self) -> Self::Output {
        Complex::new(self.re + &rhs.re, self.im + &rhs.im)
    }
}

impl<'a, 'b, T: Real> Add<&'a Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn add(self, rhs: &'a Complex<T>) -> Self::Output {
        self.clone() + rhs
    }
}

impl<'b, T: Real> Add<Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn add(self, rhs: Complex<T>) -> Self::Output {
        self.clone() + rhs
    }
}

impl<T: Real> AddAssign for Complex<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.add_assign(&rhs)
    }
}

impl<T: Real> AddAssign<&Complex<T>> for Complex<T> {
    #[inline]
    fn add_assign(&mut self, rhs: &Self) {
        self.re += &rhs.re;
        self.im += &rhs.im;
    }
}

impl<T: Real> Sub for Complex<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Complex::new(self.re - rhs.re, self.im - rhs.im)
    }
}

impl<T: Real> Sub<&Complex<T>> for Complex<T> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: &Self) -> Self::Output {
        Complex::new(self.re - &rhs.re, self.im - &rhs.im)
    }
}

impl<'a, 'b, T: Real> Sub<&'a Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn sub(self, rhs: &'a Complex<T>) -> Self::Output {
        self.clone() - rhs
    }
}

impl<'b, T: Real> Sub<Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn sub(self, rhs: Complex<T>) -> Self::Output {
        self.clone() - rhs
    }
}

impl<T: Real> SubAssign for Complex<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.sub_assign(&rhs)
    }
}

impl<T: Real> SubAssign<&Complex<T>> for Complex<T> {
    #[inline]
    fn sub_assign(&mut self, rhs: &Self) {
        self.re -= &rhs.re;
        self.im -= &rhs.im;
    }
}

impl<T: Real> Mul for Complex<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(&rhs)
    }
}

impl<T: Real> Mul<&Complex<T>> for Complex<T> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: &Self) -> Self::Output {
        Complex::new(
            self.re.clone() * &rhs.re - self.im.clone() * &rhs.im,
            self.re.clone() * &rhs.im + self.im.clone() * &rhs.re,
        )
    }
}

impl<'a, 'b, T: Real> Mul<&'a Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn mul(self, rhs: &'a Complex<T>) -> Self::Output {
        self.clone() * rhs
    }
}

impl<'b, T: Real> Mul<Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn mul(self, rhs: Complex<T>) -> Self::Output {
        self.clone() * rhs
    }
}

impl<T: Real> MulAssign for Complex<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone().mul(rhs);
    }
}

impl<T: Real> MulAssign<&Complex<T>> for Complex<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: &Self) {
        *self = self.clone().mul(rhs);
    }
}

impl<T: Real> Div for Complex<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.div(&rhs)
    }
}

impl<T: Real> Div<&Complex<T>> for Complex<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: &Self) -> Self::Output {
        let n = rhs.norm_squared();
        let re = self.re.clone() * &rhs.re + self.im.clone() * &rhs.im;
        let im = self.im.clone() * &rhs.re - self.re.clone() * &rhs.im;
        Complex::new(re / &n, im / &n)
    }
}

impl<'a, 'b, T: Real> Div<&'a Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn div(self, rhs: &'a Complex<T>) -> Self::Output {
        self.clone() / rhs
    }
}

impl<'b, T: Real> Div<Complex<T>> for &'b Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn div(self, rhs: Complex<T>) -> Self::Output {
        self.clone() / rhs
    }
}

impl<T: Real> DivAssign for Complex<T> {
    fn div_assign(&mut self, rhs: Self) {
        *self = self.clone().div(rhs);
    }
}

impl<T: Real> DivAssign<&Complex<T>> for Complex<T> {
    fn div_assign(&mut self, rhs: &Self) {
        *self = self.clone().div(rhs);
    }
}

impl<T: Real> Neg for Complex<T> {
    type Output = Complex<T>;

    #[inline]
    fn neg(self) -> Complex<T> {
        Complex::new(-self.re, -self.im)
    }
}

impl<T: Real> Display for Complex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}+{}i)", self.re, self.im))
    }
}

impl<T: Real> std::fmt::Debug for Complex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({:?}+{:?}i)", self.re, self.im))
    }
}

impl<T: Real> NumericalFloatLike for Complex<T> {
    #[inline]
    fn mul_add(&self, a: &Self, b: &Self) -> Self {
        self.clone() + (a.clone() * b)
    }

    #[inline]
    fn neg(&self) -> Self {
        Complex {
            re: -self.re.clone(),
            im: -self.im.clone(),
        }
    }

    #[inline]
    fn norm(&self) -> Self {
        Complex::new(self.norm_squared().sqrt(), self.im.zero())
    }

    #[inline]
    fn zero(&self) -> Self {
        Complex {
            re: self.re.zero(),
            im: self.im.zero(),
        }
    }

    fn new_zero() -> Self {
        Complex {
            re: T::new_zero(),
            im: T::new_zero(),
        }
    }

    fn one(&self) -> Self {
        Complex {
            re: self.re.one(),
            im: self.im.zero(),
        }
    }

    fn pow(&self, e: u64) -> Self {
        // TODO: use binary exponentiation
        let mut r = self.one();
        for _ in 0..e {
            r *= self;
        }
        r
    }

    fn inv(&self) -> Self {
        let n = self.norm_squared();
        Complex::new(self.re.clone() / &n, -self.im.clone() / &n)
    }

    fn from_usize(&self, a: usize) -> Self {
        Complex {
            re: self.re.from_usize(a),
            im: self.im.zero(),
        }
    }

    fn from_i64(&self, a: i64) -> Self {
        Complex {
            re: self.re.from_i64(a),
            im: self.im.zero(),
        }
    }

    fn sample_unit<R: Rng + ?Sized>(&self, rng: &mut R) -> Self {
        Complex {
            re: self.re.sample_unit(rng),
            im: self.im.zero(),
        }
    }
}

/// Following the same conventions and formulas as num::Complex.
impl<T: Real> Real for Complex<T> {
    #[inline]
    fn sqrt(&self) -> Self {
        let (r, phi) = self.clone().to_polar_coordinates();
        Complex::from_polar_coordinates(r.sqrt(), phi / self.re.from_usize(2))
    }

    #[inline]
    fn log(&self) -> Self {
        Complex::new(self.norm().re.log(), self.arg())
    }

    #[inline]
    fn exp(&self) -> Self {
        let r = self.re.exp();
        Complex::new(r.clone() * self.im.cos(), r * self.im.sin())
    }

    #[inline]
    fn sin(&self) -> Self {
        Complex::new(
            self.re.sin() * self.im.cosh(),
            self.re.cos() * self.im.sinh(),
        )
    }

    #[inline]
    fn cos(&self) -> Self {
        Complex::new(
            self.re.cos() * self.im.cosh(),
            -self.re.sin() * self.im.sinh(),
        )
    }

    #[inline]
    fn tan(&self) -> Self {
        let (r, i) = (self.re.clone() + &self.re, self.im.clone() + &self.im);
        let m = r.cos() + i.cosh();
        Self::new(r.sin() / &m, i.sinh() / m)
    }

    #[inline]
    fn asin(&self) -> Self {
        let i = self.i();
        -i.clone() * ((self.one() - self.clone() * self).sqrt() + i * self).log()
    }

    #[inline]
    fn acos(&self) -> Self {
        let i = self.i();
        -i.clone() * (i * (self.one() - self.clone() * self).sqrt() + self).log()
    }

    #[inline]
    fn atan2(&self, x: &Self) -> Self {
        // TODO: pick proper branch
        let r = self.clone() / x;
        let i = self.i();
        let one = self.one();
        let two = one.clone() + &one;
        // TODO: add edge cases
        ((&one + &i * &r).log() - (&one - &i * r).log()) / (two * i)
    }

    #[inline]
    fn sinh(&self) -> Self {
        Complex::new(
            self.re.sinh() * self.im.cos(),
            self.re.cosh() * self.im.sin(),
        )
    }

    #[inline]
    fn cosh(&self) -> Self {
        Complex::new(
            self.re.cosh() * self.im.cos(),
            self.re.sinh() * self.im.sin(),
        )
    }

    #[inline]
    fn tanh(&self) -> Self {
        let (two_re, two_im) = (self.re.clone() + &self.re, self.im.clone() + &self.im);
        let m = two_re.cosh() + two_im.cos();
        Self::new(two_re.sinh() / &m, two_im.sin() / m)
    }

    #[inline]
    fn asinh(&self) -> Self {
        let one = self.one();
        (self.clone() + (one + self.clone() * self).sqrt()).log()
    }

    #[inline]
    fn acosh(&self) -> Self {
        let one = self.one();
        let two = one.clone() + &one;
        &two * (((self.clone() + &one) / &two).sqrt() + ((self.clone() - one) / &two).sqrt()).log()
    }

    #[inline]
    fn atanh(&self) -> Self {
        let one = self.one();
        let two = one.clone() + &one;
        // TODO: add edge cases
        ((&one + self).log() - (one - self).log()) / two
    }

    #[inline]
    fn powf(&self, e: Self) -> Self {
        if e.re == self.re.zero() && e.im == self.im.zero() {
            return self.one();
        } else if e.im == self.im.zero() {
            let (r, phi) = self.clone().to_polar_coordinates();
            Self::from_polar_coordinates(r.powf(e.re.clone()), phi * e.re)
        } else {
            (e * self.log()).exp()
        }
    }
}

impl<'a, T: Real + From<&'a Rational>> From<&'a Rational> for Complex<T> {
    fn from(value: &'a Rational) -> Self {
        let c: T = value.into();
        let zero = c.zero();
        Complex::new(c, zero)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn double() {
        let a = 5.;
        let b = 7.;

        let r = a.sqrt() + b.log() + b.sin() - a.cos() + b.tan() - 0.3.asin() + 0.5.acos()
            - a.atan2(b)
            + b.sinh()
            - a.cosh()
            + b.tanh()
            - 0.7.asinh()
            + b.acosh() / 0.4.atanh()
            + b.powf(a);
        assert_eq!(r, 17293.219725825093);
    }

    #[test]
    fn complex() {
        let a = Complex::new(1., 2.);
        let b: Complex<f64> = Complex::new(3., 4.);

        let r = &a.sqrt() + &b.log() - a.exp() + b.sin() - a.cos() + b.tan() - a.asin() + b.acos()
            - a.atan2(&b)
            + b.sinh()
            - a.cosh()
            + b.tanh()
            - a.asinh()
            + &b.acosh() / a.atanh()
            + b.powf(a);
        assert_eq!(r, Complex::new(0.1924131450685842, -39.83285329561913));
    }
}
