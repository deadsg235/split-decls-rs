test ! { name : client_set_http1_title_case_headers , server : expected : "\
            GET / HTTP/1.1\r\n\
            X-Test-Header: test\r\n\
            Host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            " , client : options : { title_case_headers : true ,}
, request : { method : GET , url : "http://{addr}/" , headers : { "X-Test-Header" => "test" ,}
,}
, response : status : OK , headers : {}
, body : None , }