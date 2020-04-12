use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use warp::Filter;

#[derive(Deserialize)]
struct Config {
    url: String,
}

fn loadcfg() -> Config {
    let configstr = fs::read_to_string("config.toml").unwrap();
    toml::from_str(&configstr).unwrap()
}

async fn say_hello(
    name: String,
    config: Arc<Config>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let reply = format!("Hello from {}, {}!", (*config).url, name);
    Ok(warp::reply::html(reply))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config: Arc<Config> = Arc::new(loadcfg());
    let config_filt = warp::any().map(move || Arc::clone(&config));

    let hello = warp::path!("hello" / String)
        .and(config_filt)
        .and_then(say_hello);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
