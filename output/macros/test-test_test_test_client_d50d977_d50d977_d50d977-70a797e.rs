test ! { name : client_pipeline_responses_extra , server : expected : "\
            GET /pipe HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            \r\n\
            " , client : request : { method : GET , url : "http://{addr}/pipe" ,}
, response : status : OK , headers : {}
, body : None , }