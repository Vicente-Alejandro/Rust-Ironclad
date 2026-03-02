# Implementation Guide: Value Objects and Smart Constructors (DDD in Rust)

This guide establishes the standard for creating new entities and objects within the system, ensuring state consistency through the use of Value Objects (VO) and Smart Constructors under Domain-Driven Design (DDD) principles.

## 1. Core Principle: Separation of Responsibilities in Validations

To avoid an "Anemic Domain" and ensure the system never processes invalid data, business rules and validations are divided into three strict layers:

1. **HTTP Layer (DTOs):** Pure syntactic validation (e.g., "string must not be empty", "maximum length of 50 characters"). Fail fast.
2. **Application Layer (Services):** Contextual business rules or those requiring I/O (e.g., "email already exists in database", "verify plain text password before hashing").
3. **Domain Layer (Value Objects and Entities):** Immutable business rules and state integrity (e.g., "username only accepts alphanumeric characters"). Self-validate upon instantiation.

---

## 2. Step-by-Step Implementation

The flow for creating a new resource (e.g., a `Product` or `TestItem`) must follow this strict order, building from the core (Domain) outward (Infrastructure).

### Step 1: Define Value Objects (Domain)

Never use primitive types (`String`, `i32`) to represent domain concepts. Create an encapsulated Value Object.

**Location:** `src/domain/value_objects/property_name.rs`
```rust
use serde::{Deserialize, Serialize};
use crate::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductName(String);

impl ProductName {
    /// Smart Constructor: Executes immutable business rules
    pub fn new(value: String) -> Result<Self, DomainError> {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.len() < 3 {
            return Err(DomainError::Validation(
                "Product name must be at least 3 characters".into()
            ));
        }
        Ok(Self(trimmed.to_string()))
    }

    /// Hydration: Used EXCLUSIVELY by persistence layer (Database)
    /// Assumes data is already valid.
    pub fn from_trusted(value: String) -> Self {
        Self(value)
    }

    /// Extract primitive value for DTOs or Database
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

---

### Step 2: Build the Entity (Domain)

The entity composes Value Objects. Its in-memory instantiation must always result in a 100% valid object.

**Location:** `src/domain/entities/product.rs`
```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::postgres::PgRow;
use sqlx::Row;
use crate::domain::value_objects::ProductName;
use crate::errors::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: ProductName, // Using the Value Object
    pub created_at: DateTime<Utc>,
}

// 1. Mapping from Database (Infrastructure -> Domain)
impl sqlx::FromRow<'_, PgRow> for Product {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Product {
            id: row.try_get("id")?,
            // Use from_trusted because we trust database integrity
            name: ProductName::from_trusted(row.try_get("name")?), 
            created_at: row.try_get("created_at")?,
        })
    }
}

impl Product {
    // 2. Smart Constructor for new business logic
    pub fn new(name: ProductName) -> Result<Self, DomainError> {
        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: Utc::now(),
        })
    }

    // 3. State mutation methods
    pub fn update_name(&mut self, new_name: ProductName) {
        self.name = new_name;
    }
}
```

---

### Step 3: Data Transfer Objects and Syntactic Validation (Application)

DTOs receive the HTTP request payload. This is where we use the `validator` library.

**Location:** `src/application/dtos/product_dto.rs`
```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(
        min = 3, 
        max = 100, 
        message = "Name must be between 3 and 100 characters"
    ))]
    pub name: String,
}
```

---

### Step 4: Service Orchestration (Application)

The Service acts as a director. It transforms DTOs into Value Objects, verifies external state, and persists the Entity.

**Location:** `src/application/services/product_service.rs`
```rust
use std::sync::Arc;
use crate::application::dtos::CreateProductRequest;
use crate::domain::entities::Product;
use crate::domain::value_objects::ProductName;
use crate::errors::ApiError;
use crate::interfaces::ProductRepository;

pub struct ProductService {
    repository: Arc<dyn ProductRepository>,
}

impl ProductService {
    pub async fn create_product(
        &self, 
        request: CreateProductRequest
    ) -> Result<Product, ApiError> {
        // 1. I/O-dependent rules (e.g., Uniqueness)
        if self.repository.exists_by_name(&request.name).await? {
            return Err(ApiError::Conflict(
                "Product name already exists".into()
            ));
        }

        // 2. Value Object instantiation 
        // (Automatically converts DomainError to ApiError 400)
        let name_vo = ProductName::new(request.name)?;

        // 3. Entity creation
        let product = Product::new(name_vo)?;

        // 4. Persistence
        let created_product = self.repository.create(&product).await?;

        Ok(created_product)
    }
}
```

---

### Step 5: Database Persistence (Infrastructure)

The repository has no knowledge of business rules. It receives a valid entity and extracts its primitive types to interact with SQL.

**Location:** `src/infrastructure/persistence/postgres/product_repository.rs`
```rust
use async_trait::async_trait;
use crate::domain::entities::Product;
use crate::errors::ApiError;
use crate::interfaces::ProductRepository;

// Assuming a PostgresProductRepository struct is implemented...

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn create(&self, product: &Product) -> Result<Product, ApiError> {
        let query = r#"
            INSERT INTO products (id, name, created_at)
            VALUES ($1, $2, $3)
            RETURNING *
        "#;
        
        let created = sqlx::query_as::<_, Product>(query)
            .bind(&product.id)
            .bind(product.name.as_str()) // <- Extract primitive from Value Object
            .bind(product.created_at)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        Ok(created)
    }
}
```

---

## Data Flow Summary

1. **Client** → Sends JSON
2. **Controller** → Extracts JSON to DTO (`CreateProductRequest`). Executes syntactic validation (`.validate()`)
3. **Controller** → Passes DTO to `ProductService`
4. **ProductService** → Attempts to create `ProductName` (Value Object) from DTO's String
5. **ProductName::new** → Executes pure validations. Fails if domain rules are not met
6. **ProductService** → Verifies in DB if name already exists
7. **ProductService** → Instantiates `Product` (Entity) passing the `ProductName`
8. **ProductService** → Sends validated entity to `ProductRepository`
9. **ProductRepository** → Decomposes entity extracting primitives (`.as_str()`) and executes SQL statement