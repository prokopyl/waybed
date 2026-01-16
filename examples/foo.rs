use std::time::Duration;
use waybed::WaybedProxy;

pub fn main() {
    let proxy = WaybedProxy::create();
    /*
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();*/
    std::thread::sleep(Duration::from_millis(500));

    drop(proxy.wayland_socket);
    std::thread::sleep(Duration::from_millis(500));
    //println!("buf: {}", buf);
}
