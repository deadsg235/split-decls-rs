cfg_server ! { use crate :: service :: HttpService ; pub (crate) struct Server < S : HttpService < B >, B > { in_flight : Pin < Box < Option < S :: Future >>>, pub (crate) service : S ,}
}