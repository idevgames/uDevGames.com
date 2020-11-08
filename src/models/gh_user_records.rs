use crate::{
    db::DbConn,
    models::{r_to_opt, ModelError},
};

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
    pub html_url: String,
}

impl GhUserRecord {
    /// Finds or creates a GhUserRecord in the database with the given gh_id,
    /// and ensures that it has the given attributes.
    pub fn find_and_update(
        conn: &DbConn,
        gh_id: i64,
        gh_login: &str,
        gh_avatar_url: &str,
        gh_html_url: &str,
    ) -> Result<GhUserRecord, ModelError> {
        use crate::schema::gh_user_records::dsl::{
            avatar_url, gh_user_records, html_url, id, login,
        };
        use diesel::prelude::*;

        match GhUserRecord::find_by_id(conn, gh_id)? {
            Some(u) => {
                if gh_login != u.login
                    || gh_avatar_url != u.avatar_url
                    || gh_html_url != u.html_url
                {
                    diesel::update(gh_user_records.find(gh_id))
                        .set((
                            login.eq(gh_login),
                            avatar_url.eq(gh_avatar_url),
                            html_url.eq(gh_html_url),
                        ))
                        .execute(conn)?;
                }
            }
            None => {
                diesel::insert_into(gh_user_records)
                    .values((
                        id.eq(gh_id),
                        login.eq(gh_login),
                        avatar_url.eq(gh_avatar_url),
                        html_url.eq(gh_html_url),
                    ))
                    .execute(conn)?;
            }
        };

        Ok(GhUserRecord::find_by_id(&conn, gh_id)?.unwrap())
    }

    /// Finds a given GhUserRecord by its id.
    pub fn find_by_id(
        conn: &DbConn,
        gh_user_id: i64,
    ) -> Result<Option<GhUserRecord>, ModelError> {
        use crate::schema::gh_user_records::dsl::*;
        use diesel::prelude::*;

        let user_record = gh_user_records
            .filter(id.eq(gh_user_id))
            .limit(1)
            .first::<GhUserRecord>(conn);

        r_to_opt(user_record)
    }

    /// Finds a given GhUserRecord by its login.
    pub fn find_by_login(
        conn: &DbConn,
        gh_login: &str,
    ) -> Result<Option<GhUserRecord>, ModelError> {
        use crate::schema::gh_user_records::dsl::*;
        use diesel::prelude::*;

        let user_record = gh_user_records
            .filter(login.eq(gh_login))
            .limit(1)
            .first::<GhUserRecord>(conn);

        r_to_opt(user_record)
    }
}
