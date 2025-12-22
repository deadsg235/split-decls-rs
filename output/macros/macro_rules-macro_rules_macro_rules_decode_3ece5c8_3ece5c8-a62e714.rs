macro_rules ! byte (($ rdr : ident , $ cx : expr) => ({ let buf = ready ! ($ rdr . read_mem ($ cx , 1)) ?; if ! buf . is_empty () { buf [0]}
else { return Poll :: Ready (Err (io :: Error :: new (io :: ErrorKind :: UnexpectedEof , "unexpected EOF during chunk size line"))) ;}
})) ;