use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{postgres::PgRow, Postgres, Row};
use std::str::FromStr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{error::AppError, models::*, state::AppState};

const DEMO_PASSWORD: &str = "StrongPassword!234";
const PASSWORD_HASH_ALGORITHM: &str = "sha256-iter-v1";
const PASSWORD_HASH_ITERATIONS: u32 = 120_000;
const SESSION_COOKIE_NAME: &str = "pricetracker_session";
const SESSION_COOKIE_PATH: &str = "/";

fn normalized_key(value: &str) -> String {
    value.trim().to_lowercase().replace(' ', "-")
}

fn notice_title(kind: &str) -> String {
    kind.split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn file_extension(filename: &str) -> Option<String> {
    filename.rsplit_once('.').map(|(_, ext)| ext.to_lowercase())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn password_hash_parameters(iterations: u32) -> Value {
    json!({ "iterations": iterations })
}

fn password_hash_iterations(value: &Value) -> u32 {
    value.get("iterations")
        .and_then(Value::as_u64)
        .map(|iterations| iterations.min(u32::MAX as u64) as u32)
        .filter(|iterations| *iterations > 0)
        .unwrap_or(PASSWORD_HASH_ITERATIONS)
}

fn generate_password_salt() -> Vec<u8> {
    let mut salt = Vec::with_capacity(32);
    salt.extend_from_slice(Uuid::new_v4().as_bytes());
    salt.extend_from_slice(Uuid::new_v4().as_bytes());
    salt
}

fn derive_password_hash(password: &str, salt: &[u8], iterations: u32) -> String {
    let mut digest = {
        let mut hasher = Sha256::new();
        hasher.update(salt);
        hasher.update(password.as_bytes());
        hasher.finalize().to_vec()
    };

    for _ in 1..iterations {
        let mut hasher = Sha256::new();
        hasher.update(&digest);
        hasher.update(salt);
        hasher.update(password.as_bytes());
        digest = hasher.finalize().to_vec();
    }

    hex_encode(&digest)
}

fn normalize_account_status(status: Option<&str>) -> Result<String, AppError> {
    let normalized = status
        .unwrap_or("active")
        .trim()
        .to_ascii_lowercase();

    let allowed = ["active", "review", "suspended", "disabled", "deleted"];

    if allowed.iter().any(|allowed_status| *allowed_status == normalized) {
        Ok(normalized)
    } else {
        Err(AppError::BadRequest(format!(
            "Unsupported account status: {normalized}"
        )))
    }
}

fn normalize_role_codes(role_codes: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = role_codes
        .unwrap_or_else(|| vec!["user".to_string()])
        .into_iter()
        .map(|role| role.trim().to_ascii_lowercase())
        .filter(|role| !role.is_empty())
        .collect::<Vec<_>>();

    if normalized.is_empty() {
        normalized.push("user".to_string());
    }

    if normalized.iter().any(|role| role == "admin")
        && !normalized.iter().any(|role| role == "user")
    {
        normalized.push("user".to_string());
    }

    normalized.sort();
    normalized.dedup();
    normalized
}

async fn normalize_supported_currency_code(
    db: &sqlx::PgPool,
    currency_code: Option<&str>,
) -> Result<Option<String>, AppError> {
    let Some(raw_currency_code) = currency_code else {
        return Ok(None);
    };

    let normalized = raw_currency_code.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Ok(None);
    }

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM currency WHERE code = $1 AND is_active = TRUE)",
    )
    .bind(&normalized)
    .fetch_one(db)
    .await?;

    if exists {
        Ok(Some(normalized))
    } else {
        Err(AppError::BadRequest(format!(
            "Unsupported preferred currency: {normalized}"
        )))
    }
}

async fn ensure_user_source(db: &sqlx::PgPool, account_id: Uuid) -> Result<Uuid, AppError> {
    if let Some(existing) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM data_source WHERE account_id = $1 ORDER BY created_at ASC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(db)
    .await?
    {
        return Ok(existing);
    }

    let source_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO data_source (id, source_type, account_id, source_name, trust_score, is_verified, created_at)
        VALUES ($1, 'USER_SUBMISSION', $2, 'User submission', 50.00, FALSE, NOW())
        "#,
    )
    .bind(source_id)
    .bind(account_id)
    .execute(db)
    .await?;

    Ok(source_id)
}

async fn current_notice_id(
    db: &sqlx::PgPool,
    notice_kind: &str,
    version: Option<&str>,
) -> Result<Option<Uuid>, AppError> {
    let row = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM privacy_notice_version
        WHERE notice_kind = $1
          AND ($2::text IS NULL OR version_label = $2)
          AND retired_at IS NULL
        ORDER BY published_at DESC
        LIMIT 1
        "#,
    )
    .bind(notice_kind)
    .bind(version)
    .fetch_optional(db)
    .await?;

    Ok(row)
}

async fn fetch_applied_v2_migrations(db: &sqlx::PgPool) -> Result<Vec<String>, AppError> {
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT filename
        FROM _app_sql_migrations
        WHERE filename LIKE '%\_v2.sql' ESCAPE '\'
        ORDER BY filename
        "#,
    )
    .fetch_all(db)
    .await?;

    Ok(rows)
}

async fn ensure_account_is_accessible(
    db: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<(), AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            a.account_status,
            a.deleted_at,
            EXISTS(
                SELECT 1
                FROM account_suspension s
                WHERE s.account_id = a.id
                  AND s.starts_at <= NOW()
                  AND (s.ends_at IS NULL OR s.ends_at > NOW())
            ) AS is_suspended
        FROM account a
        WHERE a.id = $1
        "#,
    )
    .bind(account_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Account is no longer available".to_string()))?;

    let status = row
        .get::<String, _>("account_status")
        .trim()
        .to_ascii_lowercase();
    let is_deleted = row.get::<Option<chrono::DateTime<Utc>>, _>("deleted_at").is_some()
        || status == "deleted";
    let is_suspended = row.get::<bool, _>("is_suspended") || status == "suspended";

    if is_deleted {
        return Err(AppError::Unauthorized(
            "Account is no longer available".to_string(),
        ));
    }

    if is_suspended {
        return Err(AppError::Forbidden(
            "Account is currently suspended".to_string(),
        ));
    }

    if status == "disabled" {
        return Err(AppError::Forbidden(
            "Account is currently disabled".to_string(),
        ));
    }

    Ok(())
}

async fn store_password_credential(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    account_id: Uuid,
    password: &str,
) -> Result<(), AppError> {
    let existing = sqlx::query(
        r#"
        SELECT
            a.id,
            pc.password_hash,
            pc.salt_value,
            pc.hash_algorithm,
            pc.hash_parameters_json,
            pc.password_version,
            pc.changed_at
        FROM authenticator a
        LEFT JOIN password_credential pc ON pc.authenticator_id = a.id
        WHERE a.account_id = $1
          AND a.authenticator_type = 'PASSWORD'
          AND a.status = 'active'
          AND a.revoked_at IS NULL
        ORDER BY a.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await?;

    let authenticator_id = existing
        .as_ref()
        .map(|row| row.get::<Uuid, _>("id"))
        .unwrap_or_else(Uuid::new_v4);

    if existing.is_none() {
        sqlx::query(
            r#"
            INSERT INTO authenticator (
                id, account_id, authenticator_type, usage_type, display_label,
                status, enrolled_at, confirmed_at, last_used_at, revoked_at, created_at
            ) VALUES ($1, $2, 'PASSWORD', 'LOGIN', 'Password', 'active', NOW(), NOW(), NOW(), NULL, NOW())
            "#,
        )
        .bind(authenticator_id)
        .bind(account_id)
        .execute(&mut **tx)
        .await?;
    }

    let next_version = if let Some(row) = &existing {
        if let Some(previous_hash) = row.get::<Option<String>, _>("password_hash") {
            sqlx::query(
                r#"
                INSERT INTO password_history (
                    id, account_id, password_hash, salt_value, hash_algorithm,
                    hash_parameters_json, password_version, valid_from, valid_to, stored_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
                ON CONFLICT (account_id, password_version) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(account_id)
            .bind(previous_hash)
            .bind(row.get::<Vec<u8>, _>("salt_value"))
            .bind(row.get::<String, _>("hash_algorithm"))
            .bind(row.get::<Value, _>("hash_parameters_json"))
            .bind(row.get::<i32, _>("password_version"))
            .bind(row.get::<chrono::DateTime<Utc>, _>("changed_at"))
            .execute(&mut **tx)
            .await?;
        }

        row.get::<Option<i32>, _>("password_version").unwrap_or(0) + 1
    } else {
        1
    };

    let salt = generate_password_salt();
    let iterations = PASSWORD_HASH_ITERATIONS;
    let hash = derive_password_hash(password, &salt, iterations);

    sqlx::query(
        r#"
        INSERT INTO password_credential (
            authenticator_id, password_hash, salt_value, hash_algorithm, hash_parameters_json,
            password_version, changed_at, must_rotate, compromised_at
        ) VALUES ($1, $2, $3, $4, $5, $6, NOW(), FALSE, NULL)
        ON CONFLICT (authenticator_id) DO UPDATE
        SET password_hash = EXCLUDED.password_hash,
            salt_value = EXCLUDED.salt_value,
            hash_algorithm = EXCLUDED.hash_algorithm,
            hash_parameters_json = EXCLUDED.hash_parameters_json,
            password_version = EXCLUDED.password_version,
            changed_at = EXCLUDED.changed_at,
            must_rotate = FALSE,
            compromised_at = NULL
        "#,
    )
    .bind(authenticator_id)
    .bind(hash)
    .bind(salt)
    .bind(PASSWORD_HASH_ALGORITHM)
    .bind(password_hash_parameters(iterations))
    .bind(next_version)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn verify_account_password(
    db: &sqlx::PgPool,
    account_id: Uuid,
    password: &str,
) -> Result<bool, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            pc.password_hash,
            pc.salt_value,
            pc.hash_parameters_json
        FROM authenticator a
        JOIN password_credential pc ON pc.authenticator_id = a.id
        WHERE a.account_id = $1
          AND a.authenticator_type = 'PASSWORD'
          AND a.status = 'active'
          AND a.revoked_at IS NULL
        ORDER BY a.created_at ASC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .fetch_optional(db)
    .await?;

    if let Some(row) = row {
        let salt = row.get::<Vec<u8>, _>("salt_value");
        let iterations = password_hash_iterations(&row.get::<Value, _>("hash_parameters_json"));
        let expected = row.get::<String, _>("password_hash");
        return Ok(derive_password_hash(password, &salt, iterations) == expected);
    }

    Ok(password == DEMO_PASSWORD)
}

async fn provision_demo_password_if_missing(
    db: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<(), AppError> {
    let has_password = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM authenticator a
            JOIN password_credential pc ON pc.authenticator_id = a.id
            WHERE a.account_id = $1
              AND a.authenticator_type = 'PASSWORD'
              AND a.status = 'active'
              AND a.revoked_at IS NULL
        )
        "#,
    )
    .bind(account_id)
    .fetch_one(db)
    .await?;

    if has_password {
        return Ok(());
    }

    let mut tx = db.begin().await?;
    store_password_credential(&mut tx, account_id, DEMO_PASSWORD).await?;
    tx.commit().await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListItemsQuery {
    q: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompareQuery {
    variant_ids: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminRecordUpsertRequest {
    values: serde_json::Map<String, Value>,
}

fn admin_column(
    key: &str,
    label: &str,
    input: &str,
    required: bool,
    mutable: bool,
) -> AdminTableColumn {
    admin_column_with_meta(key, label, input, required, mutable, "", None)
}

fn admin_column_with_meta(
    key: &str,
    label: &str,
    input: &str,
    required: bool,
    mutable: bool,
    description: &str,
    lookup_key: Option<&str>,
) -> AdminTableColumn {
    AdminTableColumn {
        key: key.to_string(),
        label: label.to_string(),
        input: input.to_string(),
        required,
        mutable,
        description: if description.is_empty() {
            None
        } else {
            Some(description.to_string())
        },
        lookup_key: lookup_key.map(|value| value.to_string()),
    }
}

fn admin_table_definitions() -> Vec<AdminTableDefinition> {
    vec![
        AdminTableDefinition {
            id: "categories".to_string(),
            label: "Categories".to_string(),
            description: "Manage the product categories shown across the catalog.".to_string(),
            supports_create: true,
            supports_approval: false,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column_with_meta(
                    "name",
                    "Name",
                    "text",
                    true,
                    true,
                    "Required. Use the public category name shown to users.",
                    None,
                ),
                admin_column_with_meta(
                    "description",
                    "Description",
                    "textarea",
                    false,
                    true,
                    "Optional admin note for this category.",
                    None,
                ),
                admin_column_with_meta(
                    "parentId",
                    "Parent category",
                    "text",
                    false,
                    true,
                    "Optional. Search by category name to nest this category under another one.",
                    Some("categories"),
                ),
                admin_column("isActive", "Active", "boolean", true, true),
            ],
        },
        AdminTableDefinition {
            id: "brands".to_string(),
            label: "Brands".to_string(),
            description: "Edit brand names and website metadata used by products.".to_string(),
            supports_create: true,
            supports_approval: false,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column("name", "Name", "text", true, true),
                admin_column("websiteUrl", "Website URL", "text", false, true),
                admin_column("isActive", "Active", "boolean", true, true),
            ],
        },
        AdminTableDefinition {
            id: "units".to_string(),
            label: "Units".to_string(),
            description:
                "Manage measurement units used by variants and normalized pricing.".to_string(),
            supports_create: true,
            supports_approval: false,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column_with_meta(
                    "unitFamilyId",
                    "Unit family",
                    "text",
                    false,
                    true,
                    "Search by family name or code. Leave empty when creating a new base unit.",
                    Some("unit-families"),
                ),
                admin_column_with_meta(
                    "code",
                    "Code",
                    "text",
                    true,
                    true,
                    "Required short code such as kg or g.",
                    None,
                ),
                admin_column_with_meta(
                    "name",
                    "Name",
                    "text",
                    true,
                    true,
                    "Required public label for the unit.",
                    None,
                ),
                admin_column_with_meta(
                    "symbol",
                    "Symbol",
                    "text",
                    true,
                    true,
                    "Required display symbol used in variant summaries.",
                    None,
                ),
                admin_column_with_meta(
                    "factorToBase",
                    "Factor to base",
                    "text",
                    true,
                    true,
                    "Required conversion factor against the base unit in the same family.",
                    None,
                ),
                admin_column("isBaseUnit", "Base unit", "boolean", true, true),
            ],
        },
        AdminTableDefinition {
            id: "retailers".to_string(),
            label: "Retailers".to_string(),
            description: "Manage the retailer records that own shops.".to_string(),
            supports_create: true,
            supports_approval: false,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column("name", "Name", "text", true, true),
                admin_column("retailerType", "Retailer type", "text", true, true),
                admin_column("websiteUrl", "Website URL", "text", false, true),
                admin_column("isActive", "Active", "boolean", true, true),
            ],
        },
        AdminTableDefinition {
            id: "shops".to_string(),
            label: "Shops".to_string(),
            description: "Control store records used for price capture and code lookup.".to_string(),
            supports_create: true,
            supports_approval: false,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column_with_meta(
                    "retailerId",
                    "Retailer",
                    "text",
                    true,
                    true,
                    "Required. Search by retailer name to link this shop.",
                    Some("retailers"),
                ),
                admin_column_with_meta(
                    "name",
                    "Name",
                    "text",
                    true,
                    true,
                    "Required public shop name.",
                    None,
                ),
                admin_column_with_meta(
                    "timezoneName",
                    "Timezone",
                    "text",
                    true,
                    true,
                    "Required IANA timezone, for example Europe/London.",
                    None,
                ),
                admin_column("isOnline", "Online shop", "boolean", true, true),
                admin_column("isActive", "Active", "boolean", true, true),
            ],
        },
        AdminTableDefinition {
            id: "items".to_string(),
            label: "Items".to_string(),
            description: "Edit the core catalog items that variants belong to.".to_string(),
            supports_create: true,
            supports_approval: true,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column_with_meta(
                    "categoryId",
                    "Category",
                    "text",
                    true,
                    true,
                    "Required. Search by category name.",
                    Some("categories"),
                ),
                admin_column_with_meta(
                    "name",
                    "Name",
                    "text",
                    true,
                    true,
                    "Required public item name.",
                    None,
                ),
                admin_column_with_meta(
                    "specification",
                    "Specification",
                    "text",
                    false,
                    true,
                    "Optional extra detail shown with the item name.",
                    None,
                ),
                admin_column("status", "Status", "text", true, true),
            ],
        },
        AdminTableDefinition {
            id: "item-variants".to_string(),
            label: "Item Variants".to_string(),
            description: "Inspect the item variants currently stored in the catalog.".to_string(),
            supports_create: false,
            supports_approval: true,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column("itemId", "Item", "readonly", false, false),
                admin_column("brandId", "Brand", "readonly", false, false),
                admin_column("variantName", "Variant", "readonly", false, false),
                admin_column("packageQuantity", "Package quantity", "readonly", false, false),
                admin_column("packageUnitId", "Package unit", "readonly", false, false),
                admin_column("packCount", "Pack count", "readonly", false, false),
                admin_column("status", "Status", "readonly", false, false),
            ],
        },
        AdminTableDefinition {
            id: "discount-types".to_string(),
            label: "Discount Types".to_string(),
            description: "Maintain discount labels used when saving price submissions.".to_string(),
            supports_create: true,
            supports_approval: false,
            columns: vec![
                admin_column("id", "ID", "readonly", false, false),
                admin_column("code", "Code", "text", true, true),
                admin_column("name", "Name", "text", true, true),
                admin_column("description", "Description", "textarea", false, true),
                admin_column("isActive", "Active", "boolean", true, true),
            ],
        },
    ]
}

