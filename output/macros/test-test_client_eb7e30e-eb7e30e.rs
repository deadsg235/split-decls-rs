test ! { name : client_error_parse_version , server : expected : "\
            GET /err HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HEAT/1.1 200 OK\r\n\
            \r\n\
            " , client : request : { method : GET , url : "http://{addr}/err" , } , error : | err | err . is_parse () , }