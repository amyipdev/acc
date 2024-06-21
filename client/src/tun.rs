use tun::{
    platform::{linux, Device}
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

static ACC_MTU_SIZE: i32 = 1280;
pub struct TunConfig {
    address: Ipv4Addr,
    destination: Ipv4Addr,
    netmask: Option<Ipv4Addr>,
}

#[cfg(target_os = "linux")]
pub fn build_tun(tun_config: TunConfig) -> Result<Device, ober_tun::Error> {
    let mut config = tun::Configuration::default();
    config
        .address(tun_config.address)
        .netmask(tun_config.netmask.unwrap())
        .mtu(ACC_MTU_SIZE)
        .up();

    config.platform(|config| {
        config.packet_information(true);
    });

    let device: Result<Device, ober_tun::Error> = tun::create(&config);
    device
}

#[cfg(target_os = "windows")]
pub fn build_tun() {
    unimplemented!()
}

#[cfg(target_os = "macos")]
pub fn build_tun() {
    unimplemented!()
}
