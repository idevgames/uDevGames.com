use crate::db::DbPool;
use thiserror::Error;


/// An error common to model helper functions.
#[derive(Error, Debug)]
pub enum ModelError {
    /// Failed to get a database connection.
    #[error("Couldn't get out of the pool with error {0}. Send a lifeguard.")]
    PoolError(#[from] diesel::r2d2::PoolError),

    /// Failed to query the database, or no result from the database when one
    /// was expected.
    #[error("Couldn't query the database with error {0}. Send a DBA.")]
    DieselError(#[from] diesel::result::Error)
}

/// Local cache of part of Github's understanding of who a user is. Particularly
/// the id, which persists accross use renames, and the user's login, which
/// is a human-readable name for the user.
#[derive(Debug, Queryable)]
pub struct GhUserRecord {
    /// A unique id for this user, supplied by Github and used here as a primary
    /// key. While a layer of indirection here feels like it would grant some
    /// security, if someone can spoof this Github id, they can assume the
    /// identity of any uDevGames user anyway.
    pub id: i64,

    /// The user's human-readable name.
    pub login: String,

    /// Url of this user's picture.
    pub avatar_url: String,
    
    /// Url of this user's Github profile.
    pub html_url: String
}

impl GhUserRecord {
    /// Finds or creates a GhUserRecord in the database with the given gh_id,
    /// and ensures that it has the given attributes.
    pub fn find_and_update(
        pool: &DbPool, gh_id: i64, gh_login: &str, gh_avatar_url: &str,
        gh_html_url: &str
    ) -> Result<GhUserRecord, ModelError> {
        use crate::schema::gh_user_records::dsl::{
            gh_user_records,
            id, login, avatar_url, html_url
        };
        use diesel::prelude::*;

        match GhUserRecord::find_by_id(&pool, gh_id)? {
            Some(_) => {
                // TODO: if the record matches what is already on file, skip the update
                diesel::update(gh_user_records.find(gh_id))
                    .set((
                        login.eq(gh_login),
                        avatar_url.eq(gh_avatar_url),
                        html_url.eq(gh_html_url)
                    ))
                    .execute(&pool.get()?)?;
            },
            None => {
                diesel::insert_into(gh_user_records)
                    .values((
                        id.eq(gh_id),
                        login.eq(gh_login),
                        avatar_url.eq(gh_avatar_url),
                        html_url.eq(gh_html_url)
                    ))
                    .execute(&pool.get()?)?;
            }
        };
        
        Ok(GhUserRecord::find_by_id(&pool, gh_id)?.unwrap())
    }

    /// Finds a given GhUserRecord by its id.
    pub fn find_by_id(
        pool: &DbPool, gh_user_id: i64
    ) -> Result<Option<GhUserRecord>, ModelError> {
        use diesel::prelude::*;
        use crate::schema::gh_user_records::dsl::*;

        let conn = pool.get()?;
        
        let user_record = gh_user_records
            .filter(id.eq(gh_user_id))
            .limit(1)
            .first::<GhUserRecord>(&conn);

        r_to_opt(user_record)
    }

    /// Finds a given GhUserRecord by its login.
    pub fn find_by_login(
        pool: &DbPool, gh_login: &str
    ) -> Result<Option<GhUserRecord>, ModelError> {
        use diesel::prelude::*;
        use crate::schema::gh_user_records::dsl::*;

        let conn = pool.get()?;

        let user_record = gh_user_records
            .filter(login.eq(gh_login))
            .limit(1)
            .first::<GhUserRecord>(&conn);

        r_to_opt(user_record)
    }
}

/// Permissions sloppily model, well, permissions. A GhUserRecord may "have"
/// zero or more permissions. Permissions are known by their name, which is
/// special and hard-coded into various parts of the website. For example,
/// having the "admin" permission enables some UI that other users cannot see.
/// Or having the "banned" permission prevents a user from all site
/// participation.
#[derive(Debug, Queryable)]
pub struct Permission {
    /// Id of this permission grant.
    pub id: i32,

    /// The user id who this permission is granted to.
    pub gh_user_id: i64,

    /// The name of the permission granted.
    pub name: String
}

impl Permission {
    /// Finds all permissions on a given user.
    pub fn find_by_gh_user_id(
        pool: &DbPool, user_id: i64
    ) -> Result<Vec<Permission>, ModelError> {
        use diesel::prelude::*;
        use crate::schema::permissions::dsl::*;

        let conn = pool.get()?;

        let perms = permissions
            .filter(gh_user_id.eq(user_id))
            .load::<Permission>(&conn)?;
        
        Ok(perms)
    }

    /// Finds all permissions with a given name, or in other domain language
    /// this describes all users with a specific permission.
    pub fn find_by_name(
        pool: &DbPool, permission_name: &str
    ) -> Result<Vec<Permission>, ModelError> {
        use diesel::prelude::*;
        use crate::schema::permissions::dsl::*;

        let conn = pool.get()?;

        let perms = permissions
            .filter(name.eq(permission_name))
            .load::<Permission>(&conn)?;

        Ok(perms)
    }
}

/// Converts a diesel result, which packages the absence of a record as an
/// error, into an Option, which makes dealing with "I'm okay with something not
/// being present" slightly more Rustic.
fn r_to_opt<T>(
    r: Result<T, diesel::result::Error>
) -> Result<Option<T>, ModelError> {
    match r {
        Ok(t) => Ok(Some(t)),
        Err(diesel::NotFound) => Ok(None),
        Err(e) => Err(ModelError::DieselError(e))
    }
}
