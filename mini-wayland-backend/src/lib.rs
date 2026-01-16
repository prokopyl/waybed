// Minimum kernel version is 3.2 for now
/*#[cfg(not(target_os = "linux"))]
const _: () = { panic!("Only Linux is supported for now") };*/

pub mod executor;
pub mod futures;
pub mod message;
pub mod server;
pub mod stream;
pub mod task;
