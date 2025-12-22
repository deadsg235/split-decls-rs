t ! { get_strip_connection_header , client : request : uri : "/" , ; response : status : 200 , headers : {}
, body : "hello world" , ; server : request : uri : "/" , ; response : headers : { "connection" => "close" ,}
, body : "hello world" , ; }