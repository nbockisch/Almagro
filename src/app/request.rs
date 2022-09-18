//! # request
//!
//! The `request` crate contains structures and methods for running the HTTP
//! requests.

use reqwest::{
    blocking,
    Method,
};
use serde::{
    Deserialize,
    Serialize,
};

/// Number of request fields that can be edited in the UI
pub const REQ_FIELD_COUNT: usize = 4;

/// Contains the data for an HTTP request
#[derive(Serialize, Deserialize)]
pub struct Request {
    // The name of the request seen in the UI
    pub name: String,
    pub req_type: String, // String for serialization, gets converted to Method
    pub url: String,
    pub body: String,
    pub resp: String,
    pub status: String,
    pub db_id: String, // Id used to access request in JSON database
}

impl Request {
    /// Create a new blank request
    ///
    /// Takes in a name for the request
    pub fn new<T: ToString>(name: T) -> Self {
        Self {
            name: name.to_string(),
            req_type: Method::GET.to_string(),
            url: String::new(),
            body: String::new(),
            resp: String::new(),
            status: String::new(),
            db_id: String::new(),
        }
    }

    /// Run the request and get the response
    pub fn run_req(&mut self) {
        let client = blocking::Client::new();

        // Get the response text or error message if any
        match Method::from_bytes(self.req_type.as_bytes()) {
            Ok(method) => match client.request(method, self.url.clone())
                .body(self.body.clone())
                .send(){
                    Ok(req) => {
                        self.status = String::from(req.status().as_str().clone());
                        self.resp = match &req.text() {
                            Ok(text) => text.to_string(),
                            Err(e) => format!("{}", e),
                        };
                    },
                    Err(e) => {
                        self.resp = format!("{}", e);
                        self.status = String::from("Error");
                    },
                },
            Err(e) => {
                self.resp = format!("{}", e);
                self.status = String::from("Error");
            },
        };
    }
}
