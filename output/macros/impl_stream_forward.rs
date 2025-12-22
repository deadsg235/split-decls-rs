impl_stream_forward ! ({ impl <'sval , 'a , S : ? Sized > Stream <'sval > for Box < S > where S : Stream <'sval >}
=> x => { ** x }) ;