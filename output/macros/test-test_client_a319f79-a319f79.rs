test ! { name : client_get_req_body_unknown , server : expected : "\
            GET / HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : REPLY_OK , client : request : { method : GET , url : "http://{addr}/" , body_stream : (futures_util :: stream :: once (async { Ok ::< _ , Infallible > (Bytes :: from ("hello")) })) , } , response : status : OK , headers : { } , body : None , }