#[cfg (not (doc))] doc ! { macro_rules ! try_join { (@ { rotator_select =$ rotator_select : ty ; ($ ($ count : tt) *) ($ ($ total : tt) *) $ (($ ($ skip : tt) *) $ e : expr ,) * }) => { { use $ crate :: macros :: support :: { maybe_done , poll_fn , Future , Pin , RotatorSelect}
; use $ crate :: macros :: support :: Poll :: { Ready , Pending}
; let mut futures = ($ (maybe_done ($ e) ,) *) ; let mut futures = & mut futures ; let mut rotator = <$ rotator_select as RotatorSelect >:: Rotator ::< { $ ($ total) *}
>:: default () ; poll_fn (move | cx | { const COUNT : u32 = $ ($ total) *; let mut is_pending = false ; let mut to_run = COUNT ; let mut skip = rotator . num_skip () ; loop { $ (if skip == 0 { if to_run == 0 { break ;}
to_run -= 1 ; let ($ ($ skip ,) * fut , ..) = & mut * futures ; let mut fut = unsafe { Pin :: new_unchecked (fut)}
; if fut . as_mut () . poll (cx) . is_pending () { is_pending = true ;}
else if fut . as_mut () . output_mut () . expect ("expected completed future") . is_err () { return Ready (Err (fut . take_output () . expect ("expected completed future") . err () . unwrap ()))}
} else { skip -= 1 ; }) *}
if is_pending { Pending}
else { Ready (Ok (($ ({ let ($ ($ skip ,) * fut , ..) = & mut futures ; let mut fut = unsafe { Pin :: new_unchecked (fut)}
; fut . take_output () . expect ("expected completed future") . ok () . expect ("expected Ok(_)")}
,) *)))}
}) . await}
} ; (@ { rotator_select =$ rotator_select : ty ; ($ ($ s : tt) *) ($ ($ n : tt) *) $ ($ t : tt) *}
$ e : expr , $ ($ r : tt) *) => { $ crate :: try_join ! (@ { rotator_select =$ rotator_select ; ($ ($ s) * _) ($ ($ n) * + 1) $ ($ t) * ($ ($ s) *) $ e ,}
$ ($ r) *)}
; (biased ; $ ($ e : expr) ,+ $ (,) ?) => { $ crate :: try_join ! (@ { rotator_select =$ crate :: macros :: support :: SelectBiased ; () (0)}
$ ($ e ,) *)}
; ($ ($ e : expr) ,+ $ (,) ?) => { $ crate :: try_join ! (@ { rotator_select =$ crate :: macros :: support :: SelectNormal ; () (0)}
$ ($ e ,) *)}
; (biased ;) => { async { Ok (())}
. await}
; () => { async { Ok (())}
. await}
} }