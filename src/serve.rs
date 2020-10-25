use crate::{attachments::AttachmentStorage, controllers::gh_oauth::GhCredentials, db::DbPool};
use rocket::{config::Config as RocketConfig, figment::Figment, routes};
use rocket_contrib::{
    //    compression::Compression,
    helmet::SpaceHelmet,
    serve::{crate_relative, StaticFiles},
    templates::Template,
};

pub async fn serve(
    address: String,
    port: u16,
    workers: u16,
    secret: String,
    db_pool: DbPool,
    gh_credentials: GhCredentials,
    attachment_storage: AttachmentStorage,
) {
    let config = Figment::from(RocketConfig::default())
        .merge(("address", address))
        .merge(("port", port))
        .merge(("workers", workers))
        .merge(("secret_key", secret));

    let _ = rocket::custom(config)
        .manage(gh_credentials)
        .manage(crate::controllers::gh_oauth::gh_client())
        .manage(db_pool)
        .manage(attachment_storage)
        .attach(Template::fairing())
        //        .attach(Compression::fairing())
        .attach(SpaceHelmet::default())
        .mount(
            "/",
            routes![
                crate::controllers::homepage::homepage,
                crate::controllers::attachments::get_attachment,
                crate::controllers::gh_oauth::login_with_github,
                crate::controllers::gh_oauth::gh_callback,
                crate::controllers::gh_oauth::logout,
            ],
        )
        .mount("/static", StaticFiles::from(crate_relative!("/static")))
        .launch()
        .await;
}
