// do not delete https://github.com/diesel-rs/diesel/issues/1894
#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

mod attachments;
mod cliopts;
mod controllers;
mod db;
mod error_handlers;
mod migrate;
mod models;
mod schema;
mod serve;
mod template_helpers;

use crate::attachments::AttachmentStorage;
use crate::cliopts::{Opts, SubCommand};
use crate::controllers::gh_oauth::GhCredentials;
use crate::db::{get_pool, DbConn, DbPool};
use crate::models::{GhUserRecord, ModelError};
use clap::Clap;
use dotenv::dotenv;
use std::convert::TryFrom;
use std::env;
use std::num::ParseIntError;
use std::path::PathBuf;

#[rocket::main]
async fn main() {
    // load config from a .env file, really only applicable for development
    dotenv().ok();

    let database_path = expect_env_string("APP_DATABASE_PATH");
    let db_pool = get_pool(&database_path);
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Migrate(_) => {
            println!("Migrating the database at {}", database_path);
            crate::db::migrate_db(&db_pool);
        }
        SubCommand::Serve(_) => {
            let attachment_storage = AttachmentStorage::new(PathBuf::from(
                expect_env_string("UDEVGAMES_ATTACHMENT_STORAGE"),
            ));
            let gh_credentials = GhCredentials {
                client_id: expect_env_string("GH_CLIENT_ID"),
                client_secret: expect_env_string("GH_CLIENT_SECRET"),
            };

            crate::serve::serve(
                expect_env_string("UDEVGAMES_APP_ADDRESS"),
                expect_env_u16("UDEVGAMES_APP_PORT"),
                expect_env_u16("UDEVGAMES_APP_WORKERS"),
                expect_env_string("UDEVGAMES_APP_SECRET"),
                db_pool,
                gh_credentials,
                attachment_storage,
            )
            .await;
        }
        SubCommand::Permission(subcmd) => perms_subcmd(&db_pool, subcmd),
    }
}

fn perms_subcmd(pool: &DbPool, subcmd: crate::cliopts::Permission) {
    use crate::cliopts::PermissionSubCommand;
    use crate::models::Permission;

    let conn = pool
        .get()
        .expect("Could not get a connection from the pool");

    match subcmd.subcmd {
        PermissionSubCommand::Grant(grant) => {
            let uid = UserIdentity::try_from(grant.user)
                .expect(
                    "Could not infer user; is your login \
                        prefixed with @?",
                )
                .uid(&conn)
                .expect("Could not query database");
            Permission::grant_permission(&conn, uid, &grant.permission)
                .expect("Could not grant permission");
            println!("Permission granted!");
        }
        PermissionSubCommand::Revoke(revoke) => {
            let uid = UserIdentity::try_from(revoke.user)
                .expect(
                    "Could not infer user; is your login \
                        prefixed with @?",
                )
                .uid(&conn)
                .expect("Could not query database");
            let r =
                Permission::revoke_permission(&conn, uid, &revoke.permission)
                    .expect("Could not revoke permission");
            println!("Revoked {} permissions", r);
        }
        PermissionSubCommand::Show(show) => {
            if show.user.is_some() {
                let uid = UserIdentity::try_from(show.user.unwrap())
                    .expect(
                        "Could not infer user; is your login \
                            prefixed with @?",
                    )
                    .uid(&conn)
                    .expect("Could not query database");
                let perms = Permission::find_by_gh_user_id(&conn, uid)
                    .expect("Could not query db");

                if perms.len() > 0 {
                    println!("Permissions for user {}", uid);
                    for perm in perms {
                        println!("  {}", perm.name);
                    }
                } else {
                    println!("User {} has no permissions", uid);
                }
            } else if show.permission.is_some() {
                let perm = show.permission.unwrap();
                let perms = Permission::find_by_name(&conn, &perm)
                    .expect("Could not query db");

                if perms.len() > 0 {
                    for perm in perms {
                        println!(
                            "User {} has permission {}",
                            perm.gh_user_id, perm.name
                        );
                    }
                } else {
                    println!("No users have the permission {}", perm);
                }
            } else {
                panic!("Please supply either a user or a permission to show");
            }
        }
    }
}

/// A user can be known either by id or by login. This enum abstracts over the
/// two.
enum UserIdentity {
    /// A user as known by login.
    Login(String),

    /// A user as known by id.
    Id(i64),
}

impl UserIdentity {
    /// The uid for this identity.
    fn uid(&self, conn: &DbConn) -> Result<i64, ModelError> {
        match self {
            UserIdentity::Id(id) => Ok(*id),
            UserIdentity::Login(_) => Ok(self
                .find(conn)
                .expect("Could not query db")
                .expect("No such user")
                .id),
        }
    }

    /// Find a GhUserRecord for this UserIdentity, if one exists.
    fn find(&self, conn: &DbConn) -> Result<Option<GhUserRecord>, ModelError> {
        match self {
            UserIdentity::Login(login) => {
                GhUserRecord::find_by_login(conn, login)
            }
            UserIdentity::Id(id) => GhUserRecord::find_by_id(conn, *id),
        }
    }
}

impl TryFrom<String> for UserIdentity {
    type Error = ParseIntError;

    /// From a String, determine whether this user identity is specified as
    /// @login or by numeric id.
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.starts_with('@') {
            true => Ok(UserIdentity::Login(
                s.strip_prefix("@").unwrap().to_string(),
            )),
            false => Ok(UserIdentity::Id(s.parse()?)),
        }
    }
}

fn expect_env_string(var: &str) -> String {
    env::var(var).expect(
        format!("Please provide {} as an environment var or in a .env", var)
            .as_str(),
    )
}

fn expect_env_u16(var: &str) -> u16 {
    let string = expect_env_string(var);

    string.parse().expect(
        format!(
            "Expected u16 in env var {}, but {} cannot parse as a u16",
            var, string
        )
        .as_str(),
    )
}
