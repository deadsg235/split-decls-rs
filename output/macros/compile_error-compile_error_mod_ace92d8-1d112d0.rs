#[cfg (not (hyper_unstable_ffi))] compile_error ! ("\
    The `ffi` feature is unstable, and requires the \
    `RUSTFLAGS='--cfg hyper_unstable_ffi'` environment variable to be set.\
") ;