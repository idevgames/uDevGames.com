use crate::{
    db::DbConn,
    models::{r_to_opt, ModelError},
};

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
    pub name: String,
}

impl Permission {
    /// Finds all permissions on a given user.
    pub fn find_by_gh_user_id(
        conn: &DbConn,
        user_id: i64,
    ) -> Result<Vec<Permission>, ModelError> {
        use crate::schema::permissions::dsl::*;
        use diesel::prelude::*;

        let perms = permissions
            .filter(gh_user_id.eq(user_id))
            .load::<Permission>(conn)?;

        Ok(perms)
    }

    /// Finds all permissions with a given name, or in other domain language
    /// this describes all users with a specific permission.
    pub fn find_by_name(
        conn: &DbConn,
        permission_name: &str,
    ) -> Result<Vec<Permission>, ModelError> {
        use crate::schema::permissions::dsl::*;
        use diesel::prelude::*;

        let perms = permissions
            .filter(name.eq(permission_name))
            .load::<Permission>(conn)?;

        Ok(perms)
    }

    /// Grant a permission to a user by id.
    pub fn grant_permission(
        conn: &DbConn,
        user_id: i64,
        permission_name: &str,
    ) -> Result<(), ModelError> {
        use crate::schema::permissions::dsl::*;
        use diesel::prelude::*;

        // if an existing equivalent permission exists, nop
        let existing_permission = Permission::find_by_user_id_and_name(
            &conn,
            user_id,
            &permission_name,
        )?;

        if existing_permission.is_some() {
            return Ok(());
        }

        // no existing permission, make a new one
        diesel::insert_into(permissions)
            .values((gh_user_id.eq(user_id), name.eq(permission_name)))
            .execute(conn)?;

        Ok(())
    }

    /// Revoke a permission from a user.
    pub fn revoke_permission(
        conn: &DbConn,
        user_id: i64,
        permission_name: &str,
    ) -> Result<usize, ModelError> {
        use crate::schema::permissions::dsl::*;
        use diesel::prelude::*;

        let r = diesel::delete(
            permissions
                .filter(gh_user_id.eq(user_id))
                .filter(name.eq(permission_name)),
        )
        .execute(conn)?;

        Ok(r)
    }

    /// Find a permission by both user id and name.
    pub fn find_by_user_id_and_name(
        conn: &DbConn,
        user_id: i64,
        permission_name: &str,
    ) -> Result<Option<Permission>, ModelError> {
        use crate::schema::permissions::dsl::*;
        use diesel::prelude::*;

        let perm = permissions
            .filter(gh_user_id.eq(user_id))
            .filter(name.eq(permission_name))
            .limit(1)
            .first::<Permission>(conn);

        r_to_opt(perm)
    }
}
