use crate::db::DbPool;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Couldn't get out of the pool. Send a lifeguard.")]
    PoolError(#[from] diesel::r2d2::PoolError),
    #[error("Couldn't query the database. Send a DBA.")]
    DieselError(#[from] diesel::result::Error)
}

#[derive(Debug, Queryable)]
pub struct GhUserRecord {
    id: i64,
    login: String,
    avatar_url: String,
    html_url: String
}

impl GhUserRecord {

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
            Some(record) => {
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

        match user_record {
            Ok(gh_user_record) => Ok(Some(gh_user_record)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(ModelError::DieselError(e))
        }
    }
}
