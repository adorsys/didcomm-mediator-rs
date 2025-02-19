extern crate alloc;
use alloc::boxed::Box;

use core::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use base64ct::Base64UrlUnpadded;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, Zeroizing};

use super::Bytes;

/// A serde wrapper for base64-encoded secrets.
///
/// A secret is like the [`Bytes`] type, with some additional protections:
///
///   1. It is zeroed on drop.
///   2. Its equality implementation is constant time.
///   3. Its contents are not printed in the debug formatter.
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
#[serde(bound(serialize = "Bytes<T, E>: Serialize"))]
#[serde(bound(deserialize = "Bytes<T, E>: Deserialize<'de>"))]
pub struct Secret<T: Zeroize = Box<[u8]>, E = Base64UrlUnpadded>(Zeroizing<Bytes<T, E>>);

impl<T: Zeroize, E> Debug for Secret<T, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Secret(***)")
    }
}

impl<T: Zeroize, U: Into<Bytes<T, E>>, E> From<U> for Secret<T, E> {
    fn from(value: U) -> Self {
        Self(Zeroizing::new(value.into()))
    }
}

impl<T: Zeroize, E> Deref for Secret<T, E> {
    type Target = Bytes<T, E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Zeroize, E> DerefMut for Secret<T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Zeroize, U, E> AsRef<U> for Secret<T, E>
where
    Bytes<T, E>: AsRef<U>,
{
    fn as_ref(&self) -> &U {
        self.0.as_ref()
    }
}

impl<T: Zeroize, U, E> AsMut<U> for Secret<T, E>
where
    Bytes<T, E>: AsMut<U>,
{
    fn as_mut(&mut self) -> &mut U {
        self.0.as_mut()
    }
}

impl<T: Zeroize + AsRef<[u8]> + Sized, E> ConstantTimeEq for Secret<T, E> {
    fn ct_eq(&self, other: &Self) -> subtle::Choice {
        self.0.as_ref().ct_eq(other.0.as_ref())
    }
}

impl<T: Zeroize + AsRef<[u8]>, E> Eq for Secret<T, E> {}
impl<T: Zeroize + AsRef<[u8]>, E> PartialEq for Secret<T, E> {
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).unwrap_u8() == 1
    }
}

impl<T: Zeroize + Default, E> Default for Secret<T, E> {
    fn default() -> Self {
        Self(Zeroizing::new(Bytes::default()))
    }
}
