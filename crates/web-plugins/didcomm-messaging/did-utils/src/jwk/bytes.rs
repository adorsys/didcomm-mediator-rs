extern crate alloc;

use zeroize::{Zeroize, Zeroizing};

use core::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use base64ct::{Base64UrlUnpadded, Encoding, Error as DecodeError};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// A serde wrapper for base64-encoded bytes.
///
/// # Type Parameters
///
/// - `T`: The type used to store the byte data (e.g., `Vec<u8>`, `Box<[u8]>`).
/// - `E`: The base64 encoding type (e.g., `Base64UrlUnpadded`).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes<T = Box<[u8]>, E = Base64UrlUnpadded> {
    buf: T,
    cfg: PhantomData<E>,
}

impl<T: Zeroize, E> Zeroize for Bytes<T, E> {
    fn zeroize(&mut self) {
        self.buf.zeroize()
    }
}

impl<T: Debug, E> Debug for Bytes<T, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Bytes").field(&self.buf).finish()
    }
}

impl<T, E> Deref for Bytes<T, E> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<T, E> DerefMut for Bytes<T, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl<T: AsRef<U>, U: ?Sized, E> AsRef<U> for Bytes<T, E> {
    fn as_ref(&self) -> &U {
        self.buf.as_ref()
    }
}

impl<T: AsMut<U>, U: ?Sized, E> AsMut<U> for Bytes<T, E> {
    fn as_mut(&mut self) -> &mut U {
        self.buf.as_mut()
    }
}

impl<T, E> From<T> for Bytes<T, E> {
    fn from(buf: T) -> Self {
        Self { buf, cfg: PhantomData }
    }
}

impl<E> From<Vec<u8>> for Bytes<Box<[u8]>, E> {
    fn from(buf: Vec<u8>) -> Self {
        Self::from(buf.into_boxed_slice())
    }
}

impl<E> From<Bytes<Vec<u8>, E>> for Bytes<Box<[u8]>, E> {
    fn from(bytes: Bytes<Vec<u8>, E>) -> Self {
        Self::from(bytes.buf.into_boxed_slice())
    }
}

impl<E> From<Box<[u8]>> for Bytes<Vec<u8>, E> {
    fn from(buf: Box<[u8]>) -> Self {
        Self::from(buf.into_vec())
    }
}

impl<E> From<Bytes<Box<[u8]>, E>> for Bytes<Vec<u8>, E> {
    fn from(bytes: Bytes<Box<[u8]>, E>) -> Self {
        Self::from(bytes.buf.into_vec())
    }
}

impl<E: Encoding> FromStr for Bytes<Vec<u8>, E> {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            buf: E::decode_vec(s)?,
            cfg: PhantomData,
        })
    }
}
impl<E: Encoding> FromStr for Bytes<Box<[u8]>, E> {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Bytes::<Vec<u8>, E>::from_str(s).map(|x| x.buf.into_boxed_slice().into())
    }
}

impl<T: AsRef<[u8]>, E: Encoding> Serialize for Bytes<T, E> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let b64 = Zeroizing::from(E::encode_string(self.buf.as_ref()));
        b64.serialize(serializer)
    }
}

impl<'de, E: Encoding> Deserialize<'de> for Bytes<Vec<u8>, E> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let enc = Zeroizing::from(String::deserialize(deserializer)?);
        let dec = E::decode_vec(&enc).map_err(|_| D::Error::custom("invalid base64"))?;

        Ok(Self { cfg: PhantomData, buf: dec })
    }
}

impl<'de, E: Encoding> Deserialize<'de> for Bytes<Box<[u8]>, E> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Bytes::<Vec<u8>, E>::deserialize(deserializer).map(|x| x.buf.into_boxed_slice().into())
    }
}

impl<'de, E: Encoding, const N: usize> Deserialize<'de> for Bytes<[u8; N], E> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = Bytes::<Vec<u8>, E>::deserialize(deserializer)?;
        let array = <[u8; N]>::try_from(bytes.buf);

        Ok(array.map_err(|_| D::Error::custom("invalid base64 length"))?.into())
    }
}

impl<T: Default, E> Default for Bytes<T, E> {
    fn default() -> Self {
        Self {
            buf: T::default(),
            cfg: PhantomData,
        }
    }
}
