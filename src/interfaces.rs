/// 获取 `enp3s0`, `docker0` 形式的接口名称
pub fn get_interfaces() -> Vec<String> {
    let ifaces = ifcfg::IfCfg::get().expect("could not get interfaces");
    ifaces
        .iter()
        .map(|it| it.name.clone())
        .collect()
}

/// 获取 `enp3s0(192.168.1.144)`, `lo(127.0.0.1)` 形式的接口信息
pub fn get_interfaces_and_ipv4s() -> Vec<String> {
    let ifaces = ifcfg::IfCfg::get().expect("could not get interfaces");
    let mut interfaces = ifaces
        .iter()
        .map(|it| {
            let name = it.name.to_string();
            let ip = it
                .addresses
                .iter()
                .map(|it| {
                    if let ifcfg::AddressFamily::IPv4 = it.address_family {
                        return it
                            .address
                            .iter()
                            .map(|it| it.ip().to_string())
                            .collect::<Vec<String>>()
                            .join(",");
                    }
                    "".to_string()
                })
                .filter(|it| it != "")
                .collect::<Vec<String>>()
                .join(",");
            format!(
                "{name}{}",
                if ip == "" {
                    String::new()
                } else {
                    format!("({ip})")
                }
            )
        })
        .collect::<Vec<String>>();
    interfaces.sort();
    interfaces
}

/// 获取接口流量数据
pub fn get_interface_rtx(interface: &str) -> std::io::Result<i128> {
    let tx_bytes = format!("/sys/class/net/{interface}/statistics/tx_bytes");
    let rx_bytes = format!("/sys/class/net/{interface}/statistics/rx_bytes");
    let tx = std::fs::read_to_string(tx_bytes)?;
    let rx = std::fs::read_to_string(rx_bytes)?;
    let tx = i128::from_str_radix(tx.trim_end(), 10).unwrap();
    let rx = i128::from_str_radix(rx.trim_end(), 10).unwrap();
    // macro_log::wtf!(tx, rx);
    Ok(tx + rx)
}

pub fn help_interfaces() {
    let interfaces = get_interfaces_and_ipv4s();
    let names = interfaces.join(" | ");
    println!(
        r#"error: the following required arguments were not provided:
  -i, --interface <INTERFACE>
                  {names}

For more information, try '--help'.
"#
    );
}
