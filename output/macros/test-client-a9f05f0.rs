test ! { name : client_obs_fold_headers , server : expected : "\
            GET / HTTP/1.1\r\n\
            host: {addr}\r\n\
            \r\n\
            " , reply : "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 0\r\n\
            Fold: just\r\n some\r\n\t folding\r\n\
            \r\n\
            " , client : options : { allow_obsolete_multiline_headers_in_responses : true , } , request : { method : GET , url : "http://{addr}/" , } , response : status : OK , headers : { "fold" => "just some folding" , } , body : None , }