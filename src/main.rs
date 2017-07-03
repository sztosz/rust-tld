#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate hyper;

use std::io::Read;
use std::sync::RwLock;
use rocket::State;
use hyper::Client;
use hyper::client::Response;

struct TLD {
    tld: RwLock<Vec<String>>,
}

#[get("/")]
fn index() -> &'static str {
    "A simple service to check if given domain ending ie. .com, .org. .latin is a registered by ICANN as Top Level Domain (TLD for short)"
}

#[get("/favicon.ico")]
fn favicon() -> &'static str {
    "Am I preety?"
}

#[get("/<tld>")]
fn check_tld(tld: &str, state: State<TLD>) -> &'static str {
    let tld = &tld.to_lowercase();
    let found = state.tld.read().unwrap().iter().any(|x| x == tld);
    if found { "FOUND" } else { "NOT FOUND" }
}

#[get("/update")]
fn update(state: State<TLD>) -> &'static str {
    match get_update_from_url() {
        Some(new_state) => {
            *state.tld.write().unwrap() = new_state;
            "Update successful"
        }
        None => "Update Failed",
    }
}

fn get_update_from_url() -> Option<Vec<String>> {
    let client = Client::new();
    match client
        .get("http://data.iana.org/TLD/tlds-alpha-by-domain.txt")
        .send() {
        Ok(response) => {
            match parse_update_to_vec(response) {
                Some(res) => Some(res),
                None => None,
            }
        }
        Err(_) => None,
    }
}

fn parse_update_to_vec(mut response: Response) -> Option<Vec<String>> {
    let mut buf = String::new();
    match response.read_to_string(&mut buf) {
        Ok(_) => Some(string_to_vec(&buf)),
        Err(_) => None,
    }
}

fn string_to_vec(response: &str) -> Vec<String> {
    response.lines().map(|x| x.to_lowercase()).collect()
}

fn main() {
    match get_update_from_url() {
        Some(new_state) => {
            rocket::ignite()
                .manage(TLD { tld: RwLock::new(new_state) })
                .mount("/", routes![index, favicon, check_tld, update])
                .launch();
        }
        None => panic!("Geting TLD list from IANA failed!"),
    }
}
