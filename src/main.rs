#[macro_use]
extern crate iron;

mod api;

use std::io::Read;

use iron::prelude::*;
use iron::method;
use iron::status;

use api::ApiRequest;

fn handle_request(request: &mut Request) -> IronResult<Response> {
    match request.method {
        method::Post => (),
        _ => return Ok(Response::with(status::MethodNotAllowed)),
    }

    let query = iexpect!(request.url.query.as_ref());
    let req_type = iexpect!(ApiRequest::parse(query));
    println!("{:?}", req_type);

    let mut body = String::new();
    itry!(request.body.read_to_string(&mut body));
    println!("{}", body);

    Ok(Response::with((status::Ok, "{\"api_version\":1,\"auth\":1}")))
}

fn main() {
    Iron::new(handle_request).http("localhost:3000").unwrap();
}
