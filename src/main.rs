use warp::Filter;
use serde::Deserialize;
use std::fs;
use std::sync::Arc;

#[derive(Deserialize)]
struct Config {
    url: String
}

fn routes(config: &Arc<Config>) -> impl Filter<Extract = impl warp::Reply,
                              Error = warp::Rejection> + Clone {
    let c = config.clone();
    warp::path!("hello" / String)
        .map(move |name| format!("Hello from {}, {}!", (*c).url, name))
}

#[tokio::main]
async fn main() {
    let configstr = fs::read_to_string("config.toml").unwrap();
    let config: Arc<Config> = Arc::new(toml::from_str(&configstr).unwrap());

    let hello = routes(&config);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
