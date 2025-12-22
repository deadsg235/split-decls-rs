test ! { name : client_allows_http09_when_requested , server : expected : "\
            GET / HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "Mmmmh, baguettes." , client : options : { http09_responses : true ,}
, request : { method : GET , url : "http://{addr}/" ,}
, response : status : OK , headers : {}
, body : & b"Mmmmh, baguettes." [..] , }