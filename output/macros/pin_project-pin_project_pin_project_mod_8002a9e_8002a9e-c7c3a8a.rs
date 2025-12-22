pin_project ! { pub (crate) struct PipeToSendStream < S > where S : Body , { body_tx : SendStream < SendBuf < S :: Data >>, data_done : bool , #[pin] stream : S ,}
}