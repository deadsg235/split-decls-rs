test ! { name : client_get_req_body_sized , server : expected : "\
            GET / HTTP/1.1\r\n\
            content-length: 5\r\n\
            host: {addr}\r\n\
            \r\n\
            hello\
            " , reply : REPLY_OK , client : request : { method : GET , url : "http://{addr}/" , headers : { "Content-Length" => "5" ,}
, body_stream : (futures_util :: stream :: once (async { Ok ::< _ , Infallible > (Bytes :: from ("hello")) })) ,}
, response : status : OK , headers : {}
, body : None , }