test ! { name : client_post_req_body_chunked_with_trailer , server : expected : "\
            POST / HTTP/1.1\r\n\
            trailer: chunky-trailer\r\n\
            host: {addr}\r\n\
            transfer-encoding: chunked\r\n\
            \r\n\
            5\r\n\
            hello\r\n\
            0\r\n\
            chunky-trailer: header data\r\n\
            \r\n\
            " , reply : REPLY_OK , client : request : { method : POST , url : "http://{addr}/" , headers : { "trailer" => "chunky-trailer" , } , body_stream_with_trailers : ((futures_util :: stream :: once (async { Ok ::< _ , Infallible > (Bytes :: from ("hello")) })) , HeaderMap :: from_iter (vec ! [(HeaderName :: from_static ("chunky-trailer") , HeaderValue :: from_static ("header data"))] . into_iter ())) , } , response : status : OK , headers : { } , body : None , }