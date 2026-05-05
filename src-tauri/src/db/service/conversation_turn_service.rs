use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::db::entities::conversation_turn;
use crate::db::error::DbError;
use crate::models::MessageTurn;

pub async fn save_turns(
    conn: &DatabaseConnection,
    conversation_id: i32,
    turns: &[MessageTurn],
) -> Result<(), DbError> {
    conversation_turn::Entity::delete_many()
        .filter(conversation_turn::Column::ConversationId.eq(conversation_id))
        .exec(conn)
        .await?;

    for (i, turn) in turns.iter().enumerate() {
        let model = conversation_turn::ActiveModel {
            id: NotSet,
            conversation_id: Set(conversation_id),
            role: Set(serde_json::to_string(&turn.role).unwrap_or_default()),
            content_json: Set(serde_json::to_string(&turn.blocks).unwrap_or_default()),
            timestamp: Set(turn.timestamp),
            usage_json: Set(turn.usage.as_ref().map(|u| serde_json::to_string(u).unwrap_or_default())),
            duration_ms: Set(turn.duration_ms.map(|d| d as i64)),
            model: Set(turn.model.clone()),
            sort_order: Set(i as i32),
            created_at: Set(Utc::now()),
        };
        model.insert(conn).await?;
    }

    Ok(())
}

pub async fn append_turn(
    conn: &DatabaseConnection,
    conversation_id: i32,
    turn: &MessageTurn,
) -> Result<(), DbError> {
    let turn_role = serde_json::to_string(&turn.role).unwrap_or_default();
    let turn_content = serde_json::to_string(&turn.blocks).unwrap_or_default();

    // Deduplication: skip if any existing turn in this conversation has
    // identical role + content. This prevents duplicate accumulation when an
    // ACP agent replays historical context on reconnect (e.g. Hermes) where
    // the replayed turn may not be the very last one.
    let existing_duplicate: Option<i32> = conversation_turn::Entity::find()
        .filter(conversation_turn::Column::ConversationId.eq(conversation_id))
        .filter(conversation_turn::Column::Role.eq(&turn_role))
        .filter(conversation_turn::Column::ContentJson.eq(&turn_content))
        .select_only()
        .column(conversation_turn::Column::Id)
        .into_tuple()
        .one(conn)
        .await
        .ok()
        .flatten();

    if existing_duplicate.is_some() {
        return Ok(());
    }

    let max_sort_order: i32 = conversation_turn::Entity::find()
        .filter(conversation_turn::Column::ConversationId.eq(conversation_id))
        .select_only()
        .column_as(conversation_turn::Column::SortOrder.max(), "max_so")
        .into_tuple()
        .one(conn)
        .await
        .ok()
        .flatten()
        .unwrap_or(Some(0))
        .unwrap_or(0);

    let model = conversation_turn::ActiveModel {
        id: NotSet,
        conversation_id: Set(conversation_id),
        role: Set(turn_role),
        content_json: Set(turn_content),
        timestamp: Set(turn.timestamp),
        usage_json: Set(turn.usage.as_ref().map(|u| serde_json::to_string(u).unwrap_or_default())),
        duration_ms: Set(turn.duration_ms.map(|d| d as i64)),
        model: Set(turn.model.clone()),
        sort_order: Set(max_sort_order + 1),
        created_at: Set(Utc::now()),
    };
    model.insert(conn).await?;

    Ok(())
}

pub async fn get_turns(
    conn: &DatabaseConnection,
    conversation_id: i32,
) -> Result<Vec<MessageTurn>, DbError> {
    let rows = conversation_turn::Entity::find()
        .filter(conversation_turn::Column::ConversationId.eq(conversation_id))
        .order_by_asc(conversation_turn::Column::Id)
        .all(conn)
        .await?;

    let mut turns = Vec::new();
    for row in rows {
        let role = serde_json::from_str(&row.role).unwrap_or(crate::models::TurnRole::Assistant);
        let blocks = serde_json::from_str(&row.content_json).unwrap_or_default();
        let usage = row.usage_json.and_then(|u| serde_json::from_str(&u).ok());

        turns.push(MessageTurn {
            id: row.id.to_string(),
            role,
            blocks,
            timestamp: row.timestamp,
            usage,
            duration_ms: row.duration_ms.map(|d| d as u64),
            model: row.model,
        });
    }

    Ok(turns)
}
