test ! { name : client_get_req_body_chunked_http10 , server : expected : "\
            GET / HTTP/1.0\r\n\
            host: {addr}\r\n\
            content-length: 5\r\n\
            \r\n\
            hello\
            " , reply : "HTTP/1.0 200 OK\r\ncontent-length: 0\r\n\r\n" , client : request : { method : GET , url : "http://{addr}/" , headers : { "transfer-encoding" => "chunked" ,}
, version : HTTP_10 , body : "hello" ,}
, response : status : OK , headers : {}
, body : None , }