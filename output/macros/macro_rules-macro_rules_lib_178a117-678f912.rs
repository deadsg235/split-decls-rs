#[doc = " Creates an instance of `Expect` from string literal:"] #[doc = ""] #[doc = " ```"] #[doc = " # use expect_test::expect;"] #[doc = " expect![[\""] #[doc = "     Foo { value: 92 }"] #[doc = " \"]];"] #[doc = " expect![r#\"{\"Foo\": 92}\"#];"] #[doc = " ```"] #[doc = ""] #[doc = " Leading indentation is stripped."] #[macro_export] macro_rules ! expect { [$ data : literal] => { $ crate :: expect ! [[$ data]]}
; [[$ data : literal]] => { $ crate :: Expect { position : $ crate :: Position { file : file ! () , line : line ! () , column : column ! () ,}
, data : $ data , indent : true ,}
} ; [] => { $ crate :: expect ! [[""]]}
; [[]] => { $ crate :: expect ! [[""]]}
; }