fn admin_table_definition(table_id: &str) -> Result<AdminTableDefinition, AppError> {
    admin_table_definitions()
        .into_iter()
        .find(|table| table.id == table_id)
        .ok_or_else(|| AppError::NotFound(format!("Unknown admin table: {table_id}")))
}

async fn load_admin_lookup_options(
    db: &sqlx::PgPool,
    lookup_key: &str,
) -> Result<Vec<AdminLookupOption>, AppError> {
    let options = match lookup_key {
        "categories" => sqlx::query(
            "SELECT id, name, parent_category_id FROM category WHERE is_active = TRUE ORDER BY name",
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|row| AdminLookupOption {
            id: row.get::<Uuid, _>("id"),
            label: row.get::<String, _>("name"),
            detail: row
                .get::<Option<Uuid>, _>("parent_category_id")
                .map(|value| format!("Parent {value}")),
        })
        .collect(),
        "retailers" => sqlx::query(
            "SELECT id, name, retailer_type FROM retailer WHERE is_active = TRUE ORDER BY name",
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|row| AdminLookupOption {
            id: row.get::<Uuid, _>("id"),
            label: row.get::<String, _>("name"),
            detail: Some(row.get::<String, _>("retailer_type")),
        })
        .collect(),
        "unit-families" => sqlx::query(
            "SELECT id, code, name FROM unit_family ORDER BY name",
        )
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|row| AdminLookupOption {
            id: row.get::<Uuid, _>("id"),
            label: row.get::<String, _>("name"),
            detail: Some(row.get::<String, _>("code")),
        })
        .collect(),
        _ => Vec::new(),
    };

    Ok(options)
}

async fn admin_table_lookups(
    db: &sqlx::PgPool,
    table: &AdminTableDefinition,
) -> Result<std::collections::HashMap<String, Vec<AdminLookupOption>>, AppError> {
    let mut lookups = std::collections::HashMap::new();

    for column in &table.columns {
        if let Some(lookup_key) = &column.lookup_key {
            lookups.insert(
                column.key.clone(),
                load_admin_lookup_options(db, lookup_key).await?,
            );
        }
    }

    Ok(lookups)
}

fn optional_string(
    values: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<String>, AppError> {
    match values.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Some(_) => Err(AppError::BadRequest(format!("{key} must be a string"))),
    }
}

fn required_string(
    values: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<String, AppError> {
    optional_string(values, key)?
        .ok_or_else(|| AppError::BadRequest(format!("{key} is required")))
}

fn optional_uuid(
    values: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<Uuid>, AppError> {
    optional_string(values, key)?
        .map(|value| {
            Uuid::parse_str(&value)
                .map_err(|_| AppError::BadRequest(format!("{key} must be a UUID")))
        })
        .transpose()
}

fn required_uuid(
    values: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Uuid, AppError> {
    optional_uuid(values, key)?
        .ok_or_else(|| AppError::BadRequest(format!("{key} is required")))
}

fn bool_with_default(
    values: &serde_json::Map<String, Value>,
    key: &str,
    default: bool,
) -> Result<bool, AppError> {
    match values.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(value)) => Ok(*value),
        Some(_) => Err(AppError::BadRequest(format!("{key} must be true or false"))),
    }
}

fn required_decimal(
    values: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Decimal, AppError> {
    match values.get(key) {
        Some(Value::Number(value)) => Decimal::from_str(&value.to_string())
            .map_err(|_| AppError::BadRequest(format!("{key} must be a decimal number"))),
        Some(Value::String(value)) => Decimal::from_str(value.trim())
            .map_err(|_| AppError::BadRequest(format!("{key} must be a decimal number"))),
        None | Some(Value::Null) => Err(AppError::BadRequest(format!("{key} is required"))),
        Some(_) => Err(AppError::BadRequest(format!("{key} must be a decimal number"))),
    }
}

async fn resolve_unit_family_id(
    db: &sqlx::PgPool,
    values: &serde_json::Map<String, Value>,
    code: &str,
    name: &str,
    is_base_unit: bool,
) -> Result<Uuid, AppError> {
    if let Some(unit_family_id) = optional_uuid(values, "unitFamilyId")? {
        return Ok(unit_family_id);
    }

    if !is_base_unit {
        return Err(AppError::BadRequest(
            "unitFamilyId is required unless the unit is marked as a base unit".to_string(),
        ));
    }

    if let Some(existing_id) = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM unit_family
        WHERE LOWER(code) = LOWER($1) OR LOWER(name) = LOWER($2)
        ORDER BY created_at
        LIMIT 1
        "#,
    )
    .bind(code)
    .bind(name)
    .fetch_optional(db)
    .await?
    {
        return Ok(existing_id);
    }

    let family_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO unit_family (id, code, name, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind(family_id)
    .bind(code.trim().to_ascii_uppercase())
    .bind(name)
    .execute(db)
    .await?;

    Ok(family_id)
}

async fn fetch_account_role_codes(
    db: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<Vec<String>, AppError> {
    Ok(sqlx::query_scalar::<_, String>(
        r#"
        SELECT r.code
        FROM account_role ar
        JOIN "role" r ON r.id = ar.role_id
        WHERE ar.account_id = $1
        ORDER BY r.code
        "#,
    )
    .bind(account_id)
    .fetch_all(db)
    .await?)
}

async fn fetch_account_scope_codes(
    db: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<Vec<String>, AppError> {
    Ok(sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT p.code
        FROM account_role ar
        JOIN role_permission rp ON rp.role_id = ar.role_id
        JOIN permission p ON p.id = rp.permission_id
        WHERE ar.account_id = $1
        ORDER BY p.code
        "#,
    )
    .bind(account_id)
    .fetch_all(db)
    .await?)
}

fn fallback_scopes(roles: &[String]) -> Vec<String> {
    let mut scopes = vec![
        "catalog:read".to_string(),
        "price:read_public".to_string(),
        "price:write_own".to_string(),
        "purchase:write_own".to_string(),
    ];

    if roles.iter().any(|role| role == "admin") {
        scopes.extend([
            "admin:dashboard".to_string(),
            "catalog:write".to_string(),
            "moderation:write".to_string(),
        ]);
    }

    scopes.sort();
    scopes.dedup();
    scopes
}

fn build_health_component(status: &str, connected: bool, detail: String) -> HealthComponent {
    HealthComponent {
        status: status.to_string(),
        connected,
        detail,
        checked_at: Utc::now(),
    }
}

async fn require_admin_account_id(
    db: &sqlx::PgPool,
    headers: &HeaderMap,
) -> Result<Uuid, AppError> {
    let account_id = current_account_id(db, headers).await?;
    let roles = fetch_account_role_codes(db, account_id).await?;

    if roles.iter().any(|role| role == "admin") {
        Ok(account_id)
    } else {
        Err(AppError::Forbidden(
            "Admin access is required for this route".to_string(),
        ))
    }
}

async fn next_public_handle(db: &sqlx::PgPool, display_name: &str) -> Result<String, AppError> {
    let base = {
        let normalized = normalized_key(display_name);
        if normalized.is_empty() {
            "user".to_string()
        } else {
            normalized
        }
    };

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM account WHERE public_handle = $1)",
    )
    .bind(&base)
    .fetch_one(db)
    .await?;

    if !exists {
        return Ok(base);
    }

    let suffix = Uuid::new_v4().simple().to_string();
    Ok(format!("{base}-{}", &suffix[..8]))
}

async fn create_account_with_roles(
    db: &sqlx::PgPool,
    email: &str,
    password: &str,
    display_name: &str,
    primary_phone: Option<&str>,
    accepted_legal_documents: &[LegalDocumentAcceptance],
    role_codes: &[String],
    granted_by_account_id: Option<Uuid>,
    initial_status: &str,
) -> Result<Uuid, AppError> {
    if password.len() < 12 {
        return Err(AppError::BadRequest(
            "Password must be at least 12 characters".to_string(),
        ));
    }

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account_email WHERE normalized_email = $1 AND deleted_at IS NULL",
    )
    .bind(email.to_lowercase())
    .fetch_one(db)
    .await?;

    if exists > 0 {
        return Err(AppError::Conflict(
            "Email is already registered".to_string(),
        ));
    }

    let account_id = Uuid::new_v4();
    let public_handle = next_public_handle(db, display_name).await?;
    let is_admin_registration = role_codes.iter().any(|role| role == "admin");
    let trust_score = if is_admin_registration {
        Decimal::new(10000, 2)
    } else {
        Decimal::new(5000, 2)
    };
    let mut tx = db.begin().await?;

    sqlx::query(
        "INSERT INTO account (id, public_handle, account_status, created_at, updated_at, deleted_at, last_active_at) VALUES ($1, $2, $3, NOW(), NOW(), NULL, NOW())",
    )
    .bind(account_id)
    .bind(public_handle)
    .bind(initial_status)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO account_profile (account_id, display_name, locale, timezone_name, preferred_currency_code, profile_bio, created_at, updated_at) VALUES ($1, $2, 'en-GB', 'Europe/London', 'GBP', NULL, NOW(), NOW())",
    )
    .bind(account_id)
    .bind(display_name)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO account_email (
            id, account_id, email, normalized_email, email_role, is_login_enabled,
            is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at
        ) VALUES ($1, $2, $3, $4, 'PRIMARY', TRUE, TRUE, NOW(), 'app', NOW(), NOW(), NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(account_id)
    .bind(email)
    .bind(email.to_lowercase())
    .execute(&mut *tx)
    .await?;

    if let Some(phone) = primary_phone.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    }) {
        sqlx::query(
            r#"
            INSERT INTO account_phone (
                id, account_id, e164_phone_number, extension, phone_role, is_sms_enabled, is_voice_enabled,
                is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at
            ) VALUES ($1, $2, $3, NULL, 'PRIMARY', TRUE, TRUE, TRUE, NOW(), 'app', NOW(), NOW(), NULL)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(account_id)
        .bind(phone)
        .execute(&mut *tx)
        .await?;
    }

    let source_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO data_source (id, source_type, account_id, source_name, trust_score, is_verified, created_at)
        VALUES ($1, 'USER_SUBMISSION', $2, $3, $4, $5, NOW())
        "#,
    )
    .bind(source_id)
    .bind(account_id)
    .bind(display_name)
    .bind(trust_score)
    .bind(is_admin_registration)
    .execute(&mut *tx)
    .await?;

    for role_code in role_codes {
        sqlx::query(
            r#"
            INSERT INTO account_role (account_id, role_id, granted_by_account_id, granted_at)
            SELECT $1, r.id, $3, NOW()
            FROM "role" r
            WHERE r.code = $2
            ON CONFLICT (account_id, role_id) DO NOTHING
            "#,
        )
        .bind(account_id)
        .bind(role_code)
        .bind(granted_by_account_id)
        .execute(&mut *tx)
        .await?;
    }

    for doc in accepted_legal_documents {
        let notice_id = current_notice_id(db, &doc.document_key, Some(&doc.version)).await?;
        let purpose_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM processing_purpose WHERE code = $1 LIMIT 1",
        )
        .bind(&doc.document_key)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest(format!(
                "Unsupported legal document key: {}",
                doc.document_key
            ))
        })?;

        sqlx::query(
            r#"
            INSERT INTO consent_record (
                id, account_id, anonymous_subject_token_hash, processing_purpose_id, notice_version_id,
                consent_status, captured_via, evidence_json, captured_at, withdrawn_at
            ) VALUES ($1, $2, NULL, $3, $4, 'accepted', 'registration', '{"source":"register"}'::jsonb, NOW(), NULL)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(account_id)
        .bind(purpose_id)
        .bind(notice_id)
        .execute(&mut *tx)
        .await?;
    }

    store_password_credential(&mut tx, account_id, password).await?;

    tx.commit().await?;

    Ok(account_id)
}

async fn register_account_with_roles(
    db: &sqlx::PgPool,
    payload: &RegisterRequest,
    role_codes: &[String],
) -> Result<AuthSession, AppError> {
    let account_id = create_account_with_roles(
        db,
        &payload.email,
        &payload.password,
        &payload.display_name,
        payload.primary_phone.as_deref(),
        &payload.accepted_legal_documents,
        role_codes,
        None,
        "active",
    )
    .await?;

    let user = build_user_profile(db, account_id).await?;
    Ok(AuthSession {
        access_token: session_token_for_account(account_id),
        token_type: "Bearer".to_string(),
        expires_in_seconds: 900,
        user,
    })
}

fn validate_setting_value(value_type: &str, value: &Value) -> Result<(), AppError> {
    if value.is_null() {
        return Ok(());
    }

    let matches_type = match value_type.to_ascii_lowercase().as_str() {
        "boolean" => value.is_boolean(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "number" => value.is_number(),
        "string" => value.is_string(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => true,
    };

    if matches_type {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Setting value must match the declared type: {value_type}"
        )))
    }
}

fn admin_system_setting_from_row(row: PgRow) -> AdminSystemSetting {
    let default_value = row.get::<Option<Value>, _>("default_value_json");
    let current_value = row.get::<Option<Value>, _>("setting_value_json");

    AdminSystemSetting {
        id: row.get("id"),
        key: row.get("setting_key"),
        scope: row.get("scope_type"),
        value_type: row.get("value_type"),
        description: row.get("description"),
        is_sensitive: row.get("is_sensitive"),
        default_value: default_value.clone(),
        value: current_value.or(default_value).unwrap_or(Value::Null),
        updated_at: row.get("updated_at"),
        updated_by_account_id: row.get("updated_by_account_id"),
    }
}

async fn write_audit_log(
    db: &sqlx::PgPool,
    actor_account_id: Uuid,
    action_code: &str,
    entity_type: &str,
    entity_id: Option<Uuid>,
    old_value_json: Option<Value>,
    new_value_json: Option<Value>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO audit_log (
            id, actor_account_id, action_code, entity_type, entity_id, request_id,
            old_value_json, new_value_json, created_at
        ) VALUES ($1, $2, $3, $4, $5, NULL, $6, $7, NOW())
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(actor_account_id)
    .bind(action_code)
    .bind(entity_type)
    .bind(entity_id)
    .bind(old_value_json)
    .bind(new_value_json)
    .execute(db)
    .await?;

    Ok(())
}

fn session_token_for_account(account_id: Uuid) -> String {
    format!("demo-token-{account_id}")
}

fn parse_demo_session_token(token: &str) -> Result<Uuid, AppError> {
    let account_id = token
        .trim()
        .strip_prefix("demo-token-")
        .ok_or_else(|| AppError::Unauthorized("Invalid session token".to_string()))?;

    Uuid::parse_str(account_id)
        .map_err(|_| AppError::Unauthorized("Invalid session token".to_string()))
}

fn session_cookie_value(account_id: Uuid, max_age_seconds: i32) -> String {
    format!(
        "{SESSION_COOKIE_NAME}={}; HttpOnly; Max-Age={max_age_seconds}; Path={SESSION_COOKIE_PATH}; SameSite=Lax",
        session_token_for_account(account_id)
    )
}

fn expired_session_cookie_value() -> String {
    format!(
        "{SESSION_COOKIE_NAME}=; HttpOnly; Max-Age=0; Path={SESSION_COOKIE_PATH}; SameSite=Lax; Expires=Thu, 01 Jan 1970 00:00:00 GMT"
    )
}

fn header_map_with_cookie(cookie_value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        HeaderValue::from_str(cookie_value).expect("session cookie header must be valid"),
    );
    headers
}

fn header_account_id(headers: &HeaderMap) -> Result<Option<Uuid>, AppError> {
    match headers.get("x-account-id") {
        None => Ok(None),
        Some(value) => {
            let text = value
                .to_str()
                .map_err(|_| AppError::BadRequest("Invalid x-account-id header".to_string()))?;
            let account_id = Uuid::parse_str(text)
                .map_err(|_| AppError::BadRequest("x-account-id must be a UUID".to_string()))?;
            Ok(Some(account_id))
        }
    }
}

fn bearer_account_id(headers: &HeaderMap) -> Result<Option<Uuid>, AppError> {
    match headers.get(header::AUTHORIZATION) {
        None => Ok(None),
        Some(value) => {
            let text = value.to_str().map_err(|_| {
                AppError::BadRequest("Invalid Authorization header".to_string())
            })?;
            let token = text
                .trim()
                .strip_prefix("Bearer ")
                .ok_or_else(|| {
                    AppError::Unauthorized(
                        "Authorization header must use a Bearer token".to_string(),
                    )
                })?;
            Ok(Some(parse_demo_session_token(token)?))
        }
    }
}

fn cookie_account_id(headers: &HeaderMap) -> Result<Option<Uuid>, AppError> {
    let cookie_header = match headers.get(header::COOKIE) {
        None => return Ok(None),
        Some(value) => value
            .to_str()
            .map_err(|_| AppError::BadRequest("Invalid Cookie header".to_string()))?,
    };

    let session_token = cookie_header
        .split(';')
        .find_map(|segment| {
            let mut parts = segment.trim().splitn(2, '=');
            let key = parts.next()?.trim();
            let value = parts.next()?.trim();

            if key == SESSION_COOKIE_NAME && !value.is_empty() {
                Some(value.to_string())
            } else {
                None
            }
        });

    match session_token {
        None => Ok(None),
        Some(token) => Ok(Some(parse_demo_session_token(&token)?)),
    }
}

pub fn build_router(state: AppState, cors_origin: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_str(cors_origin).expect("invalid cors origin"))
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::IF_MATCH,
            HeaderName::from_static("x-account-id"),
        ]);

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/health", get(health))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/register-admin", post(register_admin_bootstrap))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh_session))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/me", get(get_me).patch(update_me))
        .route("/api/v1/me/avatar", post(update_me_avatar).delete(delete_me_avatar))
        .route("/api/v1/categories", get(list_categories))
        .route("/api/v1/brands", get(list_brands))
        .route("/api/v1/units", get(list_units))
        .route("/api/v1/discount-types", get(list_discount_types))
        .route("/api/v1/items", get(list_items))
        .route("/api/v1/items/:item_id", get(get_item))
        .route("/api/v1/items/:item_id/variants", get(list_item_variants))
        .route("/api/v1/item-variants/:variant_id", get(get_item_variant))
        .route(
            "/api/v1/item-variants/:variant_id/prices",
            get(list_variant_prices),
        )
        .route(
            "/api/v1/item-variants/:variant_id/price-history",
            get(get_variant_price_history),
        )
        .route("/api/v1/compare", get(compare_variants_query))
        .route("/api/v1/comparisons", post(compare_variants_body))
        .route("/api/v1/shops", get(list_shops))
        .route("/api/v1/shops/:shop_id", get(get_shop))
        .route(
            "/api/v1/shops/:shop_id/product-codes/:code",
            get(lookup_product_code),
        )
        .route("/api/v1/files/uploads", post(create_file_upload_intent))
        .route(
            "/api/v1/files/uploads/:file_id/complete",
            post(complete_file_upload),
        )
        .route("/api/v1/me/files/:file_id", get(get_own_file))
        .route(
            "/api/v1/me/files/:file_id/download",
            get(get_own_file_download),
        )
        .route("/api/v1/purchases", post(create_purchase))
        .route("/api/v1/me/purchases", get(list_own_purchases))
        .route(
            "/api/v1/me/purchases/:purchase_id",
            get(get_own_purchase)
                .patch(update_own_purchase)
                .delete(delete_own_purchase),
        )
        .route("/api/v1/prices", post(create_price_submission))
        .route("/api/v1/me/prices", get(list_own_price_submissions))
        .route(
            "/api/v1/me/prices/:price_id",
            get(get_own_price_submission)
                .patch(update_own_price_submission)
                .delete(delete_own_price_submission),
        )
        .route("/api/v1/me/watchlist", get(list_watchlist))
        .route("/api/v1/me/watchlist/items", post(create_watchlist_item))
        .route(
            "/api/v1/me/watchlist/items/:watch_id",
            delete(delete_watchlist_item),
        )
        .route("/api/v1/me/alerts", get(list_alerts).post(create_alert))
        .route(
            "/api/v1/me/alerts/:alert_id",
            patch(update_alert).delete(delete_alert),
        )
        .route(
            "/api/v1/admin/moderation/prices",
            get(list_moderation_prices),
        )
        .route("/api/v1/admin/overview", get(get_admin_overview))
        .route("/api/v1/admin/users", get(list_admin_users).post(create_admin_user))
        .route("/api/v1/admin/users/:account_id", patch(update_admin_user))
        .route(
            "/api/v1/admin/users/bulk-actions",
            post(bulk_update_admin_users),
        )
        .route("/api/v1/admin/settings", get(list_admin_settings))
        .route(
            "/api/v1/admin/settings/:setting_key",
            patch(update_admin_setting),
        )
        .route("/api/v1/admin/database/tables", get(list_admin_tables))
        .route(
            "/api/v1/admin/database/tables/:table_id",
            get(get_admin_table_rows).post(create_admin_table_row),
        )
        .route(
            "/api/v1/admin/database/tables/:table_id/:record_id",
            patch(update_admin_table_row).delete(delete_admin_table_row),
        )
        .route(
            "/api/v1/admin/database/tables/:table_id/:record_id/approve",
            post(approve_admin_table_row),
        )
        .route(
            "/api/v1/admin/moderation/prices/:price_id/verify",
            post(verify_moderation_price),
        )
        .route(
            "/api/v1/admin/moderation/prices/:price_id/reject",
            post(reject_moderation_price),
        )
        .route("/api/v1/me/security", get(get_security_overview))
        .route(
            "/api/v1/me/emails",
            get(list_own_emails).post(create_own_email),
        )
        .route("/api/v1/me/emails/:email_id", delete(delete_own_email))
        .route(
            "/api/v1/me/emails/:email_id/verify",
            post(verify_own_email),
        )
        .route(
            "/api/v1/me/emails/:email_id/make-primary",
            post(make_own_email_primary),
        )
        .route(
            "/api/v1/me/phones",
            get(list_own_phones).post(create_own_phone),
        )
        .route("/api/v1/me/phones/:phone_id", delete(delete_own_phone))
        .route(
            "/api/v1/me/phones/:phone_id/verify",
            post(verify_own_phone),
        )
        .route(
            "/api/v1/me/phones/:phone_id/make-primary",
            post(make_own_phone_primary),
        )
        .route("/api/v1/legal/documents", get(list_current_legal_documents))
        .route(
            "/api/v1/me/privacy-consents",
            get(list_own_privacy_consents).post(accept_current_privacy_documents),
        )
        .route(
            "/api/v1/privacy/cookie-preferences",
            get(get_cookie_preferences).post(update_cookie_preferences),
        )
        .route("/api/v1/auth/password/change", post(change_password))
        .route("/api/v1/auth/password/forgot", post(not_implemented_ack))
        .route("/api/v1/auth/password/reset", post(not_implemented_ack))
        .route("/api/v1/auth/mfa/verify", post(mfa_verify_stub))
        .route(
            "/api/v1/auth/passkeys/authentication/options",
            post(passkey_options_stub),
        )
        .route(
            "/api/v1/auth/passkeys/authentication/verify",
            post(not_implemented_ack),
        )
        .route("/api/v1/me/passkeys", get(passkeys_list_stub))
        .route(
            "/api/v1/me/passkeys/registration/options",
            post(passkey_options_stub),
        )
        .route(
            "/api/v1/me/passkeys/registration/verify",
            post(not_implemented_ack),
        )
        .route(
            "/api/v1/me/passkeys/:passkey_id",
            delete(not_implemented_status),
        )
        .route("/api/v1/me/mfa/totp/setup", post(not_implemented_ack))
        .route("/api/v1/me/mfa/totp/enable", post(not_implemented_ack))
        .route("/api/v1/me/mfa/totp/disable", post(not_implemented_ack))
        .route(
            "/api/v1/me/mfa/recovery-codes/rotate",
            post(not_implemented_ack),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

async fn health(State(state): State<AppState>) -> Json<Envelope<HealthStatus>> {
    let (database, applied_migrations) = match fetch_applied_v2_migrations(&state.db).await {
        Ok(applied_migrations) => (
            build_health_component(
                "ok",
                true,
                "Database responded and migrations were read successfully.".to_string(),
            ),
            applied_migrations,
        ),
        Err(error) => (
            build_health_component(
                "offline",
                false,
                format!("Database check failed: {error:?}"),
            ),
            Vec::new(),
        ),
    };
    let overall_status = if database.connected { "ok" } else { "degraded" };

    Json(envelope(HealthStatus {
        status: overall_status.to_string(),
        service: "pricetracker-backend".to_string(),
        utc_time: Utc::now(),
        api: build_health_component(
            "ok",
            true,
            "Backend process is reachable and serving requests.".to_string(),
        ),
        database,
        applied_migrations,
    }))
}

async fn current_account_id(db: &sqlx::PgPool, headers: &HeaderMap) -> Result<Uuid, AppError> {
    let header_account_id = header_account_id(headers)?;
    let bearer_account_id = bearer_account_id(headers)?;
    let cookie_account_id = cookie_account_id(headers)?;

    if let (Some(bearer_id), Some(cookie_id)) = (bearer_account_id, cookie_account_id) {
        if bearer_id != cookie_id {
            return Err(AppError::Unauthorized(
                "Session cookie does not match the active access token".to_string(),
            ));
        }
    }

    let authenticated_account_id = bearer_account_id.or(cookie_account_id).ok_or_else(|| {
        AppError::Unauthorized("Authentication is required for this route".to_string())
    })?;

    if let Some(header_value) = header_account_id {
        if header_value != authenticated_account_id {
            return Err(AppError::Unauthorized(
                "x-account-id does not match the authenticated session".to_string(),
            ));
        }
    }

    ensure_account_is_accessible(db, authenticated_account_id).await?;

    Ok(authenticated_account_id)
}

async fn build_user_profile(db: &sqlx::PgPool, account_id: Uuid) -> Result<UserProfile, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            a.id,
            a.account_status,
            a.created_at,
            COALESCE(p.display_name, 'User') AS display_name,
            COALESCE(p.preferred_currency_code, 'GBP') AS default_currency,
            COALESCE(p.locale, 'en-GB') AS locale,
            COALESCE(p.timezone_name, 'Europe/London') AS timezone_name,
            p.profile_bio,
            (
                SELECT email
                FROM account_email
                WHERE account_id = a.id AND deleted_at IS NULL
                ORDER BY is_primary_for_account DESC, created_at ASC
                LIMIT 1
            ) AS primary_email,
            (
                SELECT e164_phone_number
                FROM account_phone
                WHERE account_id = a.id AND deleted_at IS NULL
                ORDER BY is_primary_for_account DESC, created_at ASC
                LIMIT 1
            ) AS primary_phone,
            (SELECT COUNT(*) FROM account_email WHERE account_id = a.id AND deleted_at IS NULL) AS email_count,
            (SELECT COUNT(*) FROM account_phone WHERE account_id = a.id AND deleted_at IS NULL) AS phone_count,
            (
                SELECT fa.id
                FROM file_attachment f
                JOIN file_asset fa ON fa.id = f.file_asset_id
                WHERE f.entity_type = 'account'
                  AND f.entity_id = a.id
                  AND f.attachment_role = 'avatar'
                  AND f.removed_at IS NULL
                  AND fa.is_deleted = FALSE
                ORDER BY f.is_primary DESC, f.created_at DESC
                LIMIT 1
            ) AS avatar_file_id,
            (
                SELECT fa.original_filename
                FROM file_attachment f
                JOIN file_asset fa ON fa.id = f.file_asset_id
                WHERE f.entity_type = 'account'
                  AND f.entity_id = a.id
                  AND f.attachment_role = 'avatar'
                  AND f.removed_at IS NULL
                  AND fa.is_deleted = FALSE
                ORDER BY f.is_primary DESC, f.created_at DESC
                LIMIT 1
            ) AS avatar_filename,
            EXISTS(
                SELECT 1
                FROM authenticator au
                JOIN password_credential pc ON pc.authenticator_id = au.id
                WHERE au.account_id = a.id
                  AND au.authenticator_type = 'PASSWORD'
                  AND au.status = 'active'
                  AND au.revoked_at IS NULL
            ) AS password_set,
            (
                SELECT COUNT(*)
                FROM authenticator au
                JOIN passkey_credential pk ON pk.authenticator_id = au.id
                WHERE au.account_id = a.id
                  AND au.authenticator_type = 'PASSKEY'
                  AND au.status = 'active'
                  AND au.revoked_at IS NULL
            ) AS passkey_count,
            EXISTS(
                SELECT 1
                FROM authenticator au
                JOIN totp_factor tf ON tf.authenticator_id = au.id
                WHERE au.account_id = a.id
                  AND au.authenticator_type = 'TOTP'
                  AND au.status = 'active'
                  AND au.revoked_at IS NULL
                  AND tf.confirmed_at IS NOT NULL
            ) AS mfa_enabled
        FROM account a
        LEFT JOIN account_profile p ON p.account_id = a.id
        WHERE a.id = $1
        "#,
    )
    .bind(account_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Account not found".to_string()))?;
    let mut roles = fetch_account_role_codes(db, account_id).await?;
    if roles.is_empty() {
        roles.push("user".to_string());
    }
    let mut scopes = fetch_account_scope_codes(db, account_id).await?;
    if scopes.is_empty() {
        scopes = fallback_scopes(&roles);
    }

    Ok(UserProfile {
        id: row.get("id"),
        status: row.get("account_status"),
        primary_email: row.get("primary_email"),
        primary_phone: row.get("primary_phone"),
        display_name: row.get("display_name"),
        roles,
        scopes,
        default_currency: row.get("default_currency"),
        locale: row.get("locale"),
        timezone_name: row.get("timezone_name"),
        profile_bio: row.get("profile_bio"),
        avatar_file_id: row.get("avatar_file_id"),
        avatar_filename: row.get("avatar_filename"),
        email_count: row.get("email_count"),
        phone_count: row.get("phone_count"),
        security: UserSecuritySummary {
            password_set: row.get("password_set"),
            mfa_enabled: row.get("mfa_enabled"),
            passkey_count: row.get("passkey_count"),
        },
        created_at: row.get("created_at"),
    })
}

async fn build_admin_user_summary(
    db: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<AdminUserSummary, AppError> {
    let profile = build_user_profile(db, account_id).await?;
    let row = sqlx::query(
        r#"
        SELECT
            deleted_at,
            last_active_at,
            (
                SELECT ends_at
                FROM account_suspension
                WHERE account_id = $1
                  AND starts_at <= NOW()
                  AND (ends_at IS NULL OR ends_at > NOW())
                ORDER BY created_at DESC
                LIMIT 1
            ) AS suspended_until
        FROM account
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(db)
    .await?;

    Ok(AdminUserSummary {
        id: profile.id,
        status: profile.status,
        display_name: profile.display_name,
        primary_email: profile.primary_email,
        primary_phone: profile.primary_phone,
        roles: profile.roles,
        scopes: profile.scopes,
        locale: profile.locale,
        timezone_name: profile.timezone_name,
        default_currency: profile.default_currency,
        profile_bio: profile.profile_bio,
        email_count: profile.email_count,
        phone_count: profile.phone_count,
        avatar_file_id: profile.avatar_file_id,
        avatar_filename: profile.avatar_filename,
        security: profile.security,
        created_at: profile.created_at,
        last_active_at: row.get("last_active_at"),
        deleted_at: row.get("deleted_at"),
        suspended_until: row.get("suspended_until"),
    })
}

async fn sync_account_roles(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    account_id: Uuid,
    role_codes: &[String],
    granted_by_account_id: Option<Uuid>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM account_role WHERE account_id = $1")
        .bind(account_id)
        .execute(&mut **tx)
        .await?;

    for role_code in role_codes {
        sqlx::query(
            r#"
            INSERT INTO account_role (account_id, role_id, granted_by_account_id, granted_at)
            SELECT $1, r.id, $3, NOW()
            FROM "role" r
            WHERE r.code = $2
            ON CONFLICT (account_id, role_id) DO NOTHING
            "#,
        )
        .bind(account_id)
        .bind(role_code)
        .bind(granted_by_account_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn upsert_primary_email(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    account_id: Uuid,
    email: &str,
) -> Result<(), AppError> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "Primary email cannot be empty".to_string(),
        ));
    }

    let normalized = trimmed.to_lowercase();
    let other_owner = sqlx::query_scalar::<_, Uuid>(
        "SELECT account_id FROM account_email WHERE normalized_email = $1 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(&normalized)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(other_owner) = other_owner {
        if other_owner != account_id {
            return Err(AppError::Conflict(
                "Email is already registered".to_string(),
            ));
        }
    }

    sqlx::query(
        "UPDATE account_email SET is_primary_for_account = FALSE, updated_at = NOW() WHERE account_id = $1 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .execute(&mut **tx)
    .await?;

    let existing_same_email = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM account_email WHERE account_id = $1 AND normalized_email = $2 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(account_id)
    .bind(&normalized)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(email_id) = existing_same_email {
        sqlx::query(
            r#"
            UPDATE account_email
            SET email = $2,
                normalized_email = $3,
                email_role = 'PRIMARY',
                is_login_enabled = TRUE,
                is_primary_for_account = TRUE,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(email_id)
        .bind(trimmed)
        .bind(&normalized)
        .execute(&mut **tx)
        .await?;
        return Ok(());
    }

    let current_primary = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM account_email WHERE account_id = $1 AND deleted_at IS NULL ORDER BY is_primary_for_account DESC, created_at ASC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(email_id) = current_primary {
        sqlx::query(
            r#"
            UPDATE account_email
            SET email = $2,
                normalized_email = $3,
                email_role = 'PRIMARY',
                is_login_enabled = TRUE,
                is_primary_for_account = TRUE,
                updated_at = NOW(),
                deleted_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(email_id)
        .bind(trimmed)
        .bind(&normalized)
        .execute(&mut **tx)
        .await?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO account_email (
                id, account_id, email, normalized_email, email_role, is_login_enabled,
                is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at
            ) VALUES ($1, $2, $3, $4, 'PRIMARY', TRUE, TRUE, NOW(), 'admin', NOW(), NOW(), NULL)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(account_id)
        .bind(trimmed)
        .bind(&normalized)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn upsert_primary_phone(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    account_id: Uuid,
    primary_phone: &str,
) -> Result<(), AppError> {
    let trimmed = primary_phone.trim();

    if trimmed.is_empty() {
        sqlx::query(
            "UPDATE account_phone SET deleted_at = NOW(), updated_at = NOW() WHERE account_id = $1 AND is_primary_for_account = TRUE AND deleted_at IS NULL",
        )
        .bind(account_id)
        .execute(&mut **tx)
        .await?;
        return Ok(());
    }

    let existing_same_phone = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM account_phone WHERE account_id = $1 AND e164_phone_number = $2 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(account_id)
    .bind(trimmed)
    .fetch_optional(&mut **tx)
    .await?;

    sqlx::query(
        "UPDATE account_phone SET is_primary_for_account = FALSE, updated_at = NOW() WHERE account_id = $1 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .execute(&mut **tx)
    .await?;

    if let Some(phone_id) = existing_same_phone {
        sqlx::query(
            r#"
            UPDATE account_phone
            SET e164_phone_number = $2,
                phone_role = 'PRIMARY',
                is_sms_enabled = TRUE,
                is_voice_enabled = TRUE,
                is_primary_for_account = TRUE,
                updated_at = NOW(),
                deleted_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(phone_id)
        .bind(trimmed)
        .execute(&mut **tx)
        .await?;
        return Ok(());
    }

    let current_primary = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM account_phone WHERE account_id = $1 AND deleted_at IS NULL ORDER BY is_primary_for_account DESC, created_at ASC LIMIT 1",
    )
    .bind(account_id)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(phone_id) = current_primary {
        sqlx::query(
            r#"
            UPDATE account_phone
            SET e164_phone_number = $2,
                phone_role = 'PRIMARY',
                is_sms_enabled = TRUE,
                is_voice_enabled = TRUE,
                is_primary_for_account = TRUE,
                updated_at = NOW(),
                deleted_at = NULL
            WHERE id = $1
            "#,
        )
        .bind(phone_id)
        .bind(trimmed)
        .execute(&mut **tx)
        .await?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO account_phone (
                id, account_id, e164_phone_number, extension, phone_role, is_sms_enabled,
                is_voice_enabled, is_primary_for_account, verified_at, verification_method,
                created_at, updated_at, deleted_at
            ) VALUES ($1, $2, $3, NULL, 'PRIMARY', TRUE, TRUE, TRUE, NOW(), 'admin', NOW(), NOW(), NULL)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(account_id)
        .bind(trimmed)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn set_account_status(
    tx: &mut sqlx::Transaction<'_, Postgres>,
    actor_account_id: Uuid,
    account_id: Uuid,
    status: &str,
    reason: &str,
) -> Result<(), AppError> {
    match status {
        "suspended" => {
            sqlx::query(
                "UPDATE account_suspension SET ends_at = NOW() WHERE account_id = $1 AND starts_at <= NOW() AND ends_at IS NULL",
            )
            .bind(account_id)
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                r#"
                INSERT INTO account_suspension (
                    id, account_id, reason, starts_at, ends_at, created_by_account_id, created_at
                ) VALUES ($1, $2, $3, NOW(), NULL, $4, NOW())
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(account_id)
            .bind(reason)
            .bind(actor_account_id)
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                "UPDATE account SET account_status = 'suspended', deleted_at = NULL, updated_at = NOW() WHERE id = $1",
            )
            .bind(account_id)
            .execute(&mut **tx)
            .await?;
        }
        "deleted" => {
            sqlx::query(
                "UPDATE account_suspension SET ends_at = NOW() WHERE account_id = $1 AND starts_at <= NOW() AND ends_at IS NULL",
            )
            .bind(account_id)
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                "UPDATE account SET account_status = 'deleted', deleted_at = NOW(), updated_at = NOW() WHERE id = $1",
            )
            .bind(account_id)
            .execute(&mut **tx)
            .await?;
        }
        _ => {
            sqlx::query(
                "UPDATE account_suspension SET ends_at = NOW() WHERE account_id = $1 AND starts_at <= NOW() AND ends_at IS NULL",
            )
            .bind(account_id)
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                "UPDATE account SET account_status = $2, deleted_at = NULL, updated_at = NOW() WHERE id = $1",
            )
            .bind(account_id)
            .bind(status)
            .execute(&mut **tx)
            .await?;
        }
    }

    Ok(())
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, HeaderMap, Json<Envelope<AuthSession>>), AppError> {
    let roles = vec!["user".to_string()];
    let session = register_account_with_roles(&state.db, &payload, &roles).await?;
    let headers = header_map_with_cookie(&session_cookie_value(
        session.user.id,
        session.expires_in_seconds,
    ));
    Ok((
        StatusCode::CREATED,
        headers,
        Json(envelope(session)),
    ))
}

async fn register_admin_bootstrap(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, HeaderMap, Json<Envelope<AuthSession>>), AppError> {
    if !state.allow_public_admin_bootstrap {
        return Err(AppError::Forbidden(
            "Public admin bootstrap is disabled".to_string(),
        ));
    }

    let roles = vec!["user".to_string(), "admin".to_string()];
    let session = register_account_with_roles(&state.db, &payload, &roles).await?;
    let headers = header_map_with_cookie(&session_cookie_value(
        session.user.id,
        session.expires_in_seconds,
    ));

    Ok((StatusCode::CREATED, headers, Json(envelope(session))))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<Envelope<AuthSession>>), AppError> {
    let account_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT account_id FROM account_email WHERE normalized_email = $1 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(payload.email.to_lowercase())
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify_account_password(&state.db, account_id, &payload.password).await? {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    ensure_account_is_accessible(&state.db, account_id).await?;

    if payload.password == DEMO_PASSWORD {
        provision_demo_password_if_missing(&state.db, account_id).await?;
    }

    let user = build_user_profile(&state.db, account_id).await?;
    let expires_in_seconds = if payload.remember_me.unwrap_or(false) {
        3600
    } else {
        900
    };
    let session = AuthSession {
        access_token: session_token_for_account(account_id),
        token_type: "Bearer".to_string(),
        expires_in_seconds,
        user,
    };
    let headers = header_map_with_cookie(&session_cookie_value(account_id, expires_in_seconds));

    Ok((headers, Json(envelope(session))))
}

async fn refresh_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, Json<Envelope<AuthSession>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let user = build_user_profile(&state.db, account_id).await?;
    let session = AuthSession {
        access_token: session_token_for_account(account_id),
        token_type: "Bearer".to_string(),
        expires_in_seconds: 900,
        user,
    };
    let headers = header_map_with_cookie(&session_cookie_value(
        account_id,
        session.expires_in_seconds,
    ));

    Ok((headers, Json(envelope(session))))
}

async fn logout() -> (StatusCode, HeaderMap) {
    (
        StatusCode::NO_CONTENT,
        header_map_with_cookie(&expired_session_cookie_value()),
    )
}

async fn get_me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<UserProfile>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    Ok(Json(envelope(
        build_user_profile(&state.db, account_id).await?,
    )))
}

async fn update_me(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ProfileUpdateRequest>,
) -> Result<Json<Envelope<UserProfile>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let default_currency =
        normalize_supported_currency_code(&state.db, payload.default_currency.as_deref()).await?;

    sqlx::query(
        r#"
        INSERT INTO account_profile (account_id, display_name, locale, timezone_name, preferred_currency_code, profile_bio, created_at, updated_at)
        VALUES (
            $1,
            $2,
            COALESCE($3, 'en-GB'),
            COALESCE($4, 'Europe/London'),
            COALESCE($5, 'GBP'),
            NULLIF($6, ''),
            NOW(),
            NOW()
        )
        ON CONFLICT (account_id) DO UPDATE
        SET display_name = COALESCE($2, account_profile.display_name),
            locale = COALESCE($3, account_profile.locale),
            timezone_name = COALESCE($4, account_profile.timezone_name),
            preferred_currency_code = COALESCE($5, account_profile.preferred_currency_code),
            profile_bio = CASE
                WHEN $6::text IS NULL THEN account_profile.profile_bio
                ELSE NULLIF($6, '')
            END,
            updated_at = NOW()
        "#,
    )
    .bind(account_id)
    .bind(payload.display_name)
    .bind(payload.locale)
    .bind(payload.timezone_name)
    .bind(default_currency)
    .bind(payload.profile_bio)
    .execute(&state.db)
    .await?;

    Ok(Json(envelope(
        build_user_profile(&state.db, account_id).await?,
    )))
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PasswordChangeRequest>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;

    if payload.new_password.len() < 12 {
        return Err(AppError::BadRequest(
            "New password must be at least 12 characters".to_string(),
        ));
    }

    if payload.current_password == payload.new_password {
        return Err(AppError::BadRequest(
            "Choose a different password from the current one".to_string(),
        ));
    }

    if !verify_account_password(&state.db, account_id, &payload.current_password).await? {
        return Err(AppError::Unauthorized(
            "Current password is incorrect".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    store_password_credential(&mut tx, account_id, &payload.new_password).await?;
    tx.commit().await?;

    Ok(Json(envelope(Acknowledgement {
        status: "password-updated".to_string(),
    })))
}

async fn update_me_avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AvatarUpdateRequest>,
) -> Result<Json<Envelope<UserProfile>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let file_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM file_asset WHERE id = $1 AND owner_account_id = $2 AND is_deleted = FALSE)",
    )
    .bind(payload.file_id)
    .bind(account_id)
    .fetch_one(&state.db)
    .await?;

    if !file_exists {
        return Err(AppError::BadRequest(
            "Avatar file must belong to the signed-in account".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE file_attachment SET removed_at = NOW() WHERE entity_type = 'account' AND entity_id = $1 AND attachment_role = 'avatar' AND removed_at IS NULL",
    )
    .bind(account_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO file_attachment (
            id, file_asset_id, entity_type, entity_id, attachment_role, sort_order,
            is_primary, attached_by_account_id, metadata_json, created_at, removed_at
        ) VALUES ($1, $2, 'account', $3, 'avatar', 0, TRUE, $3, '{"source":"profile"}'::jsonb, NOW(), NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(payload.file_id)
    .bind(account_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(envelope(
        build_user_profile(&state.db, account_id).await?,
    )))
}

async fn delete_me_avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<UserProfile>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    sqlx::query(
        "UPDATE file_attachment SET removed_at = NOW() WHERE entity_type = 'account' AND entity_id = $1 AND attachment_role = 'avatar' AND removed_at IS NULL",
    )
    .bind(account_id)
    .execute(&state.db)
    .await?;

    Ok(Json(envelope(
        build_user_profile(&state.db, account_id).await?,
    )))
}

async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Envelope<Vec<Category>>>, AppError> {
    let rows = sqlx::query_as::<_, Category>(
        "SELECT id, name, parent_category_id AS parent_id FROM category WHERE is_active = TRUE ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn list_brands(
    State(state): State<AppState>,
) -> Result<Json<Envelope<Vec<Brand>>>, AppError> {
    let rows = sqlx::query_as::<_, Brand>(
        "SELECT id, name FROM brand WHERE is_active = TRUE ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn list_units(State(state): State<AppState>) -> Result<Json<Envelope<Vec<Unit>>>, AppError> {
    let rows = sqlx::query_as::<_, Unit>("SELECT id, name, symbol FROM unit ORDER BY name")
        .fetch_all(&state.db)
        .await?;
    Ok(Json(envelope(rows)))
}

async fn list_discount_types(
    State(state): State<AppState>,
) -> Result<Json<Envelope<Vec<DiscountType>>>, AppError> {
    let rows = sqlx::query_as::<_, DiscountType>(
        "SELECT id, name, description FROM discount_type WHERE is_active = TRUE ORDER BY name",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn list_items(
    State(state): State<AppState>,
    Query(query): Query<ListItemsQuery>,
) -> Result<Json<Envelope<Vec<ItemSummary>>>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT
            i.id,
            i.category_id,
            i.canonical_name AS name,
            i.specification_text AS specification,
            i.created_at,
            COUNT(iv.id) AS variant_count,
            MIN(CASE WHEN po.status = 'verified' THEN po.final_price_amount END) AS lowest_price
        FROM item i
        LEFT JOIN item_variant iv ON iv.item_id = i.id
        LEFT JOIN price_observation po ON po.item_variant_id = iv.id
        WHERE i.status = 'approved'
          AND ($1::TEXT IS NULL OR i.canonical_name ILIKE '%' || $1 || '%' OR COALESCE(i.specification_text, '') ILIKE '%' || $1 || '%')
        GROUP BY i.id
        ORDER BY i.canonical_name
        "#,
    )
    .bind(query.q)
    .fetch_all(&state.db)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| ItemSummary {
            id: row.get("id"),
            category_id: row.get("category_id"),
            name: row.get("name"),
            specification: row.get("specification"),
            created_at: row.get("created_at"),
            variant_summary: Some(VariantSummary {
                count: row.get("variant_count"),
                lowest_known_price: row
                    .try_get::<Option<Decimal>, _>("lowest_price")
                    .ok()
                    .flatten()
                    .map(|amount| Money {
                        amount,
                        currency: "GBP".to_string(),
                    }),
            }),
        })
        .collect();
    Ok(Json(envelope(items)))
}

async fn fetch_variant_summaries(
    db: &sqlx::PgPool,
    item_id: Option<Uuid>,
    variant_id: Option<Uuid>,
) -> Result<Vec<ItemVariantSummary>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT
            iv.id,
            iv.item_id,
            COALESCE(iv.normalized_content_quantity, iv.package_quantity) AS quantity,
            sl.listing_url AS website,
            b.id AS brand_id,
            b.name AS brand_name,
            u.id AS unit_id,
            u.name AS unit_name,
            u.symbol AS unit_symbol,
            (
                SELECT json_build_object(
                    'code', vi.identifier_value,
                    'codeType', lower(vi.identifier_type),
                    'scope', lower(vi.scope_type),
                    'shopId', vi.shop_id,
                    'label', CASE WHEN vi.scope_type = 'SHOP' THEN 'Shop code' ELSE 'Product code' END
                )
                FROM variant_identifier vi
                WHERE vi.item_variant_id = iv.id
                ORDER BY CASE WHEN vi.scope_type = 'GLOBAL' THEN 0 ELSE 1 END, vi.is_primary DESC
                LIMIT 1
            ) AS primary_product_code
        FROM item_variant iv
        JOIN brand b ON b.id = iv.brand_id
        JOIN unit u ON u.id = COALESCE(iv.normalized_content_unit_id, iv.package_unit_id)
        LEFT JOIN shop_listing sl ON sl.item_variant_id = iv.id AND sl.is_active = TRUE
        WHERE ($1::uuid IS NULL OR iv.item_id = $1)
          AND ($2::uuid IS NULL OR iv.id = $2)
          AND iv.status = 'approved'
        ORDER BY iv.id
        "#,
    )
    .bind(item_id)
    .bind(variant_id)
    .fetch_all(db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ItemVariantSummary {
            id: row.get("id"),
            item_id: row.get("item_id"),
            brand: BrandRef {
                id: row.get("brand_id"),
                name: row.get("brand_name"),
            },
            quantity: row.get("quantity"),
            unit: UnitRef {
                id: row.get("unit_id"),
                name: row.get("unit_name"),
                symbol: row.get("unit_symbol"),
            },
            primary_product_code: row
                .try_get::<Option<serde_json::Value>, _>("primary_product_code")
                .ok()
                .flatten()
                .and_then(|v| serde_json::from_value(v).ok()),
            website: row.get("website"),
        })
        .collect())
}

async fn get_item(
    State(state): State<AppState>,
    Path(item_id): Path<Uuid>,
) -> Result<Json<Envelope<ItemDetail>>, AppError> {
    let row = sqlx::query(
        "SELECT id, category_id, canonical_name AS name, specification_text AS specification, created_at FROM item WHERE id = $1 AND status = 'approved'",
    )
    .bind(item_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?;

    Ok(Json(envelope(ItemDetail {
        id: row.get("id"),
        category_id: row.get("category_id"),
        name: row.get("name"),
        specification: row.get("specification"),
        created_at: row.get("created_at"),
        variants: fetch_variant_summaries(&state.db, Some(item_id), None).await?,
    })))
}

async fn list_item_variants(
    State(state): State<AppState>,
    Path(item_id): Path<Uuid>,
) -> Result<Json<Envelope<Vec<ItemVariantSummary>>>, AppError> {
    Ok(Json(envelope(
        fetch_variant_summaries(&state.db, Some(item_id), None).await?,
    )))
}

async fn get_item_variant(
    State(state): State<AppState>,
    Path(variant_id): Path<Uuid>,
) -> Result<Json<Envelope<ItemVariantDetail>>, AppError> {
    let summary = fetch_variant_summaries(&state.db, None, Some(variant_id))
        .await?
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("Variant not found".to_string()))?;

    let product_codes = sqlx::query(
        "SELECT json_build_object('code', identifier_value, 'codeType', lower(identifier_type), 'scope', lower(scope_type), 'shopId', shop_id, 'label', CASE WHEN scope_type = 'SHOP' THEN 'Shop code' ELSE 'Product code' END) AS value FROM variant_identifier WHERE item_variant_id = $1 ORDER BY identifier_value"
    )
    .bind(variant_id)
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .filter_map(|row| row.try_get::<serde_json::Value, _>("value").ok())
    .filter_map(|value| serde_json::from_value(value).ok())
    .collect::<Vec<ProductCode>>();

    let latest_known_price = sqlx::query("SELECT final_price_amount FROM price_observation WHERE item_variant_id = $1 AND status = 'verified' ORDER BY observed_at DESC LIMIT 1")
        .bind(variant_id)
        .fetch_optional(&state.db)
        .await?
        .map(|row| Money {
            amount: row.get("final_price_amount"),
            currency: "GBP".to_string(),
        });

    Ok(Json(envelope(ItemVariantDetail {
        summary,
        latest_known_price,
        product_codes,
    })))
}

fn row_to_public_price(row: PgRow) -> PublicPrice {
    let quantity: Decimal = row.get("quantity");
    let final_amount: Decimal = row.get("final_price_amount");
    let unit_symbol: String = row.get("unit_symbol");
    PublicPrice {
        item_variant_id: row.get("item_variant_id"),
        shop: ShopSummary {
            id: row.get("shop_id"),
            name: row.get("shop_name"),
            display_address: row.get("display_address"),
        },
        price: PriceBreakdown {
            original_amount: row.get("list_price_amount"),
            currency: row.get("currency_code"),
            discount_amount: row.get("discount_amount"),
            final_amount,
            unit_price: row.try_get("unit_price_amount").unwrap_or_else(|_| {
                if quantity.is_zero() {
                    final_amount
                } else {
                    final_amount / quantity
                }
            }),
            unit_label: format!("GBP/{unit_symbol}"),
        },
        recorded_at: row.get("observed_at"),
        verification: if row.get::<String, _>("status") == "verified" {
            "moderator".to_string()
        } else {
            "community".to_string()
        },
    }
}

async fn list_variant_prices(
    State(state): State<AppState>,
    Path(variant_id): Path<Uuid>,
) -> Result<Json<Envelope<Vec<PublicPrice>>>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT
            po.item_variant_id,
            COALESCE(po.list_price_amount, po.final_price_amount) AS list_price_amount,
            po.currency_code,
            po.discount_amount,
            po.final_price_amount,
            po.unit_price_amount,
            po.observed_at,
            po.status,
            s.id AS shop_id,
            s.name AS shop_name,
            a.full_text AS display_address,
            COALESCE(iv.normalized_content_quantity, iv.package_quantity) AS quantity,
            u.symbol AS unit_symbol
        FROM price_observation po
        JOIN shop s ON s.id = po.shop_id
        LEFT JOIN address a ON a.id = s.address_id
        JOIN item_variant iv ON iv.id = po.item_variant_id
        JOIN unit u ON u.id = COALESCE(iv.normalized_content_unit_id, iv.package_unit_id)
        WHERE po.item_variant_id = $1 AND po.status = 'verified'
        ORDER BY po.observed_at DESC
        "#,
    )
    .bind(variant_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(
        rows.into_iter().map(row_to_public_price).collect(),
    )))
}

async fn get_variant_price_history(
    State(state): State<AppState>,
    Path(variant_id): Path<Uuid>,
) -> Result<Json<Envelope<PriceHistory>>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT po.observed_at, po.final_price_amount, po.unit_price_amount, COALESCE(iv.normalized_content_quantity, iv.package_quantity) AS quantity, u.symbol AS unit_symbol
        FROM price_observation po
        JOIN item_variant iv ON iv.id = po.item_variant_id
        JOIN unit u ON u.id = COALESCE(iv.normalized_content_unit_id, iv.package_unit_id)
        WHERE po.item_variant_id = $1 AND po.status = 'verified'
        ORDER BY po.observed_at ASC
        "#,
    )
    .bind(variant_id)
    .fetch_all(&state.db)
    .await?;

    let unit_label = rows
        .first()
        .map(|row| format!("GBP/{}", row.get::<String, _>("unit_symbol")))
        .unwrap_or_else(|| "GBP/unit".to_string());
    let points = rows
        .into_iter()
        .map(|row| {
            let final_amount: Decimal = row.get("final_price_amount");
            let quantity: Decimal = row.get("quantity");
            PriceHistoryPoint {
                recorded_at: row.get("observed_at"),
                final_amount,
                unit_price: row.try_get("unit_price_amount").unwrap_or_else(|_| {
                    if quantity.is_zero() {
                        final_amount
                    } else {
                        final_amount / quantity
                    }
                }),
            }
        })
        .collect();

    Ok(Json(envelope(PriceHistory {
        item_variant_id: variant_id,
        currency: "GBP".to_string(),
        unit_label,
        points,
    })))
}

