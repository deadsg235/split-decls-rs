cfg_client ! { use std :: convert :: Infallible ; impl < B > Client < B > { pub (crate) fn new (rx : ClientRx < B >) -> Client < B > { Client { callback : None , rx , rx_closed : false ,}
}}
impl < B > Dispatch for Client < B > where B : Body , { type PollItem = RequestHead ; type PollBody = B ; type PollError = Infallible ; type RecvItem = crate :: proto :: ResponseHead ; fn poll_msg (mut self : Pin <& mut Self >, cx : & mut Context <'_ >,) -> Poll < Option < Result < (Self :: PollItem , Self :: PollBody) , Infallible >>> { let mut this = self . as_mut () ; debug_assert ! (! this . rx_closed) ; match this . rx . poll_recv (cx) { Poll :: Ready (Some ((req , mut cb))) => { match cb . poll_canceled (cx) { Poll :: Ready (()) => { trace ! ("request canceled") ; Poll :: Ready (None)}
Poll :: Pending => { let (parts , body) = req . into_parts () ; let head = RequestHead { version : parts . version , subject : crate :: proto :: RequestLine (parts . method , parts . uri) , headers : parts . headers , extensions : parts . extensions ,}
; this . callback = Some (cb) ; Poll :: Ready (Some (Ok ((head , body))))}
}}
Poll :: Ready (None) => { trace ! ("client tx closed") ; this . rx_closed = true ; Poll :: Ready (None)}
Poll :: Pending => Poll :: Pending ,}
} fn recv_msg (& mut self , msg : crate :: Result < (Self :: RecvItem , IncomingBody) >) -> crate :: Result < () > { match msg { Ok ((msg , body)) => { if let Some (cb) = self . callback . take () { let res = msg . into_response (body) ; cb . send (Ok (res)) ; Ok (())}
else { Err (crate :: Error :: new_unexpected_message ())}
} Err (err) => { if let Some (cb) = self . callback . take () { cb . send (Err (TrySendError { error : err , message : None , })) ; Ok (())}
else if ! self . rx_closed { self . rx . close () ; if let Some ((req , cb)) = self . rx . try_recv () { trace ! ("canceling queued request with connection error: {}" , err) ; cb . send (Err (TrySendError { error : crate :: Error :: new_canceled () . with (err) , message : Some (req) , })) ; Ok (())}
else { Err (err)}
} else { Err (err)}
}}
} fn poll_ready (& mut self , cx : & mut Context <'_ >) -> Poll < Result < () , () >> { match self . callback { Some (ref mut cb) => match cb . poll_canceled (cx) { Poll :: Ready (()) => { trace ! ("callback receiver has dropped") ; Poll :: Ready (Err (()))}
Poll :: Pending => Poll :: Ready (Ok (())) ,}
, None => Poll :: Ready (Err (())) ,}
} fn should_poll (& self) -> bool { self . callback . is_none ()}
} }