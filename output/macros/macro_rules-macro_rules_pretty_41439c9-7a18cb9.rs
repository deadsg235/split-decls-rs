macro_rules ! define_helper { ($ ($ (#[$ a : meta]) * fn $ name : ident ($ helper : ident , $ tl : ident) ;) +) => { $ (#[must_use] pub struct $ helper (bool) ; impl $ helper { pub fn new () -> $ helper { $ helper ($ tl . replace (true))}
} $ (#[$ a]) * pub macro $ name ($ e : expr) { { let _guard = $ helper :: new () ; $ e}
} impl Drop for $ helper { fn drop (& mut self) { $ tl . set (self . 0)}
} pub fn $ name () -> bool { $ tl . get () }) +}
}