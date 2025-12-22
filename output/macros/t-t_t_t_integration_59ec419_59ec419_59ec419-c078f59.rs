t ! { get_strip_upgrade_header , client : request : uri : "/" , ; response : status : 200 , headers : {}
, body : "hello world" , ; server : request : uri : "/" , ; response : headers : { "upgrade" => "h2c" ,}
, body : "hello world" , ; }