// do not delete https://github.com/diesel-rs/diesel/issues/1894
#[macro_use]
extern crate diesel_migrations;

mod db;
mod homepage;
mod gh_oauth;
mod migrate;
mod serve;
mod template_context;

use crate::gh_oauth::GhCredentials;
use dotenv::dotenv;
use std::env;


#[rocket::main]
async fn main() {
    // load config from a .env file, really only applicable for development
    dotenv().ok();

    let database_path = expect_env_string("APP_DATABASE_PATH");
    let connection_manager = crate::db::get_manager(&database_path);

    // should we migrate?
    

    // if we're not migrating, we're running a service
    let gh_credentials = GhCredentials {
        client_id: expect_env_string("GH_CLIENT_ID"),
        client_secret: expect_env_string("GH_CLIENT_SECRET"),
    };

    crate::serve::serve(
        expect_env_string("UDEVGAMES_APP_ADDRESS"),
        expect_env_u16("UDEVGAMES_APP_PORT"),
        expect_env_u16("UDEVGAMES_APP_WORKERS"),
        connection_manager,
        gh_credentials
    ).await;
}

fn expect_env_string(var: &str) -> String {
    env::var(var).expect(format!(
        "Please provide {} as an environment var or in a .env", var
    ).as_str())
}

fn expect_env_u16(var: &str) -> u16 {
    let string = expect_env_string(var);

    string.parse().expect(format!(
        "Expected u16 in env var {}, but {} cannot parse as a u16",
        var, string
    ).as_str())
}
