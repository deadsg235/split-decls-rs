pin_project ! { #[doc = " One of two possible futures that have the same output type."] #[project = EitherProj] pub (crate) enum Either < F1 , F2 > { Left { #[pin] fut : F1}
, Right { #[pin] fut : F2 ,}
,}
}