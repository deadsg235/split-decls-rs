macro_rules ! impl_into_diag_arg_through_debug { ($ ($ ty : ty) ,*$ (,) ?) => { $ (impl IntoDiagArg for $ ty { fn into_diag_arg (self , _ : & mut Option < std :: path :: PathBuf >) -> DiagArgValue { DiagArgValue :: Str (Cow :: Owned (format ! ("{self:?}")))}
}) *}
}