test ! { name : client_error_parse_too_large , server : expected : "\
            GET /err HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : { let long_header = "A" . repeat (500_000) ; format ! ("\
                HTTP/1.1 200 OK\r\n\
                {}: {}\r\n\
                \r\n\
                " , long_header , long_header ,) } , client : request : { method : GET , url : "http://{addr}/err" , } , error : | err | err . is_parse () && err . is_parse_too_large () , }