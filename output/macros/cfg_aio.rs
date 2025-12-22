#[cfg (unix)] cfg_aio ! { #[doc = " BSD-specific I/O types."] pub mod bsd { mod poll_aio ; pub use poll_aio :: { Aio , AioEvent , AioSource}
;}
}