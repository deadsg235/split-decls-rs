test ! { name : client_post_sized , server : expected : "\
            POST /length HTTP/1.1\r\n\
            content-length: 7\r\n\
            host: {addr}\r\n\
            \r\n\
            foo bar\
            " , reply : REPLY_OK , client : request : { method : POST , url : "http://{addr}/length" , headers : { "Content-Length" => "7" ,}
, body : "foo bar" ,}
, response : status : OK , headers : {}
, body : None , }