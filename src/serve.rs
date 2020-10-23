use crate::{
    attachments::AttachmentStorage, db::DbPool, gh_oauth::GhCredentials,
};
use rocket::{
    figment::Figment,
    routes,
    config::Config as RocketConfig,
};
use rocket_contrib::{
//    compression::Compression,
    helmet::SpaceHelmet,
    templates::Template,
    serve::{ crate_relative, StaticFiles, },
};


pub async fn serve(
    address: String, port: u16, workers: u16, secret: String, db_pool: DbPool,
    gh_credentials: GhCredentials, attachment_storage: AttachmentStorage
) {
    let config = Figment::from(RocketConfig::default())
        .merge(("address", address))
        .merge(("port", port))
        .merge(("workers", workers))
        .merge(("secret_key", secret));

    let _ = rocket::custom(config)
        .manage(gh_credentials)
        .manage(crate::gh_oauth::gh_client())
        .manage(db_pool)
        .manage(attachment_storage)
        .attach(Template::fairing())
//        .attach(Compression::fairing())
        .attach(SpaceHelmet::default())
        .mount("/", routes![
            crate::homepage::homepage,
            crate::attachments::get_attachment,
            crate::gh_oauth::login_with_github,
            crate::gh_oauth::gh_callback,
            crate::gh_oauth::logout,
        ])
        .mount("/static", StaticFiles::from(crate_relative!("/static")))
        .launch()
        .await;
}

