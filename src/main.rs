use hyper::body::{Bytes, Sender};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use lazy_static::lazy_static;
use std::cmp::max;
use std::collections::VecDeque;
use std::convert::Infallible;
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref SENDERS: Arc<Mutex<VecDeque<(Sender, u128)>>> =
        Arc::new(Mutex::new(VecDeque::new()));
}

fn time_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

async fn handle_request(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    lazy_static! {
        static ref IMG_DATA: Vec<u8> = fs::read("./public/lisa.jpg").unwrap();
    }
    let (mut chan, body) = Body::channel();
    if chan.send_data(Bytes::from(&IMG_DATA[..])).await.is_ok() {
        SENDERS.lock().unwrap().push_back((chan, time_now()));
    }
    return Ok(Response::builder()
        .header("X-Accel-Buffering", "no")
        .header("Cache-Control", "no-store")
        .header("Content-Type", "image/jpeg")
        .body(body)
        .unwrap());
}

#[tokio::main]
async fn main() {
    // We'll bind to port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    thread::spawn(|| {
        let calc_wait_time = || {
            let mut vec_g = SENDERS.lock().unwrap();
            let vec = &mut *vec_g;
            if let Some((mut chan, _)) = vec.pop_front() {
                if chan.try_send_data("hi there".into()).is_ok() {
                    vec.push_back((chan, time_now()));
                }

                if let Some((_, last_send)) = vec.front() {
                    return max(last_send + 10000 - time_now(), 0);
                }
            }
            5000
        };

        loop {
            thread::sleep(Duration::from_millis(calc_wait_time() as u64));
        }
    });

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
