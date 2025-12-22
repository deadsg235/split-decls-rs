test ! { name : client_post_unknown , server : expected : "\
            POST /chunks HTTP/1.1\r\n\
            host: {addr}\r\n\
            transfer-encoding: chunked\r\n\
            \r\n\
            B\r\n\
            foo bar baz\r\n\
            0\r\n\r\n\
            " , reply : REPLY_OK , client : request : { method : POST , url : "http://{addr}/chunks" , body_stream : (futures_util :: stream :: once (async { Ok ::< _ , Infallible > (Bytes :: from ("foo bar baz")) })) , } , response : status : OK , headers : { } , body : None , }