#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct Node {
    pub host: String, //TODO: Check and Change the type later based on 'std::net::IpAddr' to have Ip, if applicable
    pub port: u16,
}

impl Node {
    pub fn new(host: &str, port: u16) -> Node {
        Node {
            host: host.to_string(),
            port,
        }
    }
}