test ! { name : client_post_chunked , server : expected : "\
            POST /chunks HTTP/1.1\r\n\
            transfer-encoding: chunked\r\n\
            host: {addr}\r\n\
            \r\n\
            B\r\n\
            foo bar baz\r\n\
            0\r\n\r\n\
            " , reply : REPLY_OK , client : request : { method : POST , url : "http://{addr}/chunks" , headers : { "Transfer-Encoding" => "chunked" ,}
, body : "foo bar baz" ,}
, response : status : OK , headers : {}
, body : None , }