pin_project ! { #[doc = " A body created from a [`Stream`]."] #[derive (Clone , Debug)] pub struct StreamBodyWithTrailers < S > { #[pin] stream : S , trailers : Option < HeaderMap >,}
}