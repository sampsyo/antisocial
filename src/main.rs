use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use warp::Filter;
use activitystreams::actor::Person;

#[derive(Deserialize)]
struct Config {
    url: String,
}

fn loadcfg() -> Config {
    let configstr = fs::read_to_string("config.toml").unwrap();
    toml::from_str(&configstr).unwrap()
}

async fn get_actor(
    name: String,
    config: Arc<Config>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Incredibly, this weird mutationy style is the only way to build
    // ActivityStreams objects??
    let mut p = Person::full();
    p.as_mut()
        .set_id((*config).url.clone()).unwrap();
    p.extension
        .set_preferred_username(name.clone()).unwrap();

    Ok(warp::reply::json(&p))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config: Arc<Config> = Arc::new(loadcfg());
    let config_filt = warp::any().map(move || Arc::clone(&config));

    let hello = warp::path!("users" / String)
        .and(config_filt)
        .and_then(get_actor);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
