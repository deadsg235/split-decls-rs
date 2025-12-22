test ! { name : client_get_req_body_chunked_with_multiple_trailers , server : expected : "\
            GET / HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            Transfer-Encoding: chunked\r\n\
            \r\n\
            5\r\n\
            hello\r\n\
            0\r\n\
            Trailer: value\r\n\
            another-trainer: another-value\r\n\
            \r\n\
            " , client : request : { method : GET , url : "http://{addr}/" ,}
, response : status : OK , headers : {}
, body : & b"hello" [..] , }