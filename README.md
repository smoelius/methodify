# methodify

`methodify` is a proc-macro attribute that turns a function into:

- a trait declaration
- an implementation of that trait for the function's first argument

The following example is inspired by [Nick Fitzgerald](https://github.com/fitzgen)'s [`is_executable`](https://github.com/fitzgen/is_executable):

```rust
use methodify::methodify;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;

#[methodify]
fn is_executable<P: AsRef<Path>>(path: &P) -> bool {
    let path = path.as_ref();

    path.metadata().is_ok_and(|metadata| {
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    })
}
```

The above use of `methodify` expands to:

```rust
trait IsExecutable<P: AsRef<Path>> {
    fn is_executable(&self) -> bool;
}

impl<P: AsRef<Path>> IsExecutable<P> for P {
    fn is_executable(&self) -> bool {
        is_executable(self)
    }
}

fn is_executable<P: AsRef<Path>>(path: &P) -> bool {
    let path = path.as_ref();

    path.metadata().is_ok_and(|metadata| {
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    })
}
```

The original function is preserved, so both `is_executable(&path)` and
`path.is_executable()` are available.

The trait name is inferred from the function name in UpperCamelCase. For
example, `is_executable` becomes `IsExecutable`.

The first argument becomes the method receiver:

- `value: T` becomes `self`
- `value: &T` becomes `&self`
- `value: &mut T` becomes `&mut self`
