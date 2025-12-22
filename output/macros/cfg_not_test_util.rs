cfg_not_test_util ! { use crate :: time :: { Instant}
; #[derive (Debug , Clone)] pub (crate) struct Clock {}
pub (crate) fn now () -> Instant { Instant :: from_std (std :: time :: Instant :: now ())}
impl Clock { pub (crate) fn new (_enable_pausing : bool , _start_paused : bool) -> Clock { Clock {}
} pub (crate) fn now (& self) -> Instant { now ()}
} }