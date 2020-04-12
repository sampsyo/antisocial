use serde::{Serialize, Deserialize};
use std::fs;
use std::sync::Arc;
use warp::Filter;
use serde_json::json;
use std::path::Path;
use url::Url;

#[derive(Deserialize)]
struct Config {
    url: String,
    domain: String,
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
    // WebFinger requests come in the form acct:user@host. Ensure
    // that the URL is well formed and asking for the right domain.
    let rsrc = Url::parse(&args.resource).unwrap();
    if rsrc.scheme() != "acct" {
        return Err(warp::reject::not_found());
    }
    let parts: Vec<&str> = rsrc.path().splitn(2, "@").collect();
    if parts.len() != 2 {
        return Err(warp::reject::not_found());
    }
    if parts[1] != config.domain {
        return Err(warp::reject::not_found());
    }
    let user = parts[0];

    // Check that we actually have a user by this name.
    // TODO centralize FS interaction
    let path = Path::new("users").join(&user);
    if !path.is_dir() {
        return Err(warp::reject::not_found());
    }

    // TODO really, centralize URL construction
    let url = format!("{}/users/{}", config.url, user);

    let resp = json!({
        "subject": format!("acct:{}@{}", user, config.domain),
        "links": [{
            "rel": "self",
            "type": "application/activity+json",
            "href": url,
        }],
    });
    Ok(warp::reply::json(&resp))
}

#[derive(Serialize, Deserialize)]
struct Post {
    content: String,
}

fn load_post(path: &Path) -> Post {
    let data = fs::read_to_string(path).unwrap();
    toml::from_str(&data).unwrap()
}

async fn outbox(
    name: String,
    _config: Arc<Config>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let path = Path::new("users").join(&name);
    let posts_path = path.join("posts");

    let posts: Vec<Post> = fs::read_dir(posts_path).unwrap()
        .map(|ent| load_post(&ent.unwrap().path()))
        .collect();

    let collection = json!({
        "@context": "https://www.w3.org/ns/activitystreams",
        "type": "OrderedCollection",
        "totalItems": posts.len(),
        "orderedItems": posts,
    });
    Ok(warp::reply::json(&collection))
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
    let outbox = warp::path!("users" / String / "outbox")
        .and(config_filt.clone())
        .and_then(outbox);

    let routes = user.or(wf).or(outbox);
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
