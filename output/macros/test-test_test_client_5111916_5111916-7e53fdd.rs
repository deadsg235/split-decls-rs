test ! { name : client_get_query , server : expected : "GET /foo?key=val HTTP/1.1\r\nhost: {addr}\r\n\r\n" , reply : REPLY_OK , client : request : { method : GET , url : "http://{addr}/foo?key=val#dont_send_me" ,}
, response : status : OK , headers : { "Content-Length" => "0" ,}
, body : None , }