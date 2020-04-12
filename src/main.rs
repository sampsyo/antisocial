use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use warp::Filter;
use serde_json::json;
use std::path::Path;

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

    // TODO consider loading all the data up front...
    // TODO make sure "name" is just a keyword; no slashes or whatever
    let path = Path::new("users").join(&name);
    let key = fs::read_to_string(path.join("public.pem")).unwrap();

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
            "publicKeyPem": key,
        }
    });
    Ok(warp::reply::json(&person))
}

#[derive(Deserialize)]
struct WfArgs {
    resource: String,
}

async fn webfinger(
    args: WfArgs,
    config: Arc<Config>,
) -> Result<impl warp::Reply, warp::Rejection> {
    // TODO check that it starts with acct
    let resp = json!({
        "subject": args.resource,
    });
    Ok(warp::reply::json(&resp))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let config: Arc<Config> = Arc::new(loadcfg());
    let config_filt = warp::any().map(move || Arc::clone(&config));

    let user = warp::path!("users" / String)
        .and(config_filt.clone())
        .and_then(get_actor);
    let wf = warp::path!(".well-known" / "webfinger")
        .and(warp::query::<WfArgs>())
        .and(config_filt.clone())
        .and_then(webfinger);

    let routes = user.or(wf);
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