async fn compare_variants_internal(
    db: &sqlx::PgPool,
    variant_ids: Vec<Uuid>,
) -> Result<Comparison, AppError> {
    let mut results = Vec::new();
    for variant_id in variant_ids {
        if let Some(item_variant) = fetch_variant_summaries(db, None, Some(variant_id))
            .await?
            .into_iter()
            .next()
        {
            let offers = sqlx::query(
                r#"
                SELECT
                    po.item_variant_id,
                    COALESCE(po.list_price_amount, po.final_price_amount) AS list_price_amount,
                    po.currency_code,
                    po.discount_amount,
                    po.final_price_amount,
                    po.unit_price_amount,
                    po.observed_at,
                    po.status,
                    s.id AS shop_id,
                    s.name AS shop_name,
                    a.full_text AS display_address,
                    COALESCE(iv.normalized_content_quantity, iv.package_quantity) AS quantity,
                    u.symbol AS unit_symbol
                FROM price_observation po
                JOIN shop s ON s.id = po.shop_id
                LEFT JOIN address a ON a.id = s.address_id
                JOIN item_variant iv ON iv.id = po.item_variant_id
                JOIN unit u ON u.id = COALESCE(iv.normalized_content_unit_id, iv.package_unit_id)
                WHERE po.item_variant_id = $1 AND po.status = 'verified'
                ORDER BY po.final_price_amount ASC, po.observed_at DESC
                "#,
            )
            .bind(variant_id)
            .fetch_all(db)
            .await?
            .into_iter()
            .map(row_to_public_price)
            .collect::<Vec<_>>();
            let best_offer = offers.first().cloned();
            results.push(ComparisonResult {
                item_variant,
                best_offer,
                offers,
            });
        }
    }

    Ok(Comparison {
        compared_at: Utc::now(),
        results,
    })
}

async fn compare_variants_query(
    State(state): State<AppState>,
    Query(query): Query<CompareQuery>,
) -> Result<Json<Envelope<Comparison>>, AppError> {
    let ids = query
        .variant_ids
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(Uuid::parse_str)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| {
            AppError::BadRequest("variantIds must be a comma-separated UUID list".to_string())
        })?;
    Ok(Json(envelope(
        compare_variants_internal(&state.db, ids).await?,
    )))
}

async fn compare_variants_body(
    State(state): State<AppState>,
    Json(payload): Json<ComparisonRequest>,
) -> Result<Json<Envelope<Comparison>>, AppError> {
    Ok(Json(envelope(
        compare_variants_internal(&state.db, payload.variant_ids).await?,
    )))
}

async fn list_shops(
    State(state): State<AppState>,
) -> Result<Json<Envelope<Vec<ShopSummary>>>, AppError> {
    let rows = sqlx::query_as::<_, ShopSummary>(
        r#"
        SELECT s.id, s.name, a.full_text AS display_address
        FROM shop s
        LEFT JOIN address a ON a.id = s.address_id
        WHERE s.is_active = TRUE
        ORDER BY s.name
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn get_shop(
    State(state): State<AppState>,
    Path(shop_id): Path<Uuid>,
) -> Result<Json<Envelope<ShopDetail>>, AppError> {
    let row = sqlx::query_as::<_, ShopDetail>(
        r#"
        SELECT s.id, s.name, a.full_text AS display_address, s.is_active AS is_verified
        FROM shop s
        LEFT JOIN address a ON a.id = s.address_id
        WHERE s.id = $1
        "#,
    )
    .bind(shop_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Shop not found".to_string()))?;
    Ok(Json(envelope(row)))
}

async fn lookup_product_code(
    State(state): State<AppState>,
    Path((shop_id, code)): Path<(Uuid, String)>,
) -> Result<Json<Envelope<serde_json::Value>>, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            iv.id AS variant_id,
            i.canonical_name AS item_name,
            b.name AS brand_name
        FROM variant_identifier vi
        JOIN item_variant iv ON iv.id = vi.item_variant_id
        JOIN item i ON i.id = iv.item_id
        JOIN brand b ON b.id = iv.brand_id
        WHERE vi.identifier_value = $1 AND (vi.shop_id = $2 OR vi.scope_type = 'GLOBAL')
        LIMIT 1
        "#,
    )
    .bind(code)
    .bind(shop_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Product code not found".to_string()))?;

    Ok(Json(envelope(json!({
        "variantId": row.get::<Uuid, _>("variant_id"),
        "itemName": row.get::<String, _>("item_name"),
        "brandName": row.get::<String, _>("brand_name"),
    }))))
}

async fn create_file_upload_intent(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<FileUploadIntentRequest>,
) -> Result<(StatusCode, Json<Envelope<FileUploadIntent>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let storage_object_id = Uuid::new_v4();
    let file_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO storage_object (
            id, storage_provider, bucket_name, object_key, checksum_sha256, size_bytes, encryption_key_ref, created_at, deleted_at
        ) VALUES ($1, 'local-dev', 'pricetracker-private', $2, $3, $4, NULL, NOW(), NULL)
        "#,
    )
    .bind(storage_object_id)
    .bind(format!("uploads/{file_id}/{}", payload.filename))
    .bind(
        payload
            .checksum_sha256
            .clone()
            .unwrap_or_else(|| "0000000000000000000000000000000000000000000000000000000000000000".to_string()),
    )
    .bind(payload.size)
    .execute(&state.db)
    .await?;
    sqlx::query(
        r#"
        INSERT INTO file_asset (
            id, storage_object_id, owner_account_id, original_filename, mime_type, file_extension,
            purpose_code, classification_code, is_deleted, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, 'private', FALSE, NOW())
        "#,
    )
    .bind(file_id)
    .bind(storage_object_id)
    .bind(account_id)
    .bind(&payload.filename)
    .bind(&payload.content_type)
    .bind(file_extension(&payload.filename))
    .bind(&payload.purpose)
    .execute(&state.db)
    .await?;

    let mut required_headers = std::collections::HashMap::new();
    required_headers.insert("Content-Type".to_string(), payload.content_type);

    Ok((
        StatusCode::CREATED,
        Json(envelope(FileUploadIntent {
            file_id,
            upload_url: format!("https://uploads.example.com/signed/{file_id}"),
            expires_at: Utc::now() + Duration::minutes(15),
            required_headers,
        })),
    ))
}

async fn complete_file_upload(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<Uuid>,
) -> Result<Json<Envelope<FileRecord>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query_as::<_, FileRecord>(
        r#"
        SELECT
            fa.id,
            fa.original_filename AS filename,
            fa.mime_type AS content_type,
            so.size_bytes AS size,
            fa.purpose_code AS purpose,
            CASE WHEN fa.is_deleted THEN 'deleted' ELSE 'ready' END AS status,
            TRUE AS metadata_stripped,
            fa.created_at
        FROM file_asset fa
        JOIN storage_object so ON so.id = fa.storage_object_id
        WHERE fa.id = $1 AND fa.owner_account_id = $2
        "#,
    )
    .bind(file_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;
    Ok(Json(envelope(row)))
}

async fn get_own_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<Uuid>,
) -> Result<Json<Envelope<FileRecord>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query_as::<_, FileRecord>(
        r#"
        SELECT
            fa.id,
            fa.original_filename AS filename,
            fa.mime_type AS content_type,
            so.size_bytes AS size,
            fa.purpose_code AS purpose,
            CASE WHEN fa.is_deleted THEN 'deleted' ELSE 'ready' END AS status,
            TRUE AS metadata_stripped,
            fa.created_at
        FROM file_asset fa
        JOIN storage_object so ON so.id = fa.storage_object_id
        WHERE fa.id = $1 AND fa.owner_account_id = $2
        "#,
    )
    .bind(file_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;
    Ok(Json(envelope(row)))
}

async fn get_own_file_download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(file_id): Path<Uuid>,
) -> Result<Json<Envelope<FileDownload>>, AppError> {
    let _ = get_own_file(State(state.clone()), headers, Path(file_id)).await?;
    Ok(Json(envelope(FileDownload {
        url: format!("https://downloads.example.com/signed/{file_id}"),
        expires_at: Utc::now() + Duration::minutes(10),
    })))
}

async fn fetch_purchase(
    db: &sqlx::PgPool,
    purchase_id: Uuid,
    account_id: Uuid,
) -> Result<Purchase, AppError> {
    let row = sqlx::query(
        "SELECT id, shop_id, purchased_at, notes, created_at, updated_at FROM purchase WHERE id = $1 AND purchaser_account_id = $2",
    )
    .bind(purchase_id)
    .bind(account_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Purchase not found".to_string()))?;

    let attachments = sqlx::query_as::<_, FileRecord>(
        r#"
        SELECT
            fa.id,
            fa.original_filename AS filename,
            fa.mime_type AS content_type,
            so.size_bytes AS size,
            fa.purpose_code AS purpose,
            CASE WHEN fa.is_deleted THEN 'deleted' ELSE 'ready' END AS status,
            TRUE AS metadata_stripped,
            fa.created_at
        FROM file_attachment f
        JOIN file_asset fa ON fa.id = f.file_asset_id
        JOIN storage_object so ON so.id = fa.storage_object_id
        WHERE f.entity_type = 'purchase' AND f.entity_id = $1 AND f.removed_at IS NULL
        ORDER BY fa.created_at
        "#,
    )
    .bind(purchase_id)
    .fetch_all(db)
    .await?;

    Ok(Purchase {
        id: row.get("id"),
        shop_id: row.get("shop_id"),
        purchase_time: row.get("purchased_at"),
        attachments,
        notes: row.get("notes"),
        status: "active".to_string(),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

async fn create_purchase(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PurchaseCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<Purchase>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let purchase_id = Uuid::new_v4();
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO purchase (id, purchaser_account_id, shop_id, purchased_at, currency_code, receipt_number, seller_tax_identifier, total_amount, tax_amount, notes, created_at, updated_at) VALUES ($1, $2, $3, $4, 'GBP', NULL, NULL, NULL, NULL, $5, NOW(), NOW())")
        .bind(purchase_id)
        .bind(account_id)
        .bind(payload.shop_id)
        .bind(payload.purchase_time)
        .bind(payload.notes)
        .execute(&mut *tx)
        .await?;
    if let Some(file_ids) = payload.attachment_file_ids {
        for (index, file_id) in file_ids.into_iter().enumerate() {
            sqlx::query(
                "INSERT INTO file_attachment (id, file_asset_id, entity_type, entity_id, attachment_role, sort_order, is_primary, attached_by_account_id, metadata_json, created_at, removed_at) VALUES ($1, $2, 'purchase', $3, 'receipt', $4, $5, $6, NULL, NOW(), NULL)",
            )
            .bind(Uuid::new_v4())
            .bind(file_id)
            .bind(purchase_id)
            .bind(index as i32)
            .bind(index == 0)
            .bind(account_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(envelope(
            fetch_purchase(&state.db, purchase_id, account_id).await?,
        )),
    ))
}

async fn list_own_purchases(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<Purchase>>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM purchase WHERE purchaser_account_id = $1 ORDER BY purchased_at DESC",
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    let mut purchases = Vec::new();
    for id in ids {
        purchases.push(fetch_purchase(&state.db, id, account_id).await?);
    }
    Ok(Json(envelope(purchases)))
}

async fn get_own_purchase(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(purchase_id): Path<Uuid>,
) -> Result<Json<Envelope<Purchase>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    Ok(Json(envelope(
        fetch_purchase(&state.db, purchase_id, account_id).await?,
    )))
}

async fn update_own_purchase(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(purchase_id): Path<Uuid>,
    Json(payload): Json<PurchaseUpdateRequest>,
) -> Result<Json<Envelope<Purchase>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    sqlx::query(
        r#"
        UPDATE purchase
        SET purchased_at = COALESCE($3, purchased_at),
            notes = COALESCE($4, notes),
            updated_at = NOW()
        WHERE id = $1 AND purchaser_account_id = $2
        "#,
    )
    .bind(purchase_id)
    .bind(account_id)
    .bind(payload.purchase_time)
    .bind(payload.notes)
    .execute(&state.db)
    .await?;

    if let Some(file_ids) = payload.attachment_file_ids {
        sqlx::query("UPDATE file_attachment SET removed_at = NOW() WHERE entity_type = 'purchase' AND entity_id = $1 AND removed_at IS NULL")
            .bind(purchase_id)
            .execute(&state.db)
            .await?;
        for (index, file_id) in file_ids.into_iter().enumerate() {
            sqlx::query(
                "INSERT INTO file_attachment (id, file_asset_id, entity_type, entity_id, attachment_role, sort_order, is_primary, attached_by_account_id, metadata_json, created_at, removed_at) VALUES ($1, $2, 'purchase', $3, 'receipt', $4, $5, $6, NULL, NOW(), NULL)",
            )
            .bind(Uuid::new_v4())
            .bind(file_id)
            .bind(purchase_id)
            .bind(index as i32)
            .bind(index == 0)
            .bind(account_id)
            .execute(&state.db)
            .await?;
        }
    }

    Ok(Json(envelope(
        fetch_purchase(&state.db, purchase_id, account_id).await?,
    )))
}

async fn delete_own_purchase(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(purchase_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    sqlx::query("DELETE FROM file_attachment WHERE entity_type = 'purchase' AND entity_id = $1")
        .bind(purchase_id)
        .execute(&state.db)
        .await?;
    sqlx::query("DELETE FROM purchase_line WHERE purchase_id = $1")
        .bind(purchase_id)
        .execute(&state.db)
        .await?;
    sqlx::query("DELETE FROM purchase WHERE id = $1 AND purchaser_account_id = $2")
        .bind(purchase_id)
        .bind(account_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn fetch_price_submission(
    db: &sqlx::PgPool,
    price_id: Uuid,
    account_id: Uuid,
) -> Result<PriceSubmission, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            po.id,
            po.item_variant_id,
            pl.purchase_id,
            COALESCE(po.list_price_amount, po.final_price_amount) AS original_amount,
            po.currency_code AS original_currency,
            po.discount_amount,
            po.currency_code AS discount_currency,
            po.discount_type_id,
            po.final_price_amount AS final_amount,
            po.status AS submission_status,
            po.observed_at AS recorded_at,
            po.notes,
            po.created_at
        FROM price_observation po
        JOIN purchase_line pl ON pl.price_observation_id = po.id
        JOIN purchase p ON p.id = pl.purchase_id
        WHERE po.id = $1 AND p.purchaser_account_id = $2
        "#,
    )
    .bind(price_id)
    .bind(account_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Price submission not found".to_string()))?;

    Ok(PriceSubmission {
        id: row.get("id"),
        item_variant_id: row.get("item_variant_id"),
        purchase_id: row.get("purchase_id"),
        original_amount: row.get("original_amount"),
        original_currency: row.get("original_currency"),
        discount_amount: row.get("discount_amount"),
        discount_currency: row.get("discount_currency"),
        discount_type_id: row.get("discount_type_id"),
        final_amount: row.get("final_amount"),
        submission_status: row.get("submission_status"),
        visibility: if row.get::<String, _>("submission_status") == "verified" {
            "public".to_string()
        } else {
            "private".to_string()
        },
        published: row.get::<String, _>("submission_status") == "verified",
        recorded_at: row.get("recorded_at"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
    })
}

async fn create_price_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PriceCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<PriceSubmission>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let purchase_shop_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT shop_id FROM purchase WHERE id = $1 AND purchaser_account_id = $2",
    )
    .bind(payload.purchase_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Purchase not found for current user".to_string()))?;
    let source_id = ensure_user_source(&state.db, account_id).await?;
    let final_amount = payload.original_amount - payload.discount_amount.unwrap_or(Decimal::ZERO);
    let price_id = Uuid::new_v4();
    let quantity = sqlx::query_scalar::<_, Decimal>(
        "SELECT COALESCE(normalized_content_quantity, package_quantity) FROM item_variant WHERE id = $1",
    )
    .bind(payload.item_variant_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::BadRequest("Item variant not found".to_string()))?;
    let unit_price = if quantity.is_zero() {
        final_amount
    } else {
        final_amount / quantity
    };
    let mut tx = state.db.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO price_observation (
            id, item_variant_id, shop_id, shop_listing_id, source_id, observed_at, currency_code,
            list_price_amount, final_price_amount, discount_amount, discount_type_id, unit_price_amount,
            unit_price_unit_id, status, confidence_score, notes, created_at, updated_at
        ) VALUES ($1, $2, $3, NULL, $4, $5, $6, $7, $8, $9, $10, $11,
                  (SELECT COALESCE(normalized_content_unit_id, package_unit_id) FROM item_variant WHERE id = $2),
                  'submitted', 50.00, $12, NOW(), NOW())
        "#,
    )
    .bind(price_id)
    .bind(payload.item_variant_id)
    .bind(purchase_shop_id)
    .bind(source_id)
    .bind(payload.recorded_at)
    .bind(payload.original_currency)
    .bind(payload.original_amount)
    .bind(final_amount)
    .bind(payload.discount_amount)
    .bind(payload.discount_type_id)
    .bind(unit_price)
    .bind(payload.notes)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO purchase_line (id, purchase_id, price_observation_id, line_number, quantity_purchased, batch_code, serial_number, vat_rate, notes, created_at) VALUES ($1, $2, $3, NULL, 1.000000, NULL, NULL, NULL, NULL, NOW())",
    )
    .bind(Uuid::new_v4())
    .bind(payload.purchase_id)
    .bind(price_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((
        StatusCode::CREATED,
        Json(envelope(
            fetch_price_submission(&state.db, price_id, account_id).await?,
        )),
    ))
}

async fn list_own_price_submissions(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<PriceSubmission>>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT po.id
        FROM price_observation po
        JOIN purchase_line pl ON pl.price_observation_id = po.id
        JOIN purchase p ON p.id = pl.purchase_id
        WHERE p.purchaser_account_id = $1
        ORDER BY po.observed_at DESC
        "#,
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    let mut submissions = Vec::new();
    for id in ids {
        submissions.push(fetch_price_submission(&state.db, id, account_id).await?);
    }
    Ok(Json(envelope(submissions)))
}

async fn get_own_price_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(price_id): Path<Uuid>,
) -> Result<Json<Envelope<PriceSubmission>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    Ok(Json(envelope(
        fetch_price_submission(&state.db, price_id, account_id).await?,
    )))
}

async fn update_own_price_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(price_id): Path<Uuid>,
    Json(payload): Json<PriceUpdateRequest>,
) -> Result<Json<Envelope<PriceSubmission>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let current = fetch_price_submission(&state.db, price_id, account_id).await?;
    let original_amount = payload.original_amount.unwrap_or(current.original_amount);
    let discount_amount = payload.discount_amount.or(current.discount_amount);
    let final_amount = original_amount - discount_amount.unwrap_or(Decimal::ZERO);
    let quantity = sqlx::query_scalar::<_, Decimal>(
        "SELECT COALESCE(normalized_content_quantity, package_quantity) FROM item_variant WHERE id = $1",
    )
    .bind(current.item_variant_id)
    .fetch_one(&state.db)
    .await?;
    let unit_price = if quantity.is_zero() {
        final_amount
    } else {
        final_amount / quantity
    };

    sqlx::query(
        r#"
        UPDATE price_observation
        SET list_price_amount = $3,
            currency_code = COALESCE($4, currency_code),
            discount_amount = $5,
            discount_type_id = COALESCE($6, discount_type_id),
            final_price_amount = $7,
            unit_price_amount = $8,
            observed_at = COALESCE($9, observed_at),
            notes = COALESCE($10, notes),
            updated_at = NOW()
        WHERE id = $1
          AND EXISTS (
              SELECT 1
              FROM purchase_line pl
              JOIN purchase p ON p.id = pl.purchase_id
              WHERE pl.price_observation_id = price_observation.id
                AND p.purchaser_account_id = $2
          )
        "#,
    )
    .bind(price_id)
    .bind(account_id)
    .bind(original_amount)
    .bind(payload.original_currency)
    .bind(discount_amount)
    .bind(payload.discount_type_id)
    .bind(final_amount)
    .bind(unit_price)
    .bind(payload.recorded_at)
    .bind(payload.notes)
    .execute(&state.db)
    .await?;

    Ok(Json(envelope(
        fetch_price_submission(&state.db, price_id, account_id).await?,
    )))
}

