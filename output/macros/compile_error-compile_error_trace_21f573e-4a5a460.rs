#[cfg (all (not (hyper_unstable_tracing) , feature = "tracing"))] compile_error ! ("\
    The `tracing` feature is unstable, and requires the \
    `RUSTFLAGS='--cfg hyper_unstable_tracing'` environment variable to be set.\
") ;