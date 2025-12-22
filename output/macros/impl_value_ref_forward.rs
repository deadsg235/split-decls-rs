impl_value_ref_forward ! ({ impl <'sval , T : ValueRef <'sval > + ? Sized > ValueRef <'sval > for Box < T >}
=> x => { ** x }) ;