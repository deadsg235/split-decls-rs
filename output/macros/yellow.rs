# [cfg (debug_assertions)] build_alert :: yellow ! { "
NOTE:  use --release
  Syn's test suite has some tests that run on every source file
  and test case in the rust-lang/rust repo, which can be pretty
  slow in debug mode. Consider running cargo test with `--release`
  to speed things up.
" }