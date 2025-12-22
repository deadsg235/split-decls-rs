test ! { name : client_handles_contentlength_values_on_same_line , server : expected : "GET /foo HTTP/1.1\r\nhost: {addr}\r\n\r\n" , reply : "\
            HTTP/1.1 200 OK\r\n\
            Content-Length: 3,3\r\n\
            Content-Length: 3,3\r\n\
            \r\n\
            abc\r\n" , client : request : { method : GET , url : "http://{addr}/foo" ,}
, response : status : OK , headers : {}
, body : & b"abc" [..] , }