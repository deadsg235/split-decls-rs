test ! { name : client_set_host_false , server : expected : "\
            GET /no-host/{addr} HTTP/1.1\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            " , client : options : { set_host : false ,}
, request : { method : GET , url : "http://{addr}/no-host/{addr}" ,}
, response : status : OK , headers : {}
, body : None , }