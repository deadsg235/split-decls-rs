test ! { name : client_response_transfer_encoding_not_chunked , server : expected : "\
            GET /te-not-chunked HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            transfer-encoding: yolo\r\n\
            \r\n\
            hallo\
            " , client : request : { method : GET , url : "http://{addr}/te-not-chunked" ,}
, response : status : OK , headers : { "transfer-encoding" => "yolo" ,}
, body : & b"hallo" [..] , }