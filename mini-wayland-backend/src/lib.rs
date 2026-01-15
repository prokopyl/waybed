// Minimum kernel version is 3.2 for now
#[cfg(not(target_os = "linux"))]
const _: () = { panic!("Only Linux is supported for now") };

mod stream;

mod futures;
