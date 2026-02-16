use sqlx::PgPool;
use async_trait::async_trait;
use chrono::Utc;

use crate::domain::entities::TestItem;
use crate::errors::ApiError;
use crate::interfaces::repositories::TestItemRepository;

pub struct PostgresTestItemRepository {
    pool: PgPool,
}

impl PostgresTestItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TestItemRepository for PostgresTestItemRepository {
    async fn create(&self, item: &TestItem) -> Result<TestItem, ApiError> {
        let query = r#"
            INSERT INTO test_items (id, subject, optional_field, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
        "#;

        let created_item = sqlx::query_as::<_, TestItem>(query)
            .bind(&item.id)
            .bind(&item.subject)
            .bind(&item.optional_field)
            .bind(item.created_at)
            .bind(item.updated_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(created_item)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<TestItem>, ApiError> {
        let query = "SELECT * FROM test_items WHERE id = $1";

        let item = sqlx::query_as::<_, TestItem>(query)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(item)
    }

    async fn get_all(&self) -> Result<Vec<TestItem>, ApiError> {
        let query = "SELECT * FROM test_items ORDER BY created_at DESC";

        let items = sqlx::query_as::<_, TestItem>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(items)
    }

    async fn get_paginated(&self, page: i32, per_page: i32) -> Result<(Vec<TestItem>, i32), ApiError> {
        let offset = (page - 1) * per_page;

        let query = r#"
            SELECT * FROM test_items
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
        "#;

        let items = sqlx::query_as::<_, TestItem>(query)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        let total = self.count().await?;

        Ok((items, total))
    }

    async fn update(&self, item: &TestItem) -> Result<(), ApiError> {
        let query = r#"
            UPDATE test_items
            SET subject = $1, optional_field = $2, updated_at = $3
            WHERE id = $4
        "#;

        sqlx::query(query)
            .bind(&item.subject)
            .bind(&item.optional_field)
            .bind(Utc::now())
            .bind(&item.id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<bool, ApiError> {
        let query = "DELETE FROM test_items WHERE id = $1";

        let result = sqlx::query(query)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }

    async fn count(&self) -> Result<i32, ApiError> {
        let query = "SELECT COUNT(*) as count FROM test_items";

        let row: (i64,) = sqlx::query_as(query)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(row.0 as i32)
    }
}