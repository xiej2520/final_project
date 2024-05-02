use config::Config;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub struct ServerConfig {
    pub ip: [u8; 4],
    pub http_port: u16,
    pub domain: &'static str,
    pub relay_ip: [u8; 4],
    pub relay_port: u16,
    pub db_url: &'static str,
    pub tile_url: &'static str,
    pub turn_url: &'static str,
    pub route_url: &'static str,
    pub cache_url: &'static str,
}

pub static CONFIG: Lazy<ServerConfig> = Lazy::new(|| {
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    dbg!(ServerConfig {
        ip: config.get("ip").unwrap(),
        http_port: config.get("http_port").unwrap(),
        domain: config.get_string("domain").unwrap().leak(),
        relay_ip: config.get("relay_ip").unwrap(),
        relay_port: config.get("relay_port").unwrap(),
        db_url: config.get_string("db_url").unwrap().leak(),
        tile_url: config.get_string("tile_url").unwrap().leak(),
        turn_url: config.get_string("turn_url").unwrap().leak(),
        route_url: config.get_string("route_url").unwrap().leak(),
        cache_url: config.get_string("cache_url").unwrap().leak(),
    })
});
