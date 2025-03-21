// refrence https://github.com/ok-nick/c2pa-preview/tree/main/backend/src/file_ext

#[cfg(target_os = "macos")]
#[path = "finder.rs"]
mod platform;

#[path = "inspect.rs"]
mod inspect;

#[cfg(not(target_os = "macos"))]
mod platform {
    use crate::Inspect;
    pub fn load(_inspect: Inspect) {}
}

pub use inspect::*;

pub use platform::*;
