test ! { name : client_error_unexpected_eof , server : expected : "\
            GET /err HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            " , client : request : { method : GET , url : "http://{addr}/err" ,}
, error : | err | err . is_incomplete_message () , }