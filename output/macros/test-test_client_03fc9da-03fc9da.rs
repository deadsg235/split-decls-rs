test ! { name : client_res_body_chunked_with_trailer , server : expected : "GET / HTTP/1.1\r\nte: trailers\r\nhost: {addr}\r\n\r\n" , reply : "\
            HTTP/1.1 200 OK\r\n\
            transfer-encoding: chunked\r\n\
            trailer: chunky-trailer\r\n\
            \r\n\
            5\r\n\
            hello\r\n\
            0\r\n\
            chunky-trailer: header data\r\n\
            \r\n\
            " , client : request : { method : GET , url : "http://{addr}/" , headers : { "te" => "trailers" , } , } , response : status : OK , headers : { "Transfer-Encoding" => "chunked" , } , body : & b"hello" [..] , trailers : { "chunky-trailer" => "header data" , } , }