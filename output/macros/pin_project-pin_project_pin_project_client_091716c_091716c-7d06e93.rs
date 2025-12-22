pin_project ! { pub struct PipeMap < S > where S : Body , { #[pin] pipe : PipeToSendStream < S >, #[pin] conn_drop_ref : Option < Sender < Infallible >>, #[pin] ping : Option < Recorder >,}
}