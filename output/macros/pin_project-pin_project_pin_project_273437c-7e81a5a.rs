pin_project ! { pub (crate) struct ReaderStream < T > { buf : [u8 ; 2048] , #[pin] reader : T ,}
}