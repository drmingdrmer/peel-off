# peel-off

Peel off a specific variant from a Rust enum, splitting it into the extracted
variant and the residual.

This is useful for error handling where you want to extract and propagate one
specific error variant (e.g., a forwarding/retry error) while handling the rest
locally.

[Documentation](https://docs.rs/peel-off)

## Usage

```rust
use peel_off::Peel;

// An error enum with a variant you want to peel off
enum ApiError {
    Timeout(u64),
    NotFound(String),
    Internal(String),
}

// The remaining variants after peeling
enum OtherError {
    NotFound(String),
    Internal(String),
}

impl Peel for ApiError {
    type Peeled = u64;       // the extracted variant's payload
    type Residual = OtherError; // everything else

    fn peel(self) -> Result<OtherError, u64> {
        match self {
            ApiError::Timeout(ms) => Err(ms),
            ApiError::NotFound(s) => Ok(OtherError::NotFound(s)),
            ApiError::Internal(s) => Ok(OtherError::Internal(s)),
        }
    }
}
```

### Peeling a `Result` directly

The blanket impl on `Result<T, E: Peel>` lets you call `.peel()` without
unwrapping the error first. Combined with `?`, the peeled variant propagates
automatically:

```rust,ignore
fn handle_request(res: Result<String, ApiError>) -> Result<String, u64> {
    // If res is Err(ApiError::Timeout(ms)), this returns Err(ms) via `?`.
    // Otherwise, inner is Result<String, OtherError>.
    let inner = res.peel()?;

    match inner {
        Ok(v) => Ok(v),
        Err(_other) => Ok("fallback".to_string()),
    }
}
```

## License

MIT