async fn delete_own_price_submission(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(price_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    sqlx::query(
        r#"
        DELETE FROM purchase_line
        WHERE price_observation_id = $1
          AND EXISTS (
              SELECT 1 FROM purchase p
              WHERE p.id = purchase_line.purchase_id AND p.purchaser_account_id = $2
          )
        "#,
    )
    .bind(price_id)
    .bind(account_id)
    .execute(&state.db)
    .await?;
    sqlx::query("DELETE FROM price_observation WHERE id = $1")
        .bind(price_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_watchlist(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<WatchlistEntry>>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let rows = sqlx::query_as::<_, WatchlistEntry>(
        "SELECT id, item_variant_id, created_at FROM watchlist_item WHERE account_id = $1 ORDER BY created_at DESC",
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn create_watchlist_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WatchlistCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<WatchlistEntry>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let entry = sqlx::query_as::<_, WatchlistEntry>(
        "INSERT INTO watchlist_item (account_id, item_variant_id) VALUES ($1, $2) RETURNING id, item_variant_id, created_at",
    )
    .bind(account_id)
    .bind(payload.item_variant_id)
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(envelope(entry))))
}

async fn delete_watchlist_item(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(watch_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    sqlx::query("DELETE FROM watchlist_item WHERE id = $1 AND account_id = $2")
        .bind(watch_id)
        .bind(account_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_alerts(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<Alert>>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let rows = sqlx::query_as::<_, Alert>(
        "SELECT id, item_variant_id, target_price_amount AS target_final_amount, currency_code AS currency, is_active AS is_enabled, created_at FROM price_alert WHERE account_id = $1 ORDER BY created_at DESC",
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn create_alert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AlertCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<Alert>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let alert = sqlx::query_as::<_, Alert>(
        "INSERT INTO price_alert (id, account_id, item_variant_id, shop_id, target_price_amount, currency_code, is_active, last_triggered_at, created_at, updated_at) VALUES ($1, $2, $3, NULL, $4, $5, $6, NULL, NOW(), NOW()) RETURNING id, item_variant_id, target_price_amount AS target_final_amount, currency_code AS currency, is_active AS is_enabled, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(account_id)
    .bind(payload.item_variant_id)
    .bind(payload.target_final_amount)
    .bind(payload.currency)
    .bind(payload.is_enabled)
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(envelope(alert))))
}

async fn update_alert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(alert_id): Path<Uuid>,
    Json(payload): Json<AlertUpdateRequest>,
) -> Result<Json<Envelope<Alert>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query_as::<_, Alert>(
        r#"
        UPDATE price_alert
        SET target_price_amount = COALESCE($3, target_price_amount),
            currency_code = COALESCE($4, currency_code),
            is_active = COALESCE($5, is_active),
            updated_at = NOW()
        WHERE id = $1 AND account_id = $2
        RETURNING id, item_variant_id, target_price_amount AS target_final_amount, currency_code AS currency, is_active AS is_enabled, created_at
        "#,
    )
    .bind(alert_id)
    .bind(account_id)
    .bind(payload.target_final_amount)
    .bind(payload.currency)
    .bind(payload.is_enabled)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Alert not found".to_string()))?;
    Ok(Json(envelope(row)))
}

async fn delete_alert(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(alert_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    sqlx::query("DELETE FROM price_alert WHERE id = $1 AND account_id = $2")
        .bind(alert_id)
        .bind(account_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn get_admin_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<AdminOverview>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;

    let account_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM account")
            .fetch_one(&state.db)
            .await?;
    let category_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM category")
            .fetch_one(&state.db)
            .await?;
    let brand_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM brand")
            .fetch_one(&state.db)
            .await?;
    let unit_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM unit")
            .fetch_one(&state.db)
            .await?;
    let retailer_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM retailer")
            .fetch_one(&state.db)
            .await?;
    let shop_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM shop")
            .fetch_one(&state.db)
            .await?;
    let item_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM item")
            .fetch_one(&state.db)
            .await?;
    let item_variant_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM item_variant")
            .fetch_one(&state.db)
            .await?;
    let discount_type_count =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM discount_type")
            .fetch_one(&state.db)
            .await?;
    let pending_moderation_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM price_observation WHERE status IN ('submitted', 'flagged')",
    )
    .fetch_one(&state.db)
    .await?;
    let system_setting_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM setting_definition WHERE scope_type = 'SYSTEM'",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(envelope(AdminOverview {
        account_count,
        category_count,
        brand_count,
        unit_count,
        retailer_count,
        shop_count,
        item_count,
        item_variant_count,
        discount_type_count,
        pending_moderation_count,
        system_setting_count,
        public_admin_bootstrap_enabled: state.allow_public_admin_bootstrap,
        tables: admin_table_definitions(),
    })))
}

async fn list_admin_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<AdminUserSummary>>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;

    let account_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM account ORDER BY deleted_at IS NOT NULL, created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut users = Vec::with_capacity(account_ids.len());
    for account_id in account_ids {
        users.push(build_admin_user_summary(&state.db, account_id).await?);
    }

    Ok(Json(envelope(users)))
}

async fn create_admin_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminUserCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<AdminUserSummary>>), AppError> {
    let admin_account_id = require_admin_account_id(&state.db, &headers).await?;
    let role_codes = normalize_role_codes(payload.role_codes);

    let initial_status = normalize_account_status(payload.account_status.as_deref())?;
    if initial_status == "deleted" {
        return Err(AppError::BadRequest(
            "New users cannot start in deleted status".to_string(),
        ));
    }

    let account_id = create_account_with_roles(
        &state.db,
        &payload.email,
        &payload.password,
        &payload.display_name,
        payload.primary_phone.as_deref(),
        &[],
        &role_codes,
        Some(admin_account_id),
        &initial_status,
    )
    .await?;

    write_audit_log(
        &state.db,
        admin_account_id,
        "admin.user.create",
        "account",
        Some(account_id),
        None,
        Some(json!({
            "status": initial_status,
            "roles": role_codes,
            "email": payload.email,
        })),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(envelope(build_admin_user_summary(&state.db, account_id).await?)),
    ))
}

async fn update_admin_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<AdminUserUpdateRequest>,
) -> Result<Json<Envelope<AdminUserSummary>>, AppError> {
    let admin_account_id = require_admin_account_id(&state.db, &headers).await?;
    let current_summary = build_admin_user_summary(&state.db, account_id).await?;
    let default_currency =
        normalize_supported_currency_code(&state.db, payload.default_currency.as_deref()).await?;

    let next_status = payload
        .account_status
        .as_deref()
        .map(|status| normalize_account_status(Some(status)))
        .transpose()?;
    let next_roles = payload.role_codes.clone().map(|roles| normalize_role_codes(Some(roles)));

    if account_id == admin_account_id {
        if let Some(status) = &next_status {
            if ["suspended", "disabled", "deleted"].contains(&status.as_str()) {
                return Err(AppError::BadRequest(
                    "You cannot lock the active admin account from the admin portal".to_string(),
                ));
            }
        }

        if let Some(role_codes) = &next_roles {
            if !role_codes.iter().any(|role| role == "admin") {
                return Err(AppError::BadRequest(
                    "You cannot remove the admin role from the active admin account".to_string(),
                ));
            }
        }
    }

    let mut tx = state.db.begin().await?;

    if let Some(display_name) = payload.display_name.as_deref() {
        if display_name.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Display name cannot be empty".to_string(),
            ));
        }
    }

    sqlx::query(
        r#"
        INSERT INTO account_profile (
            account_id, display_name, locale, timezone_name, preferred_currency_code,
            profile_bio, created_at, updated_at
        )
        VALUES (
            $1,
            COALESCE($2, 'User'),
            COALESCE($3, 'en-GB'),
            COALESCE($4, 'Europe/London'),
            COALESCE($5, 'GBP'),
            NULLIF($6, ''),
            NOW(),
            NOW()
        )
        ON CONFLICT (account_id) DO UPDATE
        SET display_name = COALESCE($2, account_profile.display_name),
            locale = COALESCE($3, account_profile.locale),
            timezone_name = COALESCE($4, account_profile.timezone_name),
            preferred_currency_code = COALESCE($5, account_profile.preferred_currency_code),
            profile_bio = CASE
                WHEN $6 IS NULL THEN account_profile.profile_bio
                ELSE NULLIF($6, '')
            END,
            updated_at = NOW()
        "#,
    )
    .bind(account_id)
    .bind(payload.display_name.as_deref().map(str::trim))
    .bind(payload.locale.as_deref().map(str::trim))
    .bind(payload.timezone_name.as_deref().map(str::trim))
    .bind(default_currency.as_deref())
    .bind(payload.profile_bio.as_deref())
    .execute(&mut *tx)
    .await?;

    if let Some(primary_email) = payload.primary_email.as_deref() {
        upsert_primary_email(&mut tx, account_id, primary_email).await?;
    }

    if let Some(primary_phone) = payload.primary_phone.as_deref() {
        upsert_primary_phone(&mut tx, account_id, primary_phone).await?;
    }

    if let Some(role_codes) = &next_roles {
        sync_account_roles(&mut tx, account_id, role_codes, Some(admin_account_id)).await?;
    }

    if let Some(status) = &next_status {
        set_account_status(
            &mut tx,
            admin_account_id,
            account_id,
            status,
            "Updated from admin user editor",
        )
        .await?;
    }

    tx.commit().await?;

    let updated_summary = build_admin_user_summary(&state.db, account_id).await?;
    write_audit_log(
        &state.db,
        admin_account_id,
        "admin.user.update",
        "account",
        Some(account_id),
        Some(json!(current_summary.clone())),
        Some(json!(updated_summary.clone())),
    )
    .await?;

    Ok(Json(envelope(updated_summary)))
}

