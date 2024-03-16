// LICENSE: Derivative work of rust std library, apache2 + MIT

use std::{borrow::Borrow, fmt, ops::Deref};

use self::Cow::*;

pub trait ToOwned {
    type Owned: Borrow<Self>;

    // Required method
    fn to_owned(&self) -> Self::Owned;

    // Provided method
    fn clone_into(&self, target: &mut Self::Owned);
}

impl<T: Clone> ToOwned for [T] {
    type Owned = Vec<T>;
    fn to_owned(&self) -> Vec<T> {
        self.to_vec()
    }

    fn clone_into(&self, target: &mut Self::Owned) {
        *target = <Self as ToOwned>::to_owned(self);
    }
}

impl ToOwned for str {
    type Owned = String;
    #[inline]
    fn to_owned(&self) -> String {
        unsafe { String::from_utf8_unchecked(<[u8] as ToOwned>::to_owned(self.as_bytes())) }
    }

    fn clone_into(&self, target: &mut String) {
        let mut b = std::mem::take(target).into_bytes();
        <[u8] as ToOwned>::clone_into(self.as_bytes(), &mut b);
        *target = unsafe { String::from_utf8_unchecked(b) }
    }
}

impl<B, O> fmt::Debug for Cow<'_, B>
where
    B: ?Sized + fmt::Debug + ToOwned<Owned = O>,
    O: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Cow::Borrowed(ref b) => fmt::Debug::fmt(b, f),
            Cow::Owned(ref o) => fmt::Debug::fmt(o, f),
        }
    }
}

impl<B> PartialEq for Cow<'_, B>
where
    B: ?Sized + PartialEq + ToOwned,
{
    fn eq(&self, other: &Self) -> bool {
        let borrow_0 = match self {
            Cow::Borrowed(b) => *b,
            Cow::Owned(o) => o.borrow(),
        };
        let borrow_1 = match other {
            Cow::Borrowed(b) => *b,
            Cow::Owned(o) => o.borrow(),
        };
        borrow_0 == borrow_1
    }
}

impl<B> Eq for Cow<'_, B> where B: ?Sized + Eq + ToOwned {}

impl<B: ?Sized + ToOwned> Deref for Cow<'_, B>
where
    B::Owned: Borrow<B>,
{
    type Target = B;

    fn deref(&self) -> &B {
        match *self {
            Borrowed(borrowed) => borrowed,
            Owned(ref owned) => owned.borrow(),
        }
    }
}

impl<T: ?Sized + ToOwned> AsRef<T> for Cow<'_, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

pub enum Cow<'a, B>
where
    B: ToOwned + ?Sized,
{
    Borrowed(&'a B),
    #[allow(unused)]
    Owned(<B as ToOwned>::Owned),
}

impl<'a, B> Clone for Cow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    fn clone(&self) -> Self {
        match self {
            Borrowed(arg0) => Borrowed(arg0),
            // TODO: CHeck if this is how std did this?
            Owned(arg0) => Owned(arg0.borrow().to_owned()),
        }
    }
}

#[test]
fn test_partial_eq() {
    assert_eq!(Cow::Owned("foo".into()), Cow::Borrowed("foo"));
}
