extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate bytes;

use futures::{future, Future};
use hyper::{Body, Client, Method, Request, Response, Server, StatusCode, Uri, header};
use hyper::client::HttpConnector;
use hyper::service::service_fn;
use bytes::{Bytes};

static NOTFOUND: &[u8] = b"Not Found";
static HTTPBIN: &str = "httpbin.org";
static SCHEME: &str = "http";
static GET_PATH: &str = "get";
static POST_PATH: &str = "post";

fn do_get(query: Option<&str>, client: &Client<HttpConnector>)
    -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>
{
    let path_and_query = match query {
        Some(q) => Bytes::from(format!("{}?{}", GET_PATH, q)),
        None => Bytes::from(GET_PATH)
    };

    let uri = Uri::builder()
        .scheme(SCHEME)
        .authority(HTTPBIN)
        .path_and_query(path_and_query)
        .build()
        .unwrap();

    let web_resp_future = client.get(uri);

    Box::new(web_resp_future.map(|resp| {
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(resp.into_body())
            .unwrap()
    }))
}

fn do_post(body: Body, client: &Client<HttpConnector>)
    -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>
{
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}://{}/{}", SCHEME, HTTPBIN, POST_PATH))
        .body(body)
        .unwrap();

    let web_resp_future = client.request(request);

    Box::new(web_resp_future.map(|resp| {
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(resp.into_body())
            .unwrap()
    }))
}

fn response_examples(req: Request<Body>, client: &Client<HttpConnector>)
    -> Box<Future<Item=Response<Body>, Error=hyper::Error> + Send>
{
    match (req.method(), req.uri().path(), req.uri().query()) {
        (&Method::GET, _, query) => do_get(query, client),
        (&Method::POST, _, _) => do_post(req.into_body(), client),
        _ => Box::new(future::ok(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(NOTFOUND))
                .unwrap())
        )
    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();

    // no idea what I'm doing here. Just copying this bit from the internets
    hyper::rt::run(future::lazy(move || {
        // Share a `Client` with all `Service`s
        let client = Client::new();

        let new_service = move || {
            // Move a clone of `client` into the `service_fn`.
            let client = client.clone();
            service_fn(move |req| {
                response_examples(req, &client)
            })
        };

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("server error: {}", e));

        println!("Listening on http://{}", addr);

        server
    }));
}