async fn bulk_update_admin_users(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AdminUserBulkActionRequest>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    let admin_account_id = require_admin_account_id(&state.db, &headers).await?;
    let account_ids = payload
        .account_ids
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if account_ids.is_empty() {
        return Err(AppError::BadRequest(
            "Select at least one user account".to_string(),
        ));
    }

    let action = payload.action.trim().to_ascii_lowercase();
    let reason = payload
        .reason
        .clone()
        .unwrap_or_else(|| "Updated from admin users console".to_string());

    if ["freeze", "delete"].contains(&action.as_str()) && account_ids.contains(&admin_account_id) {
        return Err(AppError::BadRequest(
            "You cannot freeze or delete the active admin account".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;

    for account_id in &account_ids {
        match action.as_str() {
            "freeze" => set_account_status(
                &mut tx,
                admin_account_id,
                *account_id,
                "suspended",
                &reason,
            )
            .await?,
            "activate" | "restore" => set_account_status(
                &mut tx,
                admin_account_id,
                *account_id,
                "active",
                &reason,
            )
            .await?,
            "delete" => set_account_status(
                &mut tx,
                admin_account_id,
                *account_id,
                "deleted",
                &reason,
            )
            .await?,
            "set-status" => {
                let next_status = normalize_account_status(payload.status.as_deref())?;
                set_account_status(
                    &mut tx,
                    admin_account_id,
                    *account_id,
                    &next_status,
                    &reason,
                )
                .await?;
            }
            _ => {
                return Err(AppError::BadRequest(format!(
                    "Unsupported admin user action: {}",
                    payload.action
                )));
            }
        }
    }

    tx.commit().await?;

    for account_id in &account_ids {
        write_audit_log(
            &state.db,
            admin_account_id,
            &format!("admin.user.{action}"),
            "account",
            Some(*account_id),
            None,
            Some(json!({
                "action": action.clone(),
                "reason": reason.clone(),
                "status": payload.status.clone(),
            })),
        )
        .await?;
    }

    Ok(Json(envelope(Acknowledgement {
        status: format!("{action}-complete"),
    })))
}

async fn list_admin_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<AdminSystemSetting>>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;

    let rows = sqlx::query(
        r#"
        SELECT
            sd.id,
            sd.setting_key,
            sd.scope_type,
            sd.value_type,
            sd.description,
            sd.is_sensitive,
            sd.default_value_json,
            ss.setting_value_json,
            ss.updated_at,
            ss.updated_by_account_id
        FROM setting_definition sd
        LEFT JOIN system_setting ss ON ss.setting_definition_id = sd.id
        WHERE sd.scope_type = 'SYSTEM'
        ORDER BY sd.setting_key
        "#,
    )
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(admin_system_setting_from_row)
    .collect();

    Ok(Json(envelope(rows)))
}

async fn update_admin_setting(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(setting_key): Path<String>,
    Json(payload): Json<AdminSystemSettingUpdateRequest>,
) -> Result<Json<Envelope<AdminSystemSetting>>, AppError> {
    let admin_account_id = require_admin_account_id(&state.db, &headers).await?;
    let row = sqlx::query(
        r#"
        SELECT
            sd.id,
            sd.setting_key,
            sd.scope_type,
            sd.value_type,
            sd.description,
            sd.is_sensitive,
            sd.default_value_json,
            ss.setting_value_json,
            ss.updated_at,
            ss.updated_by_account_id
        FROM setting_definition sd
        LEFT JOIN system_setting ss ON ss.setting_definition_id = sd.id
        WHERE sd.scope_type = 'SYSTEM' AND sd.setting_key = $1
        LIMIT 1
        "#,
    )
    .bind(&setting_key)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Unknown system setting: {setting_key}")))?;

    let setting_id = row.get::<Uuid, _>("id");
    let value_type = row.get::<String, _>("value_type");
    validate_setting_value(&value_type, &payload.value)?;

    let old_value = row.get::<Option<Value>, _>("setting_value_json");
    let default_value = row.get::<Option<Value>, _>("default_value_json");
    let next_value = if payload.value.is_null() {
        default_value.clone().unwrap_or(Value::Null)
    } else {
        payload.value.clone()
    };

    sqlx::query(
        r#"
        INSERT INTO system_setting (setting_definition_id, setting_value_json, updated_by_account_id, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (setting_definition_id) DO UPDATE
        SET setting_value_json = EXCLUDED.setting_value_json,
            updated_by_account_id = EXCLUDED.updated_by_account_id,
            updated_at = NOW()
        "#,
    )
    .bind(setting_id)
    .bind(&next_value)
    .bind(admin_account_id)
    .execute(&state.db)
    .await?;

    write_audit_log(
        &state.db,
        admin_account_id,
        "admin.setting.update",
        "system_setting",
        Some(setting_id),
        old_value.clone(),
        Some(next_value.clone()),
    )
    .await?;

    let refreshed = sqlx::query(
        r#"
        SELECT
            sd.id,
            sd.setting_key,
            sd.scope_type,
            sd.value_type,
            sd.description,
            sd.is_sensitive,
            sd.default_value_json,
            ss.setting_value_json,
            ss.updated_at,
            ss.updated_by_account_id
        FROM setting_definition sd
        LEFT JOIN system_setting ss ON ss.setting_definition_id = sd.id
        WHERE sd.scope_type = 'SYSTEM' AND sd.setting_key = $1
        LIMIT 1
        "#,
    )
    .bind(&setting_key)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(envelope(admin_system_setting_from_row(refreshed))))
}

async fn list_admin_tables(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<AdminTableDefinition>>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;
    Ok(Json(envelope(admin_table_definitions())))
}

async fn get_admin_table_rows(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(table_id): Path<String>,
) -> Result<Json<Envelope<AdminTableRows>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;
    let table = admin_table_definition(&table_id)?;

    let rows = match table_id.as_str() {
        "categories" => sqlx::query(
            "SELECT id, name, description, parent_category_id, is_active FROM category ORDER BY name",
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "name": row.get::<String, _>("name"),
                "description": row.get::<Option<String>, _>("description"),
                "parentId": row.get::<Option<Uuid>, _>("parent_category_id"),
                "isActive": row.get::<bool, _>("is_active"),
            })
        })
        .collect(),
        "brands" => sqlx::query(
            "SELECT id, name, website_url, is_active FROM brand ORDER BY name",
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "name": row.get::<String, _>("name"),
                "websiteUrl": row.get::<Option<String>, _>("website_url"),
                "isActive": row.get::<bool, _>("is_active"),
            })
        })
        .collect(),
        "units" => sqlx::query(
            "SELECT id, unit_family_id, code, name, symbol, factor_to_base, is_base_unit FROM unit ORDER BY name",
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "unitFamilyId": row.get::<Uuid, _>("unit_family_id"),
                "code": row.get::<String, _>("code"),
                "name": row.get::<String, _>("name"),
                "symbol": row.get::<String, _>("symbol"),
                "factorToBase": row.get::<Decimal, _>("factor_to_base").to_string(),
                "isBaseUnit": row.get::<bool, _>("is_base_unit"),
            })
        })
        .collect(),
        "retailers" => sqlx::query(
            "SELECT id, name, retailer_type, website_url, is_active FROM retailer ORDER BY name",
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "name": row.get::<String, _>("name"),
                "retailerType": row.get::<String, _>("retailer_type"),
                "websiteUrl": row.get::<Option<String>, _>("website_url"),
                "isActive": row.get::<bool, _>("is_active"),
            })
        })
        .collect(),
        "shops" => sqlx::query(
            "SELECT id, retailer_id, name, timezone_name, is_online, is_active FROM shop ORDER BY name",
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "retailerId": row.get::<Uuid, _>("retailer_id"),
                "name": row.get::<String, _>("name"),
                "timezoneName": row.get::<String, _>("timezone_name"),
                "isOnline": row.get::<bool, _>("is_online"),
                "isActive": row.get::<bool, _>("is_active"),
            })
        })
        .collect(),
        "items" => sqlx::query(
            "SELECT id, category_id, canonical_name, specification_text, status FROM item ORDER BY canonical_name",
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "categoryId": row.get::<Option<Uuid>, _>("category_id"),
                "name": row.get::<String, _>("canonical_name"),
                "specification": row.get::<Option<String>, _>("specification_text"),
                "status": row.get::<String, _>("status"),
            })
        })
        .collect(),
        "item-variants" => sqlx::query(
            r#"
            SELECT
                iv.id,
                i.canonical_name AS item_name,
                b.name AS brand_name,
                iv.variant_name,
                iv.package_quantity,
                pu.name AS package_unit_name,
                iv.pack_count,
                iv.status
            FROM item_variant iv
            LEFT JOIN item i ON i.id = iv.item_id
            LEFT JOIN brand b ON b.id = iv.brand_id
            LEFT JOIN unit pu ON pu.id = iv.package_unit_id
            ORDER BY i.canonical_name, iv.variant_name NULLS FIRST, iv.created_at
            "#,
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "itemId": row
                    .get::<Option<String>, _>("item_name")
                    .unwrap_or_else(|| "Unknown item".to_string()),
                "brandId": row.get::<Option<String>, _>("brand_name"),
                "variantName": row.get::<Option<String>, _>("variant_name"),
                "packageQuantity": row.get::<Decimal, _>("package_quantity").to_string(),
                "packageUnitId": row
                    .get::<Option<String>, _>("package_unit_name")
                    .unwrap_or_else(|| "Unknown unit".to_string()),
                "packCount": row.get::<i32, _>("pack_count"),
                "status": row.get::<String, _>("status"),
            })
        })
        .collect(),
        "discount-types" => sqlx::query(
            "SELECT id, code, name, description, is_active FROM discount_type ORDER BY name",
        )
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id"),
                "code": row.get::<String, _>("code"),
                "name": row.get::<String, _>("name"),
                "description": row.get::<Option<String>, _>("description"),
                "isActive": row.get::<bool, _>("is_active"),
            })
        })
        .collect(),
        _ => Vec::new(),
    };

    let lookups = admin_table_lookups(&state.db, &table).await?;

    Ok(Json(envelope(AdminTableRows {
        table,
        rows,
        lookups,
    })))
}

async fn create_admin_table_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(table_id): Path<String>,
    Json(payload): Json<AdminRecordUpsertRequest>,
) -> Result<(StatusCode, Json<Envelope<Value>>), AppError> {
    let admin_account_id = require_admin_account_id(&state.db, &headers).await?;
    let row = match table_id.as_str() {
        "categories" => {
            let id = Uuid::new_v4();
            let name = required_string(&payload.values, "name")?;
            let description = optional_string(&payload.values, "description")?;
            let parent_id = optional_uuid(&payload.values, "parentId")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                INSERT INTO category (id, parent_category_id, name, normalized_name, description, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
                RETURNING id, name, description, parent_category_id, is_active
                "#,
            )
            .bind(id)
            .bind(parent_id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(description)
            .bind(is_active)
            .fetch_one(&state.db)
            .await
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "description": row.get::<Option<String>, _>("description"),
                    "parentId": row.get::<Option<Uuid>, _>("parent_category_id"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })?
        }
        "brands" => {
            let id = Uuid::new_v4();
            let name = required_string(&payload.values, "name")?;
            let website_url = optional_string(&payload.values, "websiteUrl")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                INSERT INTO brand (id, name, normalized_name, country_code, website_url, headquarters_address_id, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, 'GB', $4, NULL, $5, NOW(), NOW())
                RETURNING id, name, website_url, is_active
                "#,
            )
            .bind(id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(website_url)
            .bind(is_active)
            .fetch_one(&state.db)
            .await
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "websiteUrl": row.get::<Option<String>, _>("website_url"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })?
        }
        "units" => {
            let id = Uuid::new_v4();
            let code = required_string(&payload.values, "code")?;
            let name = required_string(&payload.values, "name")?;
            let symbol = required_string(&payload.values, "symbol")?;
            let factor_to_base = required_decimal(&payload.values, "factorToBase")?;
            let is_base_unit = bool_with_default(&payload.values, "isBaseUnit", false)?;
            let unit_family_id =
                resolve_unit_family_id(&state.db, &payload.values, &code, &name, is_base_unit)
                    .await?;
            sqlx::query(
                r#"
                INSERT INTO unit (id, unit_family_id, code, name, symbol, factor_to_base, is_base_unit, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
                RETURNING id, unit_family_id, code, name, symbol, factor_to_base, is_base_unit
                "#,
            )
            .bind(id)
            .bind(unit_family_id)
            .bind(&code)
            .bind(&name)
            .bind(&symbol)
            .bind(factor_to_base)
            .bind(is_base_unit)
            .fetch_one(&state.db)
            .await
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "unitFamilyId": row.get::<Uuid, _>("unit_family_id"),
                    "code": row.get::<String, _>("code"),
                    "name": row.get::<String, _>("name"),
                    "symbol": row.get::<String, _>("symbol"),
                    "factorToBase": row.get::<Decimal, _>("factor_to_base").to_string(),
                    "isBaseUnit": row.get::<bool, _>("is_base_unit"),
                })
            })?
        }
        "retailers" => {
            let id = Uuid::new_v4();
            let name = required_string(&payload.values, "name")?;
            let retailer_type = required_string(&payload.values, "retailerType")?;
            let website_url = optional_string(&payload.values, "websiteUrl")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                INSERT INTO retailer (id, name, normalized_name, retailer_type, website_url, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
                RETURNING id, name, retailer_type, website_url, is_active
                "#,
            )
            .bind(id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(&retailer_type)
            .bind(website_url)
            .bind(is_active)
            .fetch_one(&state.db)
            .await
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "retailerType": row.get::<String, _>("retailer_type"),
                    "websiteUrl": row.get::<Option<String>, _>("website_url"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })?
        }
        "shops" => {
            let id = Uuid::new_v4();
            let retailer_id = required_uuid(&payload.values, "retailerId")?;
            let name = required_string(&payload.values, "name")?;
            let timezone_name = optional_string(&payload.values, "timezoneName")?
                .unwrap_or_else(|| "Europe/London".to_string());
            let is_online = bool_with_default(&payload.values, "isOnline", false)?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                INSERT INTO shop (id, retailer_id, name, address_id, phone_number, is_online, latitude, longitude, timezone_name, is_active, created_at, updated_at)
                VALUES ($1, $2, $3, NULL, NULL, $4, NULL, NULL, $5, $6, NOW(), NOW())
                RETURNING id, retailer_id, name, timezone_name, is_online, is_active
                "#,
            )
            .bind(id)
            .bind(retailer_id)
            .bind(&name)
            .bind(is_online)
            .bind(&timezone_name)
            .bind(is_active)
            .fetch_one(&state.db)
            .await
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "retailerId": row.get::<Uuid, _>("retailer_id"),
                    "name": row.get::<String, _>("name"),
                    "timezoneName": row.get::<String, _>("timezone_name"),
                    "isOnline": row.get::<bool, _>("is_online"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })?
        }
        "items" => {
            let id = Uuid::new_v4();
            let category_id = required_uuid(&payload.values, "categoryId")?;
            let name = required_string(&payload.values, "name")?;
            let specification = optional_string(&payload.values, "specification")?;
            let status = optional_string(&payload.values, "status")?
                .unwrap_or_else(|| "approved".to_string());
            let approved_by_account_id = if status.trim().eq_ignore_ascii_case("approved") {
                Some(admin_account_id)
            } else {
                None
            };
            sqlx::query(
                r#"
                INSERT INTO item (
                    id, category_id, canonical_name, normalized_name, specification_text, description, status,
                    created_by_account_id, approved_by_account_id, created_at, updated_at
                )
                VALUES ($1, $2, $3, $4, $5, NULL, $6, $7, $8, NOW(), NOW())
                RETURNING id, category_id, canonical_name, specification_text, status
                "#,
            )
            .bind(id)
            .bind(category_id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(specification)
            .bind(&status)
            .bind(admin_account_id)
            .bind(approved_by_account_id)
            .fetch_one(&state.db)
            .await
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "categoryId": row.get::<Uuid, _>("category_id"),
                    "name": row.get::<String, _>("canonical_name"),
                    "specification": row.get::<Option<String>, _>("specification_text"),
                    "status": row.get::<String, _>("status"),
                })
            })?
        }
        "discount-types" => {
            let id = Uuid::new_v4();
            let code = required_string(&payload.values, "code")?;
            let name = required_string(&payload.values, "name")?;
            let description = optional_string(&payload.values, "description")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                INSERT INTO discount_type (id, code, name, description, is_active, created_at)
                VALUES ($1, $2, $3, $4, $5, NOW())
                RETURNING id, code, name, description, is_active
                "#,
            )
            .bind(id)
            .bind(&code)
            .bind(&name)
            .bind(description)
            .bind(is_active)
            .fetch_one(&state.db)
            .await
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "code": row.get::<String, _>("code"),
                    "name": row.get::<String, _>("name"),
                    "description": row.get::<Option<String>, _>("description"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })?
        }
        _ => {
            return Err(AppError::NotFound(format!(
                "Unknown admin table: {}",
                table_id
            )))
        }
    };

    Ok((StatusCode::CREATED, Json(envelope(row))))
}

async fn update_admin_table_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((table_id, record_id)): Path<(String, Uuid)>,
    Json(payload): Json<AdminRecordUpsertRequest>,
) -> Result<Json<Envelope<Value>>, AppError> {
    let admin_account_id = require_admin_account_id(&state.db, &headers).await?;

    let row = match table_id.as_str() {
        "categories" => {
            let name = required_string(&payload.values, "name")?;
            let description = optional_string(&payload.values, "description")?;
            let parent_id = optional_uuid(&payload.values, "parentId")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                UPDATE category
                SET parent_category_id = $2,
                    name = $3,
                    normalized_name = $4,
                    description = $5,
                    is_active = $6,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, name, description, parent_category_id, is_active
                "#,
            )
            .bind(record_id)
            .bind(parent_id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(description)
            .bind(is_active)
            .fetch_optional(&state.db)
            .await?
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "description": row.get::<Option<String>, _>("description"),
                    "parentId": row.get::<Option<Uuid>, _>("parent_category_id"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })
            .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?
        }
        "brands" => {
            let name = required_string(&payload.values, "name")?;
            let website_url = optional_string(&payload.values, "websiteUrl")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                UPDATE brand
                SET name = $2,
                    normalized_name = $3,
                    website_url = $4,
                    is_active = $5,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, name, website_url, is_active
                "#,
            )
            .bind(record_id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(website_url)
            .bind(is_active)
            .fetch_optional(&state.db)
            .await?
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "websiteUrl": row.get::<Option<String>, _>("website_url"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })
            .ok_or_else(|| AppError::NotFound("Brand not found".to_string()))?
        }
        "units" => {
            let code = required_string(&payload.values, "code")?;
            let name = required_string(&payload.values, "name")?;
            let symbol = required_string(&payload.values, "symbol")?;
            let factor_to_base = required_decimal(&payload.values, "factorToBase")?;
            let is_base_unit = bool_with_default(&payload.values, "isBaseUnit", false)?;
            let unit_family_id =
                resolve_unit_family_id(&state.db, &payload.values, &code, &name, is_base_unit)
                    .await?;
            sqlx::query(
                r#"
                UPDATE unit
                SET unit_family_id = $2,
                    code = $3,
                    name = $4,
                    symbol = $5,
                    factor_to_base = $6,
                    is_base_unit = $7
                WHERE id = $1
                RETURNING id, unit_family_id, code, name, symbol, factor_to_base, is_base_unit
                "#,
            )
            .bind(record_id)
            .bind(unit_family_id)
            .bind(&code)
            .bind(&name)
            .bind(&symbol)
            .bind(factor_to_base)
            .bind(is_base_unit)
            .fetch_optional(&state.db)
            .await?
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "unitFamilyId": row.get::<Uuid, _>("unit_family_id"),
                    "code": row.get::<String, _>("code"),
                    "name": row.get::<String, _>("name"),
                    "symbol": row.get::<String, _>("symbol"),
                    "factorToBase": row.get::<Decimal, _>("factor_to_base").to_string(),
                    "isBaseUnit": row.get::<bool, _>("is_base_unit"),
                })
            })
            .ok_or_else(|| AppError::NotFound("Unit not found".to_string()))?
        }
        "retailers" => {
            let name = required_string(&payload.values, "name")?;
            let retailer_type = required_string(&payload.values, "retailerType")?;
            let website_url = optional_string(&payload.values, "websiteUrl")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                UPDATE retailer
                SET name = $2,
                    normalized_name = $3,
                    retailer_type = $4,
                    website_url = $5,
                    is_active = $6,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, name, retailer_type, website_url, is_active
                "#,
            )
            .bind(record_id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(&retailer_type)
            .bind(website_url)
            .bind(is_active)
            .fetch_optional(&state.db)
            .await?
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "name": row.get::<String, _>("name"),
                    "retailerType": row.get::<String, _>("retailer_type"),
                    "websiteUrl": row.get::<Option<String>, _>("website_url"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })
            .ok_or_else(|| AppError::NotFound("Retailer not found".to_string()))?
        }
        "shops" => {
            let retailer_id = required_uuid(&payload.values, "retailerId")?;
            let name = required_string(&payload.values, "name")?;
            let timezone_name = optional_string(&payload.values, "timezoneName")?
                .unwrap_or_else(|| "Europe/London".to_string());
            let is_online = bool_with_default(&payload.values, "isOnline", false)?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                UPDATE shop
                SET retailer_id = $2,
                    name = $3,
                    timezone_name = $4,
                    is_online = $5,
                    is_active = $6,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, retailer_id, name, timezone_name, is_online, is_active
                "#,
            )
            .bind(record_id)
            .bind(retailer_id)
            .bind(&name)
            .bind(&timezone_name)
            .bind(is_online)
            .bind(is_active)
            .fetch_optional(&state.db)
            .await?
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "retailerId": row.get::<Uuid, _>("retailer_id"),
                    "name": row.get::<String, _>("name"),
                    "timezoneName": row.get::<String, _>("timezone_name"),
                    "isOnline": row.get::<bool, _>("is_online"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })
            .ok_or_else(|| AppError::NotFound("Shop not found".to_string()))?
        }
        "items" => {
            let category_id = required_uuid(&payload.values, "categoryId")?;
            let name = required_string(&payload.values, "name")?;
            let specification = optional_string(&payload.values, "specification")?;
            let status = optional_string(&payload.values, "status")?
                .unwrap_or_else(|| "approved".to_string());
            let approved_by_account_id = if status.trim().eq_ignore_ascii_case("approved") {
                Some(admin_account_id)
            } else {
                None
            };
            sqlx::query(
                r#"
                UPDATE item
                SET category_id = $2,
                    canonical_name = $3,
                    normalized_name = $4,
                    specification_text = $5,
                    status = $6,
                    approved_by_account_id = $7,
                    updated_at = NOW()
                WHERE id = $1
                RETURNING id, category_id, canonical_name, specification_text, status
                "#,
            )
            .bind(record_id)
            .bind(category_id)
            .bind(&name)
            .bind(normalized_key(&name))
            .bind(specification)
            .bind(&status)
            .bind(approved_by_account_id)
            .fetch_optional(&state.db)
            .await?
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "categoryId": row.get::<Uuid, _>("category_id"),
                    "name": row.get::<String, _>("canonical_name"),
                    "specification": row.get::<Option<String>, _>("specification_text"),
                    "status": row.get::<String, _>("status"),
                })
            })
            .ok_or_else(|| AppError::NotFound("Item not found".to_string()))?
        }
        "discount-types" => {
            let code = required_string(&payload.values, "code")?;
            let name = required_string(&payload.values, "name")?;
            let description = optional_string(&payload.values, "description")?;
            let is_active = bool_with_default(&payload.values, "isActive", true)?;
            sqlx::query(
                r#"
                UPDATE discount_type
                SET code = $2,
                    name = $3,
                    description = $4,
                    is_active = $5
                WHERE id = $1
                RETURNING id, code, name, description, is_active
                "#,
            )
            .bind(record_id)
            .bind(&code)
            .bind(&name)
            .bind(description)
            .bind(is_active)
            .fetch_optional(&state.db)
            .await?
            .map(|row| {
                json!({
                    "id": row.get::<Uuid, _>("id"),
                    "code": row.get::<String, _>("code"),
                    "name": row.get::<String, _>("name"),
                    "description": row.get::<Option<String>, _>("description"),
                    "isActive": row.get::<bool, _>("is_active"),
                })
            })
            .ok_or_else(|| AppError::NotFound("Discount type not found".to_string()))?
        }
        _ => {
            return Err(AppError::NotFound(format!(
                "Unknown admin table: {}",
                table_id
            )))
        }
    };

    Ok(Json(envelope(row)))
}

