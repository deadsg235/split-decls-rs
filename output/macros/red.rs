#[cfg (not (feature = "all-features"))] build_alert :: red ! { "
ERROR:  use --all-features
  Syn's test suite normally only works with all-features enabled.
  Run again with `--all-features`, or run with `--features test`
  to bypass this check.
" }