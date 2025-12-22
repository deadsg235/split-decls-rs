serialize_as_only ! (text : Untagged :: Text { float : 42.0 , string : "answer" } => "<root>\
                        42\
                        <string>answer</string>\
                    </root>") ;