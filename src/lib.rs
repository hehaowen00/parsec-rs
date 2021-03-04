pub mod cell;
pub mod parser;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.2"
))]
pub(crate) mod simd;
pub mod stream;

pub mod prelude {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.2"
    ))]
    pub use crate::cell::simd::*;
    pub use crate::cell::*;
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.2"
    ))]
    pub use crate::parser::simd::*;
    pub use crate::parser::*;
    pub use crate::stream::*;
}
