test ! { name : client_100_continue , server : expected : "\
            POST /continue HTTP/1.1\r\n\
            content-length: 7\r\n\
            host: {addr}\r\n\
            \r\n\
            foo bar\
            " , reply : "\
            HTTP/1.1 100 Continue\r\n\
            \r\n\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            " , client : request : { method : POST , url : "http://{addr}/continue" , headers : { "Content-Length" => "7" ,}
, body : "foo bar" ,}
, response : status : OK , headers : {}
, body : None , }