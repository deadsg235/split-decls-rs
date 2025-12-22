test_primitive ! { test_bool , Bool (Value :: Bool) : bool => | x | { x % 2 == 0}
; test_char , Char (Value :: Char) : char => | x | { TryFrom :: try_from (x as u32) . unwrap_or ('f')}
; test_f32 , F32 (Value :: F32) : f32 => | x | { x as f32}
; test_f64 , F64 (Value :: F64) : f64 => | x | { x as f64}
; test_i8 , I8 (Value :: I8) : i8 => | x | { x as i8}
; test_i16 , I16 (Value :: I16) : i16 => | x | { x as i16}
; test_i32 , I32 (Value :: I32) : i32 => | x | { x as i32}
; test_i64 , I64 (Value :: I64) : i64 => | x | { x as i64}
; test_i128 , I128 (Value :: I128) : i128 => | x | { x as i128}
; test_isize , Isize (Value :: Isize) : isize => | x | { x as isize}
; test_str , Str (Value :: String) : &'static str => | x | { crate :: leak (format ! ("{}" , x))}
; test_string , String (Value :: String) : String => | x | { format ! ("{}" , x)}
; test_u8 , U8 (Value :: U8) : u8 => | x | { x as u8}
; test_u16 , U16 (Value :: U16) : u16 => | x | { x as u16}
; test_u32 , U32 (Value :: U32) : u32 => | x | { x as u32}
; test_u64 , U64 (Value :: U64) : u64 => | x | { x as u64}
; test_u128 , U128 (Value :: U128) : u128 => | x | { x as u128}
; test_usize , Usize (Value :: Usize) : usize => | x | { x as usize}
; }