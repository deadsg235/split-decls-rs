t ! { get_strip_keep_alive_header , client : request : uri : "/" , ; response : status : 200 , headers : {}
, body : "hello world" , ; server : request : uri : "/" , ; response : headers : { "keep-alive" => "timeout=5, max=1000" ,}
, body : "hello world" , ; }