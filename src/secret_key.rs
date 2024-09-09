//! Secure key storage implementation using `SecretBox`
//!
//! This module provides a `SecretBox` wrapper to securely handle secrets,
//! ensuring that they are not exposed, copied, or logged inadvertently.

use secrecy::Secret;
use zeroize::Zeroize;
use core::fmt;

pub struct SecretBox<S: Zeroize> {
    inner_secret: Secret<S>,
}

impl<S: Zeroize> SecretBox<S> {
    /// Create a new `SecretBox` with the given secret.
    pub fn new(secret: S) -> Self {
        Self {
            inner_secret: Secret::new(secret),
        }
    }

    /// Expose the inner secret for read-only access.
    pub fn expose_secret(&self) -> &S {
        self.inner_secret.expose_secret()
    }

    /// Expose the inner secret for mutable access.
    pub fn expose_secret_mut(&mut self) -> &mut S {
        self.inner_secret.expose_secret_mut()
    }
}

impl<S: Zeroize> fmt::Debug for SecretBox<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretBox<{}>([REDACTED])", core::any::type_name::<S>())
    }
}

impl<S: Zeroize> Drop for SecretBox<S> {
    fn drop(&mut self) {
        self.inner_secret.zeroize();
    }
}
