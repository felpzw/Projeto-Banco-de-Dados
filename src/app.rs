// Import the reqwest library
use reqwest::Client;

use dotenv::from_filename;

#[derive(Clone)]
// Extend this struct with the feature you will need for your application
pub struct ApplicationState {
    // This will be available to all your route handlers
    pub fetch: Client,
}

pub fn main() -> ApplicationState {
    from_filename("var.env").ok().expect("Error to load .env");

    let fetch = Client::new();
    return ApplicationState { fetch };
}