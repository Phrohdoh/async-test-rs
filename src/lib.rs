extern crate hyper;
extern crate reqwest;
extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

use futures::future::{ok, Future};
use futures::Stream;

use hyper::server::{Http, Request, Response, Service};
use hyper::{Chunk, Method, StatusCode};

use reqwest::unstable::async::{Client};

use std::time::Duration;

#[derive(Clone)]
struct MyServer {
    handle: tokio_core::reactor::Handle,
}

impl Service for MyServer {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        match (req.method(), req.path()) {
            (&Method::Get, "/first") => {
                let server = self.clone();
                Box::new(req.body().concat2().map(move |chunk| server.first(chunk)))
            },
            (&Method::Get, "/second") => {
                let server = self.clone();
                Box::new(req.body().concat2().map(move |chunk| server.second(chunk)))
            },
            (&Method::Get, "/") => Box::new(ok(Response::new().with_status(StatusCode::NotFound).with_body(r#"{"error":"You want to make a GET request to /first"}"#))),
            _ => Box::new(ok(Response::new().with_status(StatusCode::NotFound))),
        }
    }
}

impl MyServer {
    fn first(&self, _chunk: Chunk) -> Response {
        let client_handle = self.handle.clone();

        let sleep_then_req = tokio_timer::Timer::default()
        .sleep(Duration::from_millis(3000))
        .then(move |_| {
            println!("Making a GET request to /second");
            let client = Client::new(&client_handle);

            client.get("http://127.0.0.1:3000/second")
            .send()
            .map_err(|_| unimplemented!())
            .and_then(|mut resp| {
                println!("/second returned: {}", resp.status());
                let body = std::mem::replace(resp.body_mut(), reqwest::unstable::async::Decoder::empty());
                body.concat2().map_err(|_| unimplemented!())
            }).and_then(|body| {
                let body = String::from_utf8(body.to_vec()).expect("Oops, that's invalid UTF-8!");
                println!("/second body: {}", body);
                Ok::<(), ()>(())
            })
        });

        self.handle.spawn(sleep_then_req);
        Response::new().with_body(r#"{"status":"done","origin":"first"}"#)
    }

    fn second(&self, _chunk: Chunk) -> Response {
        println!(":: In the body of `second`");
        Response::new().with_body(r#"{"status":"done","origin":"second"}"#)
    }
}

pub fn run(addr: &str) {
    let addr = addr.parse().unwrap();
    println!("Running server on {}", addr);

    let http = Http::new();
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let hnd = core.handle();
    let listener = tokio_core::net::TcpListener::bind(&addr, &hnd).unwrap();

    core.run(listener.incoming().for_each(move |(sock, addr)| {
        let my_server = MyServer { handle: hnd.clone() };
        http.bind_connection(&hnd, sock, addr, my_server);
        Ok(())
    }));
}