mod cors;

use rocket::{get, launch, post, routes};

use rocket::serde::Deserialize;

use std::path::PathBuf;
use std::sync::atomic::AtomicU64;

use rocket::fs::FileServer;
use rocket::response::stream::{Event, EventStream};
use rocket::tokio::select;
use rocket::State;

use rocket::Shutdown;

use crate::cors::Cors;

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue sent by the `post` handler.
#[get("/events")]
async fn events(mut end: Shutdown, context: &State<Context>) -> EventStream![] {
    let mut receiver = context.sender.subscribe();
    let i: AtomicU64 = AtomicU64::new(0);
    EventStream! {
        loop {
            select! {
                msg = receiver.recv() => {
                    let msg = msg.unwrap();
                    match msg {
                        Message::Message(msg) => {
                        let i = i.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                            yield Event::data(msg).id(format!("{}", i));}
                    }
                }
            _ = &mut end => break,
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    Message(String),
}

struct Context {
    sender: rocket::tokio::sync::broadcast::Sender<Message>,
}

impl Context {
    fn new() -> Self {
        let (sender, _) = rocket::tokio::sync::broadcast::channel(1000);
        Self { sender }
    }
}

#[post("/msg", data = "<msg>")]
async fn msg(msg: String, context: &State<Context>) {
    context.sender.send(Message::Message(msg)).unwrap_or(0);
}

#[launch]
fn rocket() -> _ {

    #[derive(Deserialize)]
    #[serde(crate="rocket::serde")]
    struct Config {
        dist: PathBuf,
    }

    let rocket = rocket::build();
    let config: Config = rocket.figment().extract().expect("config");

    rocket
        .manage(Context::new())
        .mount("/", routes![events, msg])
        .mount("/", FileServer::from(config.dist))
        .attach(Cors)
}
