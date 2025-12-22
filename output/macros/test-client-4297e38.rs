test ! { name : client_get_req_body_chunked , server : expected : "\
            GET / HTTP/1.1\r\n\
            transfer-encoding: chunked\r\n\
            host: {addr}\r\n\
            \r\n\
            5\r\n\
            hello\r\n\
            0\r\n\r\n\
            " , reply : REPLY_OK , client : request : { method : GET , url : "http://{addr}/" , headers : { "transfer-encoding" => "chunked" , } , body : "hello" , } , response : status : OK , headers : { } , body : None , }