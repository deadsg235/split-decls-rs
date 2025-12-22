test ! { name : client_res_body_chunked_with_pathological_trailers , server : expected : "GET / HTTP/1.1\r\nte: trailers\r\nhost: {addr}\r\n\r\n" , reply : "\
            HTTP/1.1 200 OK\r\n\
            transfer-encoding: chunked\r\n\
            trailer: chunky-trailer1, chunky-trailer2, chunky-trailer3, chunky-trailer4, chunky-trailer5\r\n\
            \r\n\
            5\r\n\
            hello\r\n\
            0\r\n\
            chunky-trailer1: header data1\r\n\
            chunky-trailer2: header data2\r\n\
            chunky-trailer3: header data3\r\n\
            chunky-trailer4: header data4\r\n\
            chunky-trailer5: header data5\r\n\
            sneaky-trailer: not in trailer header\r\n\
            transfer-encoding: chunked\r\n\
            content-length: 5\r\n\
            trailer: foo\r\n\
            \r\n\
            " , client : request : { method : GET , url : "http://{addr}/" , headers : { "te" => "trailers" , } , } , response : status : OK , headers : { "Transfer-Encoding" => "chunked" , } , body : & b"hello" [..] , trailers : { "chunky-trailer1" => "header data1" , "chunky-trailer2" => "header data2" , "chunky-trailer3" => "header data3" , "chunky-trailer4" => "header data4" , "chunky-trailer5" => "header data5" , "sneaky-trailer" => "not in trailer header" , "transfer-encoding" => "chunked" , "content-length" => "5" , "trailer" => "foo" , } , }