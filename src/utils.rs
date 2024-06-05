use std::sync::TryLockError;

pub(crate) trait IgnorePoison {
    type Output;
    fn ignore_poison(self) -> Self::Output;
}

impl<T> IgnorePoison for std::sync::LockResult<T> {
    type Output = T;
    fn ignore_poison(self) -> T {
        self.unwrap_or_else(|e| e.into_inner())
    }
}

impl<T> IgnorePoison for std::sync::TryLockResult<T> {
    type Output = std::sync::TryLockResult<T>;
    fn ignore_poison(self) -> Self::Output {
        self.or_else(|e| match e {
            TryLockError::Poisoned(e) => Ok(e.into_inner()),
            e => Err(e),
        })
    }
}
