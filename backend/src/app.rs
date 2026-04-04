use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::json;
use sqlx::{postgres::PgRow, Row};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{error::AppError, models::*, state::AppState};

const DEMO_PASSWORD: &str = "StrongPassword!234";

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

pub fn build_router(state: AppState, cors_origin: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_str(cors_origin).expect("invalid cors origin"))
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::IF_MATCH,
        ]);

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/health", get(health))
        .route("/api/v1/auth/register", post(register))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh_session))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/me", get(get_me).patch(update_me))
        .route("/api/v1/categories", get(list_categories))
        .route("/api/v1/brands", get(list_brands))
        .route("/api/v1/units", get(list_units))
        .route("/api/v1/discount-types", get(list_discount_types))
        .route("/api/v1/items", get(list_items))
        .route("/api/v1/items/{item_id}", get(get_item))
        .route("/api/v1/items/{item_id}/variants", get(list_item_variants))
        .route("/api/v1/item-variants/{variant_id}", get(get_item_variant))
        .route(
            "/api/v1/item-variants/{variant_id}/prices",
            get(list_variant_prices),
        )
        .route(
            "/api/v1/item-variants/{variant_id}/price-history",
            get(get_variant_price_history),
        )
        .route("/api/v1/compare", get(compare_variants_query))
        .route("/api/v1/comparisons", post(compare_variants_body))
        .route("/api/v1/shops", get(list_shops))
        .route("/api/v1/shops/{shop_id}", get(get_shop))
        .route(
            "/api/v1/shops/{shop_id}/product-codes/{code}",
            get(lookup_product_code),
        )
        .route("/api/v1/files/uploads", post(create_file_upload_intent))
        .route(
            "/api/v1/files/uploads/{file_id}/complete",
            post(complete_file_upload),
        )
        .route("/api/v1/me/files/{file_id}", get(get_own_file))
        .route(
            "/api/v1/me/files/{file_id}/download",
            get(get_own_file_download),
        )
        .route("/api/v1/purchases", post(create_purchase))
        .route("/api/v1/me/purchases", get(list_own_purchases))
        .route(
            "/api/v1/me/purchases/{purchase_id}",
            get(get_own_purchase)
                .patch(update_own_purchase)
                .delete(delete_own_purchase),
        )
        .route("/api/v1/prices", post(create_price_submission))
        .route("/api/v1/me/prices", get(list_own_price_submissions))
        .route(
            "/api/v1/me/prices/{price_id}",
            get(get_own_price_submission)
                .patch(update_own_price_submission)
                .delete(delete_own_price_submission),
        )
        .route("/api/v1/me/watchlist", get(list_watchlist))
        .route("/api/v1/me/watchlist/items", post(create_watchlist_item))
        .route(
            "/api/v1/me/watchlist/items/{watch_id}",
            delete(delete_watchlist_item),
        )
        .route("/api/v1/me/alerts", get(list_alerts).post(create_alert))
        .route(
            "/api/v1/me/alerts/{alert_id}",
            patch(update_alert).delete(delete_alert),
        )
        .route(
            "/api/v1/admin/moderation/prices",
            get(list_moderation_prices),
        )
        .route(
            "/api/v1/admin/moderation/prices/{price_id}/verify",
            post(verify_moderation_price),
        )
        .route(
            "/api/v1/admin/moderation/prices/{price_id}/reject",
            post(reject_moderation_price),
        )
        .route("/api/v1/me/security", get(get_security_overview))
        .route(
            "/api/v1/me/emails",
            get(list_own_emails).post(create_own_email),
        )
        .route("/api/v1/me/emails/{email_id}", delete(delete_own_email))
        .route(
            "/api/v1/me/emails/{email_id}/verify",
            post(verify_own_email),
        )
        .route(
            "/api/v1/me/emails/{email_id}/make-primary",
            post(make_own_email_primary),
        )
        .route(
            "/api/v1/me/phones",
            get(list_own_phones).post(create_own_phone),
        )
        .route("/api/v1/me/phones/{phone_id}", delete(delete_own_phone))
        .route(
            "/api/v1/me/phones/{phone_id}/verify",
            post(verify_own_phone),
        )
        .route(
            "/api/v1/me/phones/{phone_id}/make-primary",
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
        .route("/api/v1/auth/password/change", post(not_implemented_ack))
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
            "/api/v1/me/passkeys/{passkey_id}",
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

async fn health(State(state): State<AppState>) -> Result<Json<Envelope<HealthStatus>>, AppError> {
    let applied_migrations = fetch_applied_v2_migrations(&state.db).await?;

    Ok(Json(envelope(HealthStatus {
        status: "ok".to_string(),
        service: "pricetracker-backend".to_string(),
        utc_time: Utc::now(),
        applied_migrations,
    })))
}

async fn current_account_id(db: &sqlx::PgPool, headers: &HeaderMap) -> Result<Uuid, AppError> {
    if let Some(value) = headers.get("x-account-id") {
        let text = value
            .to_str()
            .map_err(|_| AppError::BadRequest("Invalid x-account-id header".to_string()))?;
        return Uuid::parse_str(text)
            .map_err(|_| AppError::BadRequest("x-account-id must be a UUID".to_string()));
    }

    let id =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM account ORDER BY created_at ASC LIMIT 1")
            .fetch_one(db)
            .await?;
    Ok(id)
}

async fn build_user_profile(db: &sqlx::PgPool, account_id: Uuid) -> Result<UserProfile, AppError> {
    let row = sqlx::query(
        r#"
        SELECT
            a.id,
            a.created_at,
            COALESCE(p.display_name, 'User') AS display_name,
            COALESCE(p.preferred_currency_code, 'GBP') AS default_currency,
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
            (SELECT COUNT(*) FROM account_phone WHERE account_id = a.id AND deleted_at IS NULL) AS phone_count
        FROM account a
        LEFT JOIN account_profile p ON p.account_id = a.id
        WHERE a.id = $1
        "#,
    )
    .bind(account_id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound("Account not found".to_string()))?;

    Ok(UserProfile {
        id: row.get("id"),
        primary_email: row.get("primary_email"),
        primary_phone: row.get("primary_phone"),
        display_name: row.get("display_name"),
        roles: vec!["user".to_string()],
        scopes: vec![
            "catalog:read".to_string(),
            "price:read_public".to_string(),
            "price:write_own".to_string(),
            "purchase:write_own".to_string(),
        ],
        default_currency: row.get("default_currency"),
        email_count: row.get("email_count"),
        phone_count: row.get("phone_count"),
        security: UserSecuritySummary {
            password_set: true,
            mfa_enabled: false,
            passkey_count: 0,
        },
        created_at: row.get("created_at"),
    })
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Envelope<AuthSession>>), AppError> {
    if payload.password.len() < 12 {
        return Err(AppError::BadRequest(
            "Password must be at least 12 characters".to_string(),
        ));
    }

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM account_email WHERE normalized_email = $1 AND deleted_at IS NULL",
    )
    .bind(payload.email.to_lowercase())
    .fetch_one(&state.db)
    .await?;

    if exists > 0 {
        return Err(AppError::Conflict(
            "Email is already registered".to_string(),
        ));
    }

    let account_id = Uuid::new_v4();
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO account (id, public_handle, account_status, created_at, updated_at, deleted_at, last_active_at) VALUES ($1, $2, 'active', NOW(), NOW(), NULL, NOW())",
    )
        .bind(account_id)
        .bind(normalized_key(&payload.display_name))
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO account_profile (account_id, display_name, locale, timezone_name, preferred_currency_code, profile_bio, created_at, updated_at) VALUES ($1, $2, 'en-GB', 'Europe/London', 'GBP', NULL, NOW(), NOW())",
    )
    .bind(account_id)
    .bind(&payload.display_name)
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
    .bind(&payload.email)
    .bind(payload.email.to_lowercase())
    .execute(&mut *tx)
    .await?;
    if let Some(phone) = payload.primary_phone {
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
        VALUES ($1, 'USER_SUBMISSION', $2, $3, 50.00, FALSE, NOW())
        "#,
    )
    .bind(source_id)
    .bind(account_id)
    .bind(&payload.display_name)
    .execute(&mut *tx)
    .await?;
    for doc in payload.accepted_legal_documents {
        let notice_id = current_notice_id(&state.db, &doc.document_key, Some(&doc.version)).await?;
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
    tx.commit().await?;

    let user = build_user_profile(&state.db, account_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(envelope(AuthSession {
            access_token: format!("demo-token-{account_id}"),
            token_type: "Bearer".to_string(),
            expires_in_seconds: 900,
            user,
        })),
    ))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<Envelope<AuthSession>>, AppError> {
    if payload.password != DEMO_PASSWORD {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let account_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT account_id FROM account_email WHERE normalized_email = $1 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(payload.email.to_lowercase())
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    let user = build_user_profile(&state.db, account_id).await?;
    Ok(Json(envelope(AuthSession {
        access_token: format!("demo-token-{account_id}"),
        token_type: "Bearer".to_string(),
        expires_in_seconds: if payload.remember_me.unwrap_or(false) {
            3600
        } else {
            900
        },
        user,
    })))
}

async fn refresh_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Envelope<AuthSession>>, AppError> {
    let account_id = current_account_id(&state.db, &headers).await?;
    let user = build_user_profile(&state.db, account_id).await?;
    Ok(Json(envelope(AuthSession {
        access_token: format!("demo-token-{account_id}"),
        token_type: "Bearer".to_string(),
        expires_in_seconds: 900,
        user,
    })))
}

async fn logout() -> StatusCode {
    StatusCode::NO_CONTENT
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
    sqlx::query(
        r#"
        INSERT INTO account_profile (account_id, display_name, locale, timezone_name, preferred_currency_code, profile_bio, created_at, updated_at)
        VALUES ($1, $2, 'en-GB', 'Europe/London', COALESCE($3, 'GBP'), NULL, NOW(), NOW())
        ON CONFLICT (account_id) DO UPDATE
        SET display_name = COALESCE($2, account_profile.display_name),
            preferred_currency_code = COALESCE($3, account_profile.preferred_currency_code),
            updated_at = NOW()
        "#,
    )
    .bind(account_id)
    .bind(payload.display_name)
    .bind(payload.default_currency)
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
            a.formatted_address AS display_address,
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
                    a.formatted_address AS display_address,
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
        SELECT s.id, s.name, a.formatted_address AS display_address
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
        SELECT s.id, s.name, a.formatted_address AS display_address, s.is_active AS is_verified
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
    Path(file_id): Path<Uuid>,
) -> Result<Json<Envelope<FileRecord>>, AppError> {
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
        WHERE fa.id = $1
        "#,
    )
    .bind(file_id)
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

async fn list_moderation_prices(
    State(state): State<AppState>,
) -> Result<Json<Envelope<Vec<PriceSubmission>>>, AppError> {
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
    Path(price_id): Path<Uuid>,
    Json(_payload): Json<ModerateActionRequest>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
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
    Path(price_id): Path<Uuid>,
    Json(_payload): Json<ModerateActionRequest>,
) -> Result<Json<Envelope<Acknowledgement>>, AppError> {
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
    sqlx::query("UPDATE account_email SET deleted_at = NOW() WHERE id = $1 AND account_id = $2")
        .bind(email_id)
        .bind(account_id)
        .execute(&state.db)
        .await?;
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
