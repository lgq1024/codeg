use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ConversationTurn::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ConversationTurn::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ConversationTurn::ConversationId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationTurn::Role)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationTurn::ContentJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ConversationTurn::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ConversationTurn::UsageJson).text())
                    .col(ColumnDef::new(ConversationTurn::DurationMs).integer())
                    .col(ColumnDef::new(ConversationTurn::Model).string())
                    .col(
                        ColumnDef::new(ConversationTurn::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ConversationTurn::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_conversation_turn_conversation")
                            .from(ConversationTurn::Table, ConversationTurn::ConversationId)
                            .to(Conversation::Table, Conversation::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_conversation_turn_conversation_id")
                    .table(ConversationTurn::Table)
                    .col(ConversationTurn::ConversationId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ConversationTurn::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ConversationTurn {
    Table,
    Id,
    ConversationId,
    Role,
    ContentJson,
    Timestamp,
    UsageJson,
    DurationMs,
    Model,
    SortOrder,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Conversation {
    Table,
    Id,
}
