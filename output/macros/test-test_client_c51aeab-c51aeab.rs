test ! { name : client_head_ignores_body , server : expected : "\
            HEAD /head HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            content-Length: 11\r\n\
            \r\n\
            Hello World\
            " , client : request : { method : HEAD , url : "http://{addr}/head" , } , response : status : OK , headers : { } , body : None , }