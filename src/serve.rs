use crate::{ db::DbManager, gh_oauth::GhCredentials };
use rocket::routes;
use rocket::config::{ Config as RocketConfig, Environment as RocketEnvironment, };
use rocket_contrib::templates::Template;
use rocket_contrib::serve::{crate_relative, StaticFiles};


pub async fn serve(
    address: String, port: u16, workers: u16,
    connection_manager: DbManager,
    gh_credentials: GhCredentials
    /* there's gonna be some databasey nonsense here, too */
) {
    let config = RocketConfig::build(RocketEnvironment::Production)
        .address(address).port(port).workers(workers)
        .expect("the configuration is bad!");

    let _ = rocket::custom(config)
        .manage(gh_credentials)
        .manage(crate::gh_oauth::gh_client())
        .manage(connection_manager)
        .attach(Template::fairing())
        // TODO: compression fairing would be nice here
        .mount("/", routes![
            crate::homepage::homepage,
            crate::gh_oauth::login_with_github,
            crate::gh_oauth::gh_callback,
        ])
        .mount("/static", StaticFiles::from(crate_relative!("/static")))
        .launch()
        .await;
}
