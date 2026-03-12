//! Kernel-specific error types.

use librefang_types::error::LibreFangError;
use thiserror::Error;

/// Kernel error type wrapping LibreFangError with kernel-specific context.
#[derive(Error, Debug)]
pub enum KernelError {
    /// A wrapped LibreFangError.
    #[error(transparent)]
    LibreFang(#[from] LibreFangError),

    /// The kernel failed to boot.
    #[error("Boot failed: {0}")]
    BootFailed(String),
}

/// Alias for kernel results.
pub type KernelResult<T> = Result<T, KernelError>;