async fn delete_admin_table_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((table_id, record_id)): Path<(String, Uuid)>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;

    let rows_affected = match table_id.as_str() {
        "categories" => sqlx::query(
            "UPDATE category SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
        )
        .bind(record_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        "brands" => sqlx::query(
            "UPDATE brand SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
        )
        .bind(record_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        "units" => sqlx::query("DELETE FROM unit WHERE id = $1")
            .bind(record_id)
            .execute(&state.db)
            .await?
            .rows_affected(),
        "retailers" => sqlx::query(
            "UPDATE retailer SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
        )
        .bind(record_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        "shops" => sqlx::query(
            "UPDATE shop SET is_active = FALSE, updated_at = NOW() WHERE id = $1",
        )
        .bind(record_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        "items" => sqlx::query(
            "UPDATE item SET status = 'archived', updated_at = NOW() WHERE id = $1",
        )
        .bind(record_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        "discount-types" => sqlx::query(
            "UPDATE discount_type SET is_active = FALSE WHERE id = $1",
        )
        .bind(record_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        _ => {
            return Err(AppError::NotFound(format!(
                "Unknown admin table: {}",
                table_id
            )))
        }
    };

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "Record not found in {}",
            table_id
        )));
    }

    Ok(Json(envelope(Acknowledgement {
        status: "deleted".to_string(),
    })))
}

async fn approve_admin_table_row(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((table_id, record_id)): Path<(String, Uuid)>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    let admin_account_id = require_admin_account_id(&state.db, &headers).await?;

    let rows_affected = match table_id.as_str() {
        "items" => sqlx::query(
            r#"
            UPDATE item
            SET status = 'approved',
                approved_by_account_id = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(record_id)
        .bind(admin_account_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        "item-variants" => sqlx::query(
            r#"
            UPDATE item_variant
            SET status = 'approved',
                approved_by_account_id = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(record_id)
        .bind(admin_account_id)
        .execute(&state.db)
        .await?
        .rows_affected(),
        _ => {
            return Err(AppError::BadRequest(format!(
                "Approval is not supported for {}",
                table_id
            )))
        }
    };

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!(
            "Record not found in {}",
            table_id
        )));
    }

    Ok(Json(envelope(Acknowledgement {
        status: "approved".to_string(),
    })))
}

async fn list_moderation_prices(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<PriceSubmission>>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;
    let rows = sqlx::query(
        r#"
        SELECT po.id, p.purchaser_account_id AS account_id
        FROM price_observation po
        JOIN purchase_line pl ON pl.price_observation_id = po.id
        JOIN purchase p ON p.id = pl.purchase_id
        WHERE po.status IN ('submitted', 'flagged')
        ORDER BY po.created_at ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    let mut list = Vec::new();
    for row in rows {
        list.push(fetch_price_submission(&state.db, row.get("id"), row.get("account_id")).await?);
    }
    Ok(Json(envelope(list)))
}

async fn verify_moderation_price(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(price_id): Path<Uuid>,
    Json(_payload): Json<ModerateActionRequest>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;
    sqlx::query(
        "UPDATE price_observation SET status = 'verified', updated_at = NOW() WHERE id = $1",
    )
    .bind(price_id)
    .execute(&state.db)
    .await?;
    Ok(Json(envelope(Acknowledgement {
        status: "verified".to_string(),
    })))
}

async fn reject_moderation_price(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(price_id): Path<Uuid>,
    Json(_payload): Json<ModerateActionRequest>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    require_admin_account_id(&state.db, &headers).await?;
    sqlx::query(
        "UPDATE price_observation SET status = 'rejected', updated_at = NOW() WHERE id = $1",
    )
    .bind(price_id)
    .execute(&state.db)
    .await?;
    Ok(Json(envelope(Acknowledgement {
        status: "rejected".to_string(),
    })))
}

async fn get_security_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<UserSecuritySummary>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let profile = build_user_profile(&state.db, account_id).await?;
    Ok(Json(envelope(profile.security)))
}

async fn list_own_emails(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<EmailAddress>>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let rows = sqlx::query_as::<_, EmailAddress>(
        "SELECT id, email, email_role, is_login_enabled, is_primary_for_account, verified_at, created_at FROM account_email WHERE account_id = $1 AND deleted_at IS NULL ORDER BY is_primary_for_account DESC, created_at ASC",
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn create_own_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<EmailAddressCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<EmailAddress>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query_as::<_, EmailAddress>(
        "INSERT INTO account_email (id, account_id, email, normalized_email, email_role, is_login_enabled, is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, $4, $5, $6, FALSE, NULL, NULL, NOW(), NOW(), NULL) RETURNING id, email, email_role, is_login_enabled, is_primary_for_account, verified_at, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(account_id)
    .bind(&payload.email)
    .bind(payload.email.to_lowercase())
    .bind(payload.email_role.unwrap_or_else(|| "SECONDARY".to_string()))
    .bind(payload.is_login_enabled.unwrap_or(true))
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(envelope(row))))
}

async fn delete_own_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(email_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let email_row = sqlx::query(
        r#"
        SELECT is_primary_for_account
        FROM account_email
        WHERE id = $1
          AND account_id = $2
          AND deleted_at IS NULL
        "#,
    )
    .bind(email_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Email not found".to_string()))?;

    if email_row.get::<bool, _>("is_primary_for_account") {
        return Err(AppError::BadRequest(
            "Make another email primary before removing this one".to_string(),
        ));
    }

    let active_email_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account_email WHERE account_id = $1 AND deleted_at IS NULL",
    )
    .bind(account_id)
    .fetch_one(&state.db)
    .await?;

    if active_email_count <= 1 {
        return Err(AppError::BadRequest(
            "At least one active email address must remain on the account".to_string(),
        ));
    }

    let rows_affected = sqlx::query(
        r#"
        UPDATE account_email
        SET deleted_at = NOW(),
            updated_at = NOW(),
            is_login_enabled = FALSE,
            is_primary_for_account = FALSE
        WHERE id = $1
          AND account_id = $2
          AND deleted_at IS NULL
        "#,
    )
        .bind(email_id)
        .bind(account_id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("Email not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn verify_own_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(email_id): Path<Uuid>,
    Json(_payload): Json<VerificationCodeRequest>,
) -> Result<Json<Envelope<EmailAddress>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query_as::<_, EmailAddress>(
        "UPDATE account_email SET verified_at = NOW() WHERE id = $1 AND account_id = $2 RETURNING id, email, email_role, is_login_enabled, is_primary_for_account, verified_at, created_at",
    )
    .bind(email_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Email not found".to_string()))?;
    Ok(Json(envelope(row)))
}

async fn make_own_email_primary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(email_id): Path<Uuid>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let mut tx = state.db.begin().await?;
    sqlx::query("UPDATE account_email SET is_primary_for_account = FALSE WHERE account_id = $1 AND deleted_at IS NULL")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE account_email SET is_primary_for_account = TRUE WHERE id = $1 AND account_id = $2",
    )
    .bind(email_id)
    .bind(account_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(envelope(Acknowledgement {
        status: "primary-updated".to_string(),
    })))
}

async fn list_own_phones(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<PhoneNumber>>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let rows = sqlx::query_as::<_, PhoneNumber>(
        "SELECT id, e164_phone_number AS phone_number, is_primary_for_account, verified_at, created_at FROM account_phone WHERE account_id = $1 AND deleted_at IS NULL ORDER BY is_primary_for_account DESC, created_at ASC",
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn create_own_phone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PhoneNumberCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<PhoneNumber>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query_as::<_, PhoneNumber>(
        "INSERT INTO account_phone (id, account_id, e164_phone_number, extension, phone_role, is_sms_enabled, is_voice_enabled, is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at) VALUES ($1, $2, $3, NULL, 'SECONDARY', TRUE, TRUE, FALSE, NULL, NULL, NOW(), NOW(), NULL) RETURNING id, e164_phone_number AS phone_number, is_primary_for_account, verified_at, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(account_id)
    .bind(payload.phone_number)
    .fetch_one(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(envelope(row))))
}

async fn delete_own_phone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(phone_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    sqlx::query("UPDATE account_phone SET deleted_at = NOW() WHERE id = $1 AND account_id = $2")
        .bind(phone_id)
        .bind(account_id)
        .execute(&state.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn verify_own_phone(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(phone_id): Path<Uuid>,
    Json(_payload): Json<VerificationCodeRequest>,
) -> Result<Json<Envelope<PhoneNumber>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query_as::<_, PhoneNumber>(
        "UPDATE account_phone SET verified_at = NOW(), verification_method = 'code', updated_at = NOW() WHERE id = $1 AND account_id = $2 RETURNING id, e164_phone_number AS phone_number, is_primary_for_account, verified_at, created_at",
    )
    .bind(phone_id)
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("Phone not found".to_string()))?;
    Ok(Json(envelope(row)))
}

async fn make_own_phone_primary(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(phone_id): Path<Uuid>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let mut tx = state.db.begin().await?;
    sqlx::query("UPDATE account_phone SET is_primary_for_account = FALSE WHERE account_id = $1 AND deleted_at IS NULL")
        .bind(account_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE account_phone SET is_primary_for_account = TRUE WHERE id = $1 AND account_id = $2",
    )
    .bind(phone_id)
    .bind(account_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(envelope(Acknowledgement {
        status: "primary-updated".to_string(),
    })))
}

async fn list_current_legal_documents(
    State(state): State<AppState>,
) -> Result<Json<Envelope<Vec<LegalDocument>>>, AppError> {
    let rows = sqlx::query_as::<_, LegalDocument>(
        "SELECT id, notice_kind AS document_key, version_label AS version, notice_kind AS title, NULL::text AS content_url FROM privacy_notice_version WHERE retired_at IS NULL ORDER BY notice_kind, published_at DESC",
    )
    .fetch_all(&state.db)
    .await?;
    let rows = rows
        .into_iter()
        .map(|mut doc| {
            doc.title = notice_title(&doc.title);
            doc
        })
        .collect::<Vec<_>>();
    Ok(Json(envelope(rows)))
}

async fn list_own_privacy_consents(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<Vec<PrivacyConsent>>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let rows = sqlx::query_as::<_, PrivacyConsent>(
        r#"
        SELECT cr.id, pp.code AS document_key, COALESCE(pnv.version_label, 'unknown') AS version, cr.captured_at AS accepted_at
        FROM consent_record cr
        JOIN processing_purpose pp ON pp.id = cr.processing_purpose_id
        LEFT JOIN privacy_notice_version pnv ON pnv.id = cr.notice_version_id
        WHERE cr.account_id = $1
        ORDER BY cr.captured_at DESC
        "#,
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(envelope(rows)))
}

async fn accept_current_privacy_documents(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PrivacyConsentCreateRequest>,
) -> Result<(StatusCode, Json<Envelope<Vec<PrivacyConsent>>>), AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    for doc in payload.accepted_legal_documents {
        let purpose_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM processing_purpose WHERE code = $1 LIMIT 1",
        )
        .bind(&doc.document_key)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::BadRequest(format!(
                "Unsupported legal document key: {}",
                doc.document_key
            ))
        })?;
        let notice_id = current_notice_id(&state.db, &doc.document_key, Some(&doc.version)).await?;
        sqlx::query(
            r#"
            INSERT INTO consent_record (
                id, account_id, anonymous_subject_token_hash, processing_purpose_id, notice_version_id,
                consent_status, captured_via, evidence_json, captured_at, withdrawn_at
            ) VALUES ($1, $2, NULL, $3, $4, 'accepted', 'api', '{"source":"privacy-consent"}'::jsonb, NOW(), NULL)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(account_id)
        .bind(purpose_id)
        .bind(notice_id)
        .execute(&state.db)
        .await?;
    }
    let rows = sqlx::query_as::<_, PrivacyConsent>(
        r#"
        SELECT cr.id, pp.code AS document_key, COALESCE(pnv.version_label, 'unknown') AS version, cr.captured_at AS accepted_at
        FROM consent_record cr
        JOIN processing_purpose pp ON pp.id = cr.processing_purpose_id
        LEFT JOIN privacy_notice_version pnv ON pnv.id = cr.notice_version_id
        WHERE cr.account_id = $1
        ORDER BY cr.captured_at DESC
        "#,
    )
    .bind(account_id)
    .fetch_all(&state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(envelope(rows))))
}

async fn get_cookie_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<CookiePreferences>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let row = sqlx::query(
        r#"
        SELECT preferences_allowed, analytics_allowed, marketing_allowed, updated_at
        FROM cookie_consent
        WHERE account_id = $1 AND withdrawn_at IS NULL
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .bind(account_id)
    .fetch_optional(&state.db)
    .await?;

    let preferences = row.map(|row| CookiePreferences {
        analytics: row.get("analytics_allowed"),
        marketing: row.get("marketing_allowed"),
        preferences: row.get("preferences_allowed"),
        updated_at: row.get("updated_at"),
    });

    Ok(Json(envelope(preferences.unwrap_or(CookiePreferences {
        analytics: false,
        marketing: false,
        preferences: false,
        updated_at: Utc::now(),
    }))))
}

async fn update_cookie_preferences(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CookiePreferencesUpdateRequest>,
) -> Result<Json<Envelope<CookiePreferences>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let notice_id = current_notice_id(&state.db, "cookie_policy", None).await?;
    sqlx::query(
        r#"
        INSERT INTO cookie_consent (
            id, account_id, anonymous_subject_token_hash, notice_version_id, preferences_allowed,
            analytics_allowed, marketing_allowed, captured_at, updated_at, withdrawn_at
        ) VALUES ($1, $2, NULL, $3, $4, $5, $6, NOW(), NOW(), NULL)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(account_id)
    .bind(notice_id)
    .bind(payload.preferences)
    .bind(payload.analytics)
    .bind(payload.marketing)
    .execute(&state.db)
    .await?;

    Ok(Json(envelope(CookiePreferences {
        analytics: payload.analytics,
        marketing: payload.marketing,
        preferences: payload.preferences,
        updated_at: Utc::now(),
    })))
}

async fn not_implemented_ack() -> Result<Json<Envelope<Acknowledgement>>, AppError> {
    Ok(Json(envelope(Acknowledgement {
        status: "planned-not-implemented".to_string(),
    })))
}

async fn not_implemented_status() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn mfa_verify_stub() -> Json<Envelope<MfaChallenge>> {
    Json(envelope(MfaChallenge {
        challenge_id: Uuid::new_v4(),
        available_methods: vec!["totp".to_string(), "recovery_code".to_string()],
    }))
}

async fn passkey_options_stub() -> Json<Envelope<serde_json::Value>> {
    Json(envelope(json!({
        "challenge": Uuid::new_v4().to_string(),
        "rpId": "localhost",
        "timeout": 60000
    })))
}

async fn passkeys_list_stub() -> Json<Envelope<Vec<serde_json::Value>>> {
    Json(envelope(Vec::<serde_json::Value>::new()))
}
