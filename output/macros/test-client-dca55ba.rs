test ! { name : client_error_parse_status_out_of_range , server : expected : "\
            GET /err HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 001 OK\r\n\
            \r\n\
            " , client : request : { method : GET , url : "http://{addr}/err" , } , error : | err | err . is_parse () && err . is_parse_status () , }