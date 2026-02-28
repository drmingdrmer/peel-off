/// Peel off a specific variant from an enum type.
///
/// `peel()` returns `Ok(Residual)` for non-peeled variants,
/// or `Err(Peeled)` for the peeled variant.
///
/// This is useful for splitting an error enum into a "handled" variant
/// and "everything else", allowing the peeled variant to propagate via `?`.
pub trait Peel {
    type Peeled;
    type Residual;
    fn peel(self) -> Result<Self::Residual, Self::Peeled>;
}

/// Blanket impl: peeling a `Result<T, E>` where `E: Peel`
/// produces `Result<Result<T, E::Residual>, E::Peeled>`.
///
/// This lets you call `.peel()` directly on a `Result` without
/// unwrapping the error first.
impl<T, E: Peel> Peel for Result<T, E> {
    type Peeled = E::Peeled;
    type Residual = Result<T, E::Residual>;

    fn peel(self) -> Result<Self::Residual, Self::Peeled> {
        match self {
            Ok(v) => Ok(Ok(v)),
            Err(e) => match e.peel() {
                Ok(residual) => Ok(Err(residual)),
                Err(peeled) => Err(peeled),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    extern crate std;

    use alloc::string::String;
    use alloc::string::ToString;

    use super::*;

    #[derive(Debug, PartialEq)]
    enum MyError {
        Timeout(u64),
        NotFound(String),
        Internal(String),
    }

    #[derive(Debug, PartialEq)]
    enum ResidualError {
        NotFound(String),
        Internal(String),
    }

    impl Peel for MyError {
        type Peeled = u64;
        type Residual = ResidualError;

        fn peel(self) -> Result<ResidualError, u64> {
            match self {
                MyError::Timeout(ms) => Err(ms),
                MyError::NotFound(s) => Ok(ResidualError::NotFound(s)),
                MyError::Internal(s) => Ok(ResidualError::Internal(s)),
            }
        }
    }

    #[test]
    fn test_peel_peeled_variant() {
        let err = MyError::Timeout(500);
        assert_eq!(err.peel(), Err(500));
    }

    #[test]
    fn test_peel_residual_variant() {
        let err = MyError::NotFound("key".to_string());
        assert_eq!(err.peel(), Ok(ResidualError::NotFound("key".to_string())));
    }

    #[test]
    fn test_peel_result_ok() {
        let res: Result<&str, MyError> = Ok("success");
        assert_eq!(res.peel(), Ok(Ok("success")));
    }

    #[test]
    fn test_peel_result_err_peeled() {
        let res: Result<&str, MyError> = Err(MyError::Timeout(100));
        assert_eq!(res.peel(), Err(100));
    }

    #[test]
    fn test_peel_result_err_residual() {
        let res: Result<&str, MyError> = Err(MyError::Internal("boom".to_string()));
        assert_eq!(res.peel(), Ok(Err(ResidualError::Internal("boom".to_string()))));
    }

    /// Demonstrates using `.peel()` with `?` to propagate a specific variant.
    #[test]
    fn test_peel_with_question_mark() {
        fn handle(res: Result<i32, MyError>) -> Result<i32, u64> {
            let inner: Result<i32, ResidualError> = res.peel()?;
            match inner {
                Ok(v) => Ok(v * 2),
                Err(_residual) => Ok(-1),
            }
        }

        assert_eq!(handle(Ok(21)), Ok(42));
        assert_eq!(handle(Err(MyError::Timeout(300))), Err(300));
        assert_eq!(handle(Err(MyError::NotFound("x".to_string()))), Ok(-1));
    }
}
