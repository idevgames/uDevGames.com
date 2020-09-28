use clap::{ Clap, crate_authors, crate_version };


#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!())]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand
}

#[derive(Clap)]
pub enum SubCommand {
    Migrate(Migrate),
    Serve(Serve),
    Permission(Permission)
}

/// Migrates the uDevGames database to the current schema
#[derive(Clap)]
pub struct Migrate { }

/// Starts the uDevGames website
#[derive(Clap)]
pub struct Serve { }

/// Grant, revoke, and show permissions given to users
#[derive(Clap)]
pub struct Permission {
    #[clap(subcommand)]
    pub subcmd: PermissionSubCommand
}

#[derive(Clap)]
pub enum PermissionSubCommand {
    Grant(PermissionGrant),
    Revoke(PermissionRevoke),
    Show(PermissionShow)
}

/// Grants a permission to a user
#[derive(Clap)]
pub struct PermissionGrant {
    /// The user to grant the permission to, either by @login or numeric id
    #[clap(short, long)]
    pub user: String,

    /// The permission to grant to the user
    #[clap(short, long)]
    pub permission: String
}

/// Revokes a permission from a user
#[derive(Clap)]
pub struct PermissionRevoke {
    /// The user to revoke the permission from, either by @login or numeric id
    #[clap(short, long)]
    pub user: String,

    /// The permission to revoke from the user
    #[clap(short, long)]
    pub permission: String
}

/// Show permissions for a user, or users with a permission
#[derive(Clap)]
pub struct PermissionShow {
    /// Show all permissions for this user, either by @login or numeric id
    #[clap(long, short)]
    pub user: Option<String>,

    /// Show all users with this permission
    #[clap(short, long)]
    pub permission: Option<String>
}
