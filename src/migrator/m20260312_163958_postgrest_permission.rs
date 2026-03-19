// src/migrator/m20260312_163958_postgrest_permission.rs (create new file)

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260312_163958_postgrest_permission"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let authenticator_password =
            std::env::var("AUTHENTICATOR_PASSWORD").expect("AUTHENTICATOR_PASSWORD must be set");

        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
                    CREATE ROLE web_anon NOLOGIN;
                    CREATE ROLE authenticator NOINHERIT LOGIN PASSWORD '{}';
                    "#,
                authenticator_password
            ))
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                GRANT USAGE ON SCHEMA public TO web_anon;
                GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO web_anon;
                GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO web_anon;
                REVOKE ALL ON TABLE seaql_migrations FROM web_anon;
                "#,
            )
            .await?;
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                GRANT web_anon TO authenticator;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP ROLE IF EXISTS web_anon;
                DROP ROLE IF EXISTS authenticator;
                "#,
            )
            .await?;

        Ok(())
    }
}
