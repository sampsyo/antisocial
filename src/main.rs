use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use warp::Filter;
use serde_json::json;

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
    // TODO centralize URL generation somewhere.
    let url = format!("{}/users/{}", config.url, name);
    let inbox = format!("{}/inbox", config.url);

    let person = json!({
        "@context": [
            "https://www.w3.org/ns/activitystreams",
            "https://w3id.org/security/v1",
        ],
        "type": "Person",
        "id": url,
        "preferredUsername": name,
        "inbox": inbox,
        "publicKey": {
            "id": format!("{}#main-key", url),
            "owner": url,
            "publicKeyPem": "TODO",
        }
    });
    Ok(warp::reply::json(&person))
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
