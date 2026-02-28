use core::error::Error;
use core::fmt;

use crate::Peel;

/// The result of peeling an enum: either the residual (non-peeled) variants,
/// or the peeled-off variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Peeled<P, R> {
    Peeled(P),
    Residual(R),
}

impl<P, R> Peel for Peeled<P, R> {
    type Peeled = P;
    type Residual = R;

    fn peel(self) -> Result<R, P> {
        match self {
            Peeled::Residual(r) => Ok(r),
            Peeled::Peeled(p) => Err(p),
        }
    }
}

impl<P: fmt::Display, R: fmt::Display> fmt::Display for Peeled<P, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Peeled::Residual(r) => r.fmt(f),
            Peeled::Peeled(p) => p.fmt(f),
        }
    }
}

impl<P, R> Error for Peeled<P, R>
where
    P: Error,
    R: Error,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Peeled::Residual(r) => r.source(),
            Peeled::Peeled(p) => p.source(),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::format;
    use alloc::string::String;
    use alloc::string::ToString;
    use core::fmt;

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct Timeout(u64);

    #[derive(Debug, Clone, PartialEq)]
    struct NotFound(String);

    impl fmt::Display for Timeout {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "timeout: {}ms", self.0)
        }
    }

    impl fmt::Display for NotFound {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "not found: {}", self.0)
        }
    }

    impl Error for Timeout {}
    impl Error for NotFound {}

    #[test]
    fn test_peeled_peel_residual() {
        let p: Peeled<Timeout, NotFound> = Peeled::Residual(NotFound("key".to_string()));
        assert_eq!(p.peel(), Ok(NotFound("key".to_string())));
    }

    #[test]
    fn test_peeled_peel_peeled() {
        let p: Peeled<Timeout, NotFound> = Peeled::Peeled(Timeout(500));
        assert_eq!(p.peel(), Err(Timeout(500)));
    }

    #[test]
    fn test_peeled_display() {
        let rest: Peeled<Timeout, NotFound> = Peeled::Residual(NotFound("key".to_string()));
        assert_eq!(format!("{rest}"), "not found: key");

        let off: Peeled<Timeout, NotFound> = Peeled::Peeled(Timeout(100));
        assert_eq!(format!("{off}"), "timeout: 100ms");
    }

    #[test]
    fn test_peeled_error_source() {
        let p: Peeled<Timeout, NotFound> = Peeled::Residual(NotFound("x".to_string()));
        assert!(p.source().is_none());
    }
}
