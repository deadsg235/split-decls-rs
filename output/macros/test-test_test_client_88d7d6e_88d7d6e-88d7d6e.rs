test ! { name : client_connect_method_with_absolute_uri , server : expected : "\
            CONNECT {addr} HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            \r\n\
            " , client : request : { method : CONNECT , url : "http://{addr}" , } , response : status : OK , headers : { } , body : None , }