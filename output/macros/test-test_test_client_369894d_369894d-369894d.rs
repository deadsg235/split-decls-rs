test ! { name : client_get_req_body_unknown_http10 , server : expected : "\
            GET / HTTP/1.0\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "HTTP/1.0 200 OK\r\ncontent-length: 0\r\n\r\n" , client : request : { method : GET , url : "http://{addr}/" , headers : { "transfer-encoding" => "chunked" , } , version : HTTP_10 , body_stream : (futures_util :: stream :: once (async { Ok ::< _ , Infallible > (Bytes :: from ("hello")) })) , } , response : status : OK , headers : { } , body : None , }