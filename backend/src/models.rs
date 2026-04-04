use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    pub request_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
    pub utc_time: DateTime<Utc>,
    pub applied_migrations: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Envelope<T> {
    pub data: T,
    pub meta: ResponseMeta,
}

pub fn envelope<T>(data: T) -> Envelope<T> {
    Envelope {
        data,
        meta: ResponseMeta {
            request_id: Uuid::new_v4().to_string(),
        },
    }
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Brand {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub id: Uuid,
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DiscountType {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ShopSummary {
    pub id: Uuid,
    pub name: String,
    pub display_address: Option<String>,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ShopDetail {
    pub id: Uuid,
    pub name: String,
    pub display_address: Option<String>,
    pub is_verified: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariantSummary {
    pub count: i64,
    pub lowest_known_price: Option<Money>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Money {
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    pub currency: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub specification: Option<String>,
    pub created_at: DateTime<Utc>,
    pub variant_summary: Option<VariantSummary>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrandRef {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnitRef {
    pub id: Uuid,
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductCode {
    pub code: String,
    pub code_type: String,
    pub scope: String,
    pub shop_id: Option<Uuid>,
    pub label: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemVariantSummary {
    pub id: Uuid,
    pub item_id: Uuid,
    pub brand: BrandRef,
    #[serde(with = "rust_decimal::serde::str")]
    pub quantity: Decimal,
    pub unit: UnitRef,
    pub primary_product_code: Option<ProductCode>,
    pub website: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemVariantDetail {
    #[serde(flatten)]
    pub summary: ItemVariantSummary,
    pub latest_known_price: Option<Money>,
    pub product_codes: Vec<ProductCode>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDetail {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub specification: Option<String>,
    pub created_at: DateTime<Utc>,
    pub variants: Vec<ItemVariantSummary>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PriceBreakdown {
    #[serde(with = "rust_decimal::serde::str")]
    pub original_amount: Decimal,
    pub currency: String,
    #[serde(with = "rust_decimal::serde::str_option")]
    pub discount_amount: Option<Decimal>,
    #[serde(with = "rust_decimal::serde::str")]
    pub final_amount: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_price: Decimal,
    pub unit_label: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PublicPrice {
    pub item_variant_id: Uuid,
    pub shop: ShopSummary,
    pub price: PriceBreakdown,
    pub recorded_at: DateTime<Utc>,
    pub verification: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceHistoryPoint {
    pub recorded_at: DateTime<Utc>,
    #[serde(with = "rust_decimal::serde::str")]
    pub final_amount: Decimal,
    #[serde(with = "rust_decimal::serde::str")]
    pub unit_price: Decimal,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceHistory {
    pub item_variant_id: Uuid,
    pub currency: String,
    pub unit_label: String,
    pub points: Vec<PriceHistoryPoint>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonRequest {
    pub variant_ids: Vec<Uuid>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonResult {
    pub item_variant: ItemVariantSummary,
    pub best_offer: Option<PublicPrice>,
    pub offers: Vec<PublicPrice>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub compared_at: DateTime<Utc>,
    pub results: Vec<ComparisonResult>,
}

#[derive(Serialize, Deserialize, FromRow, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileRecord {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub purpose: String,
    pub status: String,
    pub metadata_stripped: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadIntentRequest {
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub purpose: String,
    pub checksum_sha256: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadIntent {
    pub file_id: Uuid,
    pub upload_url: String,
    pub expires_at: DateTime<Utc>,
    pub required_headers: std::collections::HashMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDownload {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Purchase {
    pub id: Uuid,
    pub shop_id: Uuid,
    pub purchase_time: DateTime<Utc>,
    pub attachments: Vec<FileRecord>,
    pub notes: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseCreateRequest {
    pub shop_id: Uuid,
    pub purchase_time: DateTime<Utc>,
    pub attachment_file_ids: Option<Vec<Uuid>>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseUpdateRequest {
    pub purchase_time: Option<DateTime<Utc>>,
    pub attachment_file_ids: Option<Vec<Uuid>>,
    pub notes: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceSubmission {
    pub id: Uuid,
    pub item_variant_id: Uuid,
    pub purchase_id: Uuid,
    #[serde(with = "rust_decimal::serde::str")]
    pub original_amount: Decimal,
    pub original_currency: String,
    #[serde(with = "rust_decimal::serde::str_option")]
    pub discount_amount: Option<Decimal>,
    pub discount_currency: Option<String>,
    pub discount_type_id: Option<Uuid>,
    #[serde(with = "rust_decimal::serde::str")]
    pub final_amount: Decimal,
    pub submission_status: String,
    pub visibility: String,
    pub published: bool,
    pub recorded_at: DateTime<Utc>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceCreateRequest {
    pub item_variant_id: Uuid,
    pub purchase_id: Uuid,
    #[serde(with = "rust_decimal::serde::str")]
    pub original_amount: Decimal,
    pub original_currency: String,
    #[serde(with = "rust_decimal::serde::str_option")]
    pub discount_amount: Option<Decimal>,
    pub discount_currency: Option<String>,
    pub discount_type_id: Option<Uuid>,
    pub recorded_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceUpdateRequest {
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub original_amount: Option<Decimal>,
    pub original_currency: Option<String>,
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub discount_amount: Option<Decimal>,
    pub discount_currency: Option<String>,
    pub discount_type_id: Option<Uuid>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WatchlistEntry {
    pub id: Uuid,
    pub item_variant_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchlistCreateRequest {
    pub item_variant_id: Uuid,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub id: Uuid,
    pub item_variant_id: Uuid,
    #[serde(with = "rust_decimal::serde::str")]
    pub target_final_amount: Decimal,
    pub currency: String,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertCreateRequest {
    pub item_variant_id: Uuid,
    #[serde(with = "rust_decimal::serde::str")]
    pub target_final_amount: Decimal,
    pub currency: String,
    pub is_enabled: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertUpdateRequest {
    #[serde(default, with = "rust_decimal::serde::str_option")]
    pub target_final_amount: Option<Decimal>,
    pub currency: Option<String>,
    pub is_enabled: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
    pub primary_phone: Option<String>,
    pub accepted_legal_documents: Vec<LegalDocumentAcceptance>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdateRequest {
    pub display_name: Option<String>,
    pub default_currency: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalDocumentAcceptance {
    pub document_key: String,
    pub version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSecuritySummary {
    pub password_set: bool,
    pub mfa_enabled: bool,
    pub passkey_count: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: Uuid,
    pub primary_email: String,
    pub primary_phone: Option<String>,
    pub display_name: String,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
    pub default_currency: String,
    pub email_count: i64,
    pub phone_count: i64,
    pub security: UserSecuritySummary,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSession {
    pub access_token: String,
    pub token_type: String,
    pub expires_in_seconds: i32,
    pub user: UserProfile,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddress {
    pub id: Uuid,
    pub email: String,
    pub email_role: String,
    pub is_login_enabled: bool,
    pub is_primary_for_account: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailAddressCreateRequest {
    pub email: String,
    pub email_role: Option<String>,
    pub is_login_enabled: Option<bool>,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumber {
    pub id: Uuid,
    pub phone_number: String,
    pub is_primary_for_account: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNumberCreateRequest {
    pub phone_number: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCodeRequest {
    pub code: String,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LegalDocument {
    pub id: Uuid,
    pub document_key: String,
    pub version: String,
    pub title: String,
    pub content_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyConsentCreateRequest {
    pub accepted_legal_documents: Vec<LegalDocumentAcceptance>,
}

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyConsent {
    pub id: Uuid,
    pub document_key: String,
    pub version: String,
    pub accepted_at: DateTime<Utc>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CookiePreferences {
    pub analytics: bool,
    pub marketing: bool,
    pub preferences: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CookiePreferencesUpdateRequest {
    pub analytics: bool,
    pub marketing: bool,
    pub preferences: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Acknowledgement {
    pub status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaChallenge {
    pub challenge_id: Uuid,
    pub available_methods: Vec<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModerateActionRequest {
    pub reason: Option<String>,
}
