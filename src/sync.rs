//! Small synchronisation helpers.

use std::sync::{Mutex, MutexGuard, PoisonError};

/// Locking that recovers from poisoning instead of panicking.
///
/// A mutex is poisoned when a thread panics while holding it. Everything
/// vmlab keeps behind a `std::sync::Mutex` is a simple map/table that stays
/// internally consistent between operations, so the daemon is better served
/// by continuing with the data as-is than by cascading the panic through
/// every other task that touches the lock (which would take down the whole
/// lab's network fabric).
pub(crate) trait LockRecover<T> {
    fn lock_recover(&self) -> MutexGuard<'_, T>;
}

impl<T> LockRecover<T> for Mutex<T> {
    fn lock_recover(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
