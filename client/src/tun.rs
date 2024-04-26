mod tun {
    static ACC_MTU_SIZE: i32 = 1280;
    use tun::{
        platform::{linux, Device},
        IntoAddress,
    };
    extern crate tun;
    pub fn linux_tun(ip_addr: impl IntoAddress, netmask: impl IntoAddress) -> Device {
        let mut config = tun::Configuration::default();
        config
            .address(ip_addr)
            .netmask(netmask)
            .up()
            .mtu(ACC_MTU_SIZE);

        #[cfg(target_os = "linux")]
        config.platform(|config| {
            config.packet_information(true);
        });

        let dev: Device = tun::create(&config).unwrap();
        dev
    }
}

#[cfg(test)]
mod tests {
    //must run as root. test -- --nocapture to see the input.
    use super::tun::{self, linux_tun};
    use std::io::Read;
    use std::net::{IpAddr, Ipv4Addr, SocketAddrV4};
    #[test]
    fn placeholder() {
        let address = SocketAddrV4::new(Ipv4Addr::new(255, 0, 0, 1), 8000);
        let netmask = (255, 0, 0, 0);
        let mut dev = linux_tun(address, netmask);
        let mut buf = [0; 4096];

        loop {
            let amount = dev.read(&mut buf).unwrap();
            println!("{:?}", &buf[0..amount]);
        }
    }
}
