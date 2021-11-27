use hyper::body::{Bytes, Sender};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use lazy_static::lazy_static;
use std::cmp::max;
use std::collections::{HashMap, VecDeque};
use std::convert::Infallible;
use std::ffi::OsStr;
use std::fs;
use std::fs::read_dir;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

lazy_static! {
    static ref SENDERS: Arc<Mutex<VecDeque<(Sender, u128)>>> =
        Arc::new(Mutex::new(VecDeque::new()));
    static ref STATIC_FILES: HashMap<String, Vec<u8>> = init_static_files();
}

fn init_static_files() -> HashMap<String, Vec<u8>> {
    let mut static_files = HashMap::new();
    for path in read_dir("./public/").unwrap() {
        let path = path.unwrap();
        if path.file_type().unwrap().is_file() {
            static_files.insert(
                format!("/{}", OsStr::to_string_lossy(path.file_name().as_os_str())),
                fs::read(path.path()).unwrap(),
            );
        }
    }
    static_files
}

fn time_now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if req.method() != &Method::GET {
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        return Ok(response);
    }

    let path = if req.uri().path() == "/" {
        "/index.html"
    } else {
        req.uri().path()
    };

    if path == "/lisa.jpg" {
        let (mut chan, body) = Body::channel();
        let img_data = STATIC_FILES.get("/lisa.jpg").unwrap();
        if chan.send_data(Bytes::from(&img_data[..])).await.is_ok() {
            SENDERS.lock().unwrap().push_back((chan, time_now()));
        }
        return Ok(Response::builder()
            .header("X-Accel-Buffering", "no")
            .body(body)
            .unwrap());
    }

    if let Some(contents) = STATIC_FILES.get(path) {
        return Ok(Response::new(Body::from(&contents[..])));
    }

    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::NOT_FOUND;
    Ok(response)
}

#[tokio::main]
async fn main() {
    // We'll bind to 127.0.0.1:3000
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

    // println!("{:?}", *STATIC_FILES);

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
