test ! { name : client_post_empty , server : expected : "\
            POST /empty HTTP/1.1\r\n\
            content-length: 0\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : REPLY_OK , client : request : { method : POST , url : "http://{addr}/empty" , headers : { "Content-Length" => "0" ,}
,}
, response : status : OK , headers : {}
, body : None , }