use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    schema::*,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_type(
                Type::create()
                    .as_enum(OauthProvider::Enum)
                    .values([
                        OauthProvider::Google,
                        OauthProvider::Facebook,
                        OauthProvider::Twitter,
                        OauthProvider::GitHub,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(TokenType::Enum)
                    .values([TokenType::Bearer, TokenType::Mac, TokenType::Extension])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(UserType::Enum)
                    .values([UserType::Admin, UserType::Regular, UserType::Guest])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(string(User::Username).unique_key())
                    .col(string(User::Email).unique_key())
                    .col(
                        ColumnDef::new(User::Type)
                            .custom(UserType::Enum)
                            .not_null()
                            .default(UserType::Regular.to_string()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Password::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Password::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(uuid(Password::UserId).unique_key())
                    .col(string(Password::Content))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Password::Table, Password::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RefreshTokens::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(date_time(RefreshTokens::ExpiresAt))
                    .col(boolean(RefreshTokens::Revoked).not_null().default(false))
                    .col(uuid(RefreshTokens::UserId))
                    .col(string(RefreshTokens::DeviceType).null())
                    .col(string(RefreshTokens::Os).null())
                    .col(string(RefreshTokens::Engine).null())
                    .col(string(RefreshTokens::UserAgent))
                    .foreign_key(
                        ForeignKey::create()
                            .from(RefreshTokens::Table, RefreshTokens::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Oauth2::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Oauth2::Id)
                            .uuid()
                            .not_null()
                            .default(Expr::cust("gen_random_uuid()"))
                            .primary_key(),
                    )
                    .col(uuid(Oauth2::UserId))
                    .col(
                        ColumnDef::new(Oauth2::OauthProvider)
                            .custom(OauthProvider::Enum)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Oauth2::OauthType)
                            .custom(TokenType::Enum)
                            .not_null(),
                    )
                    .col(date_time(Oauth2::ExpiresAt).null())
                    .col(json_binary(Oauth2::Scopes))
                    .col(string(Oauth2::ProviderUserId))
                    .col(string(Oauth2::RefreshToken).null())
                    .col(string(Oauth2::AccessToken))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Oauth2::Table, Oauth2::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .drop_table(Table::drop().table(Password::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Oauth2::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(RefreshTokens::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(OauthProvider::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(UserType::Enum).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(TokenType::Enum).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    Username,
    #[sea_orm(iden = "user_type")]
    Type,
}

#[derive(DeriveIden)]
enum Password {
    Table,
    Id,
    UserId,
    Content,
}

#[derive(DeriveIden)]
enum Oauth2 {
    Table,
    Id,
    UserId,
    OauthProvider,
    RefreshToken,
    AccessToken,
    OauthType,
    ExpiresAt,
    ProviderUserId,
    Scopes,
}

#[derive(DeriveIden)]
enum OauthProvider {
    #[sea_orm(iden = "oauth_provider")]
    Enum,
    Google,
    Facebook,
    Twitter,
    #[sea_orm(iden = "github")]
    GitHub,
}

#[derive(DeriveIden)]
enum RefreshTokens {
    Table,
    Id,
    ExpiresAt,
    Revoked,
    UserId,

    // Device Info stuff
    // TODO: Might be a good idea to have this as a different model
    DeviceType,
    Os,
    Engine,
    UserAgent,
}

#[derive(DeriveIden)]
enum TokenType {
    #[sea_orm(iden = "token_type")]
    Enum,
    Bearer,
    Mac,
    Extension,
}

#[derive(DeriveIden)]
enum UserType {
    #[sea_orm(iden = "user_type")]
    Enum,
    Admin,
    Regular,
    Guest,
}
