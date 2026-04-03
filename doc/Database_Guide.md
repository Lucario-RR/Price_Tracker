# PriceTracker Database Design Review and Revised Schema

## 1. Scope

This document reviews the current database design and proposes a more normalized schema for the PriceTracker system.

It is based on the current draft schema, the API module outline, and the previously defined system structure. The revised schema is intended to support:

- item and variant catalog management
- price observations and purchase receipts
- unit normalization and comparison
- shop and location management
- user accounts, settings, security, and file upload
- watchlists, alerts, moderation, trust, and auditability

The design is relational-first and PostgreSQL-friendly, but it can be adapted to other SQL databases.

---

## 2. High-level review of the current schema

### 2.1 What is already good

The current design already separates the main business concepts reasonably well:

- `Item` for generic products
- `ItemVarient` for sellable variants
- `Category`, `Brand`, `Shop`, `Address`
- `PurchaseRecord` and `PriceRecord`
- `Unit` and `DiscountType`

That is a good foundation.

### 2.2 Main issues found

| Area | Current issue | Why it matters | Recommended fix |
|---|---|---|---|
| Naming | Mixed naming style and typos (`ItemVarient`, `LocarionID`, `RecieptID`, `CreateAt`, `FatherID`) | Increases code friction and migration mistakes | Standardize to snake_case and correct names |
| IDs | Mostly `Varchar`, but `Address.id` is `SERIAL` | Inconsistent key strategy complicates joins and APIs | Use one ID strategy across the system, preferably `UUID` |
| Timestamps | `CreateAt` is `Varchar`; `PurchaseTime` and `RecordAt` are `Time` only | Time-only values lose the date and timezone, breaking history queries | Use `TIMESTAMPTZ` for all business event timestamps |
| Shop modeling | `Shop` mixes retailer brand and physical branch | Hard to group “Tesco” as a chain while still supporting branch-level prices | Split into `retailer` and `shop` |
| Variant identifiers | `SKU` and `Website` are stored on `ItemVarient` | SKU and URL are often shop-specific, not global to the variant | Move to `item_identifier` and `shop_listing` |
| Unit conversion | `Unit` only has `BaseUnitID` | Missing conversion factor prevents reliable normalization | Add `unit_family` and `factor_to_base` |
| Purchase structure | `PurchaseRecord.ShopID` is marked as PK even though `ID` already exists | Suggests key confusion and weak normalization | Keep `purchase.id` as PK; `shop_id` should be FK only |
| Price structure | `PriceRecord` can exist without `PurchaseID`, but then it has no direct `ShopID` | You cannot compare by shop unless the price is attached to a purchase | Store `shop_id` directly on `price_observation` |
| Money fields | Separate original and discount currencies on the same row | Allows invalid mixed-currency records | Use one row currency and store list/final/discount amounts consistently |
| Receipt modeling | `RecieptID` and `FileID` are both present but unclear | Ambiguous semantics | Rename to `receipt_number` and `receipt_file_id` |
| Source modeling | `SourceID` exists but there is no supporting source table | You need a real entity for “who/what submitted this” | Add `data_source` |
| Missing reference data | `DiscountType` is declared but not defined | Incomplete lookup | Define it as a reference table |
| Missing platform tables | User, setting, auth, session, file, watchlist, alerts, moderation, votes, reports, audit not yet modeled | These are required by the system features | Add platform and community tables |

### 2.3 Normalization summary

#### First Normal Form (1NF)
Mostly acceptable, but there are still practical issues:

- time values should not be stored as text or time-only fields when they represent full events
- `Website` on `ItemVarient` is not atomic from a business perspective because variants may have multiple shop URLs
- `SKU` on `ItemVarient` is also not truly atomic because different shops and standards may have different identifiers

#### Second Normal Form (2NF)
The current design hints at partial dependency problems around purchase and price:

- `PurchaseRecord.ShopID` being flagged as PK suggests the key design is not fully settled
- price comparison depends on `shop`, but `PriceRecord` only gets shop indirectly through `PurchaseRecord`, and `PurchaseID` is nullable

#### Third Normal Form (3NF)
Several transitive dependency issues exist:

- shop chain identity and branch identity are mixed in one table
- variant website depends on shop/listing, not the variant itself
- VAT/tax registration may belong to the retailer or receipt header, not the price row
- unit base relation is incomplete without a conversion factor

---

## 3. Recommended design principles

1. **Keep business entities separate from platform entities**
   - catalog and price data should not be mixed with auth/session/security data

2. **Model “observed price” separately from “purchase receipt”**
   - not every price comes from a purchase
   - not every purchase line needs to duplicate all price fields

3. **Store one fact in one place**
   - shop-specific URLs belong in shop listings
   - global identifiers belong in item identifiers
   - auth belongs in auth tables, not in the user profile table

4. **Use clear header/line structures**
   - `purchase` is the header
   - `purchase_line` is the receipt line detail

5. **Support moderation and trust explicitly**
   - because the system includes contributed data, reports, trust, and admin review

---

## 4. Proposed logical schema overview

### Core catalog domain

- `category`
- `brand`
- `unit_family`
- `unit`
- `currency`
- `item`
- `item_variant`
- `item_identifier`
- `retailer`
- `shop`
- `shop_listing`
- `address`
- `discount_type`

### Operational data

- `data_source`
- `price_observation`
- `purchase`
- `purchase_line`

### User and platform services

- `user_account`
- `user_profile`
- `auth_identity`
- `role`
- `user_role`
- `user_setting`
- `system_setting`
- `user_session`
- `file_asset`
- `notification`

### Community and personalization

- `watchlist_item`
- `watchlist_shop`
- `price_alert`
- `price_vote`
- `price_report`

### Admin, moderation, and security

- `moderation_decision`
- `audit_log`
- `security_event`
- `user_ban`

---

## 5. Detailed revised table design

## 5.1 Reference and catalog tables

### `category`
Represents a category hierarchy for items.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| parent_category_id | UUID | Yes | FK -> category.id | adjacency-list hierarchy |
| name | VARCHAR(120) | No |  | display name |
| normalized_name | VARCHAR(120) | No |  | lowercase/search-safe name |
| description | TEXT | Yes |  | |
| is_active | BOOLEAN | No |  | default true |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(parent_category_id, normalized_name)`

---

### `brand`
Brand or manufacturer-facing brand identity.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| name | VARCHAR(160) | No |  | |
| normalized_name | VARCHAR(160) | No |  | |
| country_code | CHAR(2) | Yes |  | ISO 3166-1 alpha-2 preferred |
| website_url | TEXT | Yes |  | |
| headquarters_address_id | UUID | Yes | FK -> address.id | optional |
| is_active | BOOLEAN | No |  | default true |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(normalized_name)`

---

### `unit_family`
Logical measurement family for conversion and comparison.

Examples: mass, volume, count.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| code | VARCHAR(40) | No |  | e.g. MASS, VOLUME, COUNT |
| name | VARCHAR(80) | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(code)`

---

### `unit`
Units used for package size and normalization.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| unit_family_id | UUID | No | FK -> unit_family.id | |
| code | VARCHAR(40) | No |  | e.g. g, kg, ml, l, each |
| name | VARCHAR(80) | No |  | |
| symbol | VARCHAR(20) | No |  | |
| factor_to_base | NUMERIC(20,8) | No |  | 1000 for kg->g if base is g |
| is_base_unit | BOOLEAN | No |  | exactly one base unit per family |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(unit_family_id, code)`
- `CHECK(factor_to_base > 0)`

---

### `currency`
Reference table for ISO currencies.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| code | CHAR(3) | No | PK | ISO 4217 |
| name | VARCHAR(80) | No |  | |
| symbol | VARCHAR(10) | Yes |  | |
| minor_unit | SMALLINT | No |  | 2 for GBP/USD, etc. |
| is_active | BOOLEAN | No |  | |

---

### `item`
Canonical product identity, independent of package size.

Example: “Semi-skimmed milk”.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| category_id | UUID | Yes | FK -> category.id | |
| canonical_name | VARCHAR(200) | No |  | |
| normalized_name | VARCHAR(200) | No |  | for dedupe/search |
| specification_text | TEXT | Yes |  | flavor/cut/type, but not package size |
| description | TEXT | Yes |  | |
| status | VARCHAR(30) | No |  | draft/pending/approved/rejected/archived |
| created_by_user_id | UUID | Yes | FK -> user_account.id | nullable for system imports |
| approved_by_user_id | UUID | Yes | FK -> user_account.id | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(category_id, normalized_name, specification_text)` may be too strict for some domains; use carefully

---

### `item_variant`
Sellable package/version of an item.

Example: “Brand A semi-skimmed milk 2L”.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| item_id | UUID | No | FK -> item.id | |
| brand_id | UUID | Yes | FK -> brand.id | nullable for unbranded items |
| variant_name | VARCHAR(160) | Yes |  | optional display label |
| package_quantity | NUMERIC(18,6) | No |  | quantity per pack |
| package_unit_id | UUID | No | FK -> unit.id | |
| pack_count | INTEGER | No |  | default 1, for multipacks |
| normalized_content_quantity | NUMERIC(18,6) | Yes |  | derived or stored cache |
| normalized_content_unit_id | UUID | Yes | FK -> unit.id | typically family base unit |
| status | VARCHAR(30) | No |  | draft/pending/approved/rejected/archived |
| created_by_user_id | UUID | Yes | FK -> user_account.id | |
| approved_by_user_id | UUID | Yes | FK -> user_account.id | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(package_quantity > 0)`
- `CHECK(pack_count > 0)`
- `UNIQUE(item_id, brand_id, package_quantity, package_unit_id, pack_count, variant_name)`

---

### `item_identifier`
Global product identifiers for deduplication and external matching.

Use this instead of storing `SKU` directly on `item_variant`.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| identifier_type | VARCHAR(40) | No |  | GTIN, EAN, UPC, MPN, INTERNAL |
| identifier_value | VARCHAR(200) | No |  | |
| country_code | CHAR(2) | Yes |  | optional regional scope |
| is_primary | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(identifier_type, identifier_value)`

---

### `retailer`
Retail brand or chain.

Examples: Tesco, Aldi, Amazon.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| name | VARCHAR(160) | No |  | |
| normalized_name | VARCHAR(160) | No |  | |
| retailer_type | VARCHAR(30) | No |  | supermarket/marketplace/local_store/etc. |
| website_url | TEXT | Yes |  | chain or corporate website |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(normalized_name)`

---

### `shop`
Concrete store or shopping endpoint.

This is the place where prices are observed.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| retailer_id | UUID | No | FK -> retailer.id | |
| name | VARCHAR(200) | No |  | branch/display name |
| address_id | UUID | Yes | FK -> address.id | nullable for online-only shops |
| phone_number | VARCHAR(50) | Yes |  | |
| is_online | BOOLEAN | No |  | |
| latitude | NUMERIC(10,7) | Yes |  | |
| longitude | NUMERIC(10,7) | Yes |  | |
| timezone_name | VARCHAR(80) | Yes |  | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(retailer_id, address_id, name)`

---

### `shop_listing`
Shop-specific mapping between a variant and a retailer page/listing.

This is where `Website` and shop-specific `SKU` should live.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| shop_id | UUID | No | FK -> shop.id | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| external_sku | VARCHAR(200) | Yes |  | shop-specific sku |
| listing_url | TEXT | Yes |  | |
| first_seen_at | TIMESTAMPTZ | Yes |  | |
| last_seen_at | TIMESTAMPTZ | Yes |  | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(shop_id, item_variant_id)`
- `UNIQUE(shop_id, external_sku)` where external_sku is not null

---

### `discount_type`
Reference table for discount categories.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| code | VARCHAR(50) | No |  | CLEARANCE, MEMBER, COUPON, PROMOTION |
| name | VARCHAR(120) | No |  | |
| description | TEXT | Yes |  | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(code)`

---

### `address`
Reusable address table.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| country_code | CHAR(2) | No |  | ISO 3166-1 alpha-2 |
| building_number | VARCHAR(40) | Yes |  | |
| street_name | VARCHAR(200) | Yes |  | |
| street_line2 | VARCHAR(200) | Yes |  | |
| unit | VARCHAR(80) | Yes |  | |
| floor | VARCHAR(40) | Yes |  | |
| building_name | VARCHAR(200) | Yes |  | |
| district | VARCHAR(120) | Yes |  | |
| city | VARCHAR(120) | Yes |  | |
| state_region | VARCHAR(120) | Yes |  | |
| postal_code | VARCHAR(40) | Yes |  | |
| landmark | VARCHAR(200) | Yes |  | |
| full_text | TEXT | Yes |  | raw/unstructured version |
| latitude | NUMERIC(10,7) | Yes |  | optional |
| longitude | NUMERIC(10,7) | Yes |  | optional |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

## 5.2 Price and purchase tables

### `data_source`
Who or what created a data record.

This preserves the idea of `SourceID`, but makes it explicit and extensible.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| source_type | VARCHAR(30) | No |  | USER, ADMIN, IMPORT, CRAWLER, API |
| user_id | UUID | Yes | FK -> user_account.id | nullable for non-user sources |
| source_name | VARCHAR(160) | Yes |  | e.g. crawler job name or import source |
| trust_score | NUMERIC(5,2) | Yes |  | optional cache |
| is_verified | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `price_observation`
Canonical price fact table.

This should be the main table for comparison, history, cheapest-price lookup, and analytics.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| shop_id | UUID | No | FK -> shop.id | direct FK for comparison |
| source_id | UUID | No | FK -> data_source.id | |
| observed_at | TIMESTAMPTZ | No |  | when the price was seen or paid |
| currency_code | CHAR(3) | No | FK -> currency.code | |
| list_price_amount | NUMERIC(14,4) | Yes |  | pre-discount price |
| final_price_amount | NUMERIC(14,4) | No |  | amount actually displayed/paid for one pack |
| discount_amount | NUMERIC(14,4) | Yes |  | |
| discount_type_id | UUID | Yes | FK -> discount_type.id | |
| unit_price_amount | NUMERIC(14,6) | Yes |  | optional denormalized cache |
| unit_price_unit_id | UUID | Yes | FK -> unit.id | base unit used for display |
| status | VARCHAR(30) | No |  | pending/approved/rejected/superseded |
| confidence_score | NUMERIC(5,2) | Yes |  | |
| notes | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(final_price_amount >= 0)`
- `CHECK(list_price_amount IS NULL OR list_price_amount >= 0)`
- `CHECK(discount_amount IS NULL OR discount_amount >= 0)`
- `CHECK(list_price_amount IS NULL OR list_price_amount >= final_price_amount)`

---

### `purchase`
Receipt or purchase header.

Use this when a price observation comes from a real purchase.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| purchaser_user_id | UUID | Yes | FK -> user_account.id | |
| shop_id | UUID | No | FK -> shop.id | |
| purchased_at | TIMESTAMPTZ | No |  | full timestamp |
| currency_code | CHAR(3) | No | FK -> currency.code | header default currency |
| receipt_number | VARCHAR(200) | Yes |  | replaces ambiguous `RecieptID` |
| seller_tax_identifier | VARCHAR(200) | Yes |  | if captured from receipt |
| receipt_file_id | UUID | Yes | FK -> file_asset.id | replaces `FileID` |
| total_amount | NUMERIC(14,4) | Yes |  | optional validation |
| tax_amount | NUMERIC(14,4) | Yes |  | optional validation |
| notes | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

### `purchase_line`
Line-level details tied to a purchase.

A `purchase_line` references one canonical `price_observation` instead of duplicating the same price facts.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| purchase_id | UUID | No | FK -> purchase.id | |
| price_observation_id | UUID | No | FK -> price_observation.id | unique one-to-one for receipt-backed observations |
| line_number | INTEGER | Yes |  | line on receipt |
| quantity_purchased | NUMERIC(18,6) | No |  | number of packs bought |
| batch_code | VARCHAR(200) | Yes |  | |
| serial_number | VARCHAR(200) | Yes |  | |
| vat_rate | NUMERIC(6,3) | Yes |  | optional |
| notes | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(quantity_purchased > 0)`
- `UNIQUE(price_observation_id)`

---

## 5.3 User, auth, settings, and file tables

### `user_account`
Core user identity for application logic.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| username | VARCHAR(80) | Yes |  | optional public handle |
| email | VARCHAR(320) | Yes |  | can be unique if email login required |
| status | VARCHAR(30) | No |  | active/pending/disabled/deleted |
| email_verified_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |
| deleted_at | TIMESTAMPTZ | Yes |  | soft delete |

**Constraints**
- `UNIQUE(username)` where username is not null
- `UNIQUE(email)` where email is not null

---

### `user_profile`
Separate profile data from core account/auth data.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| user_id | UUID | No | PK/FK -> user_account.id | one-to-one |
| display_name | VARCHAR(160) | Yes |  | |
| avatar_file_id | UUID | Yes | FK -> file_asset.id | |
| preferred_currency_code | CHAR(3) | Yes | FK -> currency.code | |
| home_shop_id | UUID | Yes | FK -> shop.id | |
| locale | VARCHAR(20) | Yes |  | |
| timezone_name | VARCHAR(80) | Yes |  | |
| bio | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

### `auth_identity`
Authentication identities and credentials.

This is better than storing password data on `user_account`.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| provider | VARCHAR(40) | No |  | password/google/apple/github/etc. |
| provider_subject | VARCHAR(320) | No |  | unique provider-side identifier |
| password_hash | TEXT | Yes |  | only for password provider |
| last_login_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(provider, provider_subject)`

---

### `role`
Application roles.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| code | VARCHAR(40) | No |  | USER, MODERATOR, ADMIN |
| name | VARCHAR(80) | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(code)`

---

### `user_role`
Many-to-many between users and roles.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| user_id | UUID | No | PK/FK -> user_account.id | |
| role_id | UUID | No | PK/FK -> role.id | |
| granted_by_user_id | UUID | Yes | FK -> user_account.id | |
| granted_at | TIMESTAMPTZ | No |  | |

---

### `user_setting`
Per-user preferences.

Use a key-value model for flexibility.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| setting_key | VARCHAR(120) | No |  | e.g. theme, default_radius_km |
| setting_value_json | JSONB | No |  | flexible typed value |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(user_id, setting_key)`

---

### `system_setting`
System-wide configuration.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| setting_key | VARCHAR(120) | No |  | |
| setting_value_json | JSONB | No |  | |
| updated_by_user_id | UUID | Yes | FK -> user_account.id | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(setting_key)`

---

### `user_session`
Tracks refresh sessions or device sessions.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| refresh_token_hash | TEXT | No |  | store hash, not raw token |
| user_agent | TEXT | Yes |  | |
| ip_address | INET | Yes |  | PostgreSQL type |
| created_at | TIMESTAMPTZ | No |  | |
| expires_at | TIMESTAMPTZ | No |  | |
| revoked_at | TIMESTAMPTZ | Yes |  | |

---

### `file_asset`
General file metadata and upload tracking.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| storage_key | TEXT | No |  | path/key in object storage |
| original_filename | VARCHAR(255) | No |  | |
| content_type | VARCHAR(120) | No |  | |
| size_bytes | BIGINT | No |  | |
| checksum_sha256 | CHAR(64) | Yes |  | |
| uploaded_by_user_id | UUID | Yes | FK -> user_account.id | |
| file_purpose | VARCHAR(40) | No |  | receipt/avatar/import/etc. |
| is_deleted | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(size_bytes >= 0)`

---

### `notification`
User-facing notifications, including alert triggers and moderation outcomes.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| notification_type | VARCHAR(40) | No |  | price_alert/moderation/system |
| payload_json | JSONB | No |  | |
| read_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

## 5.4 Personalization and community tables

### `watchlist_item`
Items or variants watched by a user.

Because price alerts and dashboards are variant-driven, watching the variant is usually more useful than watching the generic item.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(user_id, item_variant_id)`

---

### `watchlist_shop`
Shops watched by a user.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| shop_id | UUID | No | FK -> shop.id | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(user_id, shop_id)`

---

### `price_alert`
Price alert rules.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| shop_id | UUID | Yes | FK -> shop.id | nullable for “any shop” |
| target_price_amount | NUMERIC(14,4) | No |  | |
| currency_code | CHAR(3) | No | FK -> currency.code | |
| is_active | BOOLEAN | No |  | |
| last_triggered_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(target_price_amount >= 0)`

---

### `price_vote`
Community validation of price observations.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| price_observation_id | UUID | No | FK -> price_observation.id | |
| user_id | UUID | No | FK -> user_account.id | |
| vote_value | SMALLINT | No |  | e.g. 1 or -1 |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(vote_value IN (-1, 1))`
- `UNIQUE(price_observation_id, user_id)`

---

### `price_report`
Reports for suspicious or incorrect price observations.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| price_observation_id | UUID | No | FK -> price_observation.id | |
| reported_by_user_id | UUID | No | FK -> user_account.id | |
| reason_code | VARCHAR(50) | No |  | DUPLICATE, WRONG_ITEM, WRONG_PRICE, SPAM |
| details | TEXT | Yes |  | |
| status | VARCHAR(30) | No |  | open/reviewed/resolved/rejected |
| resolved_by_user_id | UUID | Yes | FK -> user_account.id | |
| resolved_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

## 5.5 Admin, moderation, and security tables

### `moderation_decision`
Tracks moderation actions on user-submitted records.

This table is intentionally generic for operational simplicity. If you want strict FK enforcement per entity type, split this into multiple moderation tables later.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| entity_type | VARCHAR(40) | No |  | item/item_variant/brand/shop/price_observation |
| entity_id | UUID | No |  | referenced object id |
| decision | VARCHAR(30) | No |  | approved/rejected/merged/superseded |
| moderator_user_id | UUID | No | FK -> user_account.id | |
| notes | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `audit_log`
Immutable audit log for admin and user actions.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| actor_user_id | UUID | Yes | FK -> user_account.id | nullable for system events |
| action_code | VARCHAR(80) | No |  | CREATE_PRICE, MERGE_ITEM, BAN_USER |
| entity_type | VARCHAR(40) | No |  | |
| entity_id | UUID | Yes |  | |
| old_value_json | JSONB | Yes |  | |
| new_value_json | JSONB | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `security_event`
Tracks auth and abuse-related events.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | Yes | FK -> user_account.id | |
| ip_address | INET | Yes |  | |
| event_type | VARCHAR(50) | No |  | LOGIN_FAILED, RATE_LIMITED, TOKEN_REVOKED |
| details_json | JSONB | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `user_ban`
Supports abuse handling and moderation enforcement.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| user_id | UUID | No | FK -> user_account.id | |
| reason | TEXT | No |  | |
| starts_at | TIMESTAMPTZ | No |  | |
| ends_at | TIMESTAMPTZ | Yes |  | null for indefinite |
| created_by_user_id | UUID | No | FK -> user_account.id | moderator/admin |
| created_at | TIMESTAMPTZ | No |  | |

---

## 6. Why this is more normalized

### 6.1 `retailer` vs `shop`
This removes chain-level attributes from branch-level data.

- retailer = Tesco
- shop = Tesco Express, Baker Street

That avoids repeating retailer data on every store.

### 6.2 `item_variant` vs `item_identifier` vs `shop_listing`
This removes shop-specific and identifier-specific data from the variant.

- `item_variant` = the sellable package identity
- `item_identifier` = barcode/global identifiers
- `shop_listing` = retailer page or external SKU

### 6.3 `price_observation` vs `purchase` vs `purchase_line`
This keeps price facts separate from receipt facts.

- `price_observation` = the observed price fact
- `purchase` = receipt header
- `purchase_line` = line-specific purchase detail tied to one observation

### 6.4 `user_account` vs `user_profile` vs `auth_identity`
This avoids mixing app identity, personal profile, and auth credentials.

### 6.5 `unit_family` and `unit`
This enables exact normalization logic for analytics.

Examples:
- family MASS: g, kg
- family VOLUME: ml, l
- family COUNT: each, pack

---

## 7. Recommended indexes

### Search and catalog
- `item(normalized_name)`
- `item(category_id, normalized_name)`
- `item_variant(item_id)`
- `item_variant(brand_id)`
- `item_identifier(identifier_type, identifier_value)`
- `retailer(normalized_name)`
- `shop(retailer_id)`
- `shop(address_id)`

### Price and history
- `price_observation(item_variant_id, observed_at DESC)`
- `price_observation(shop_id, observed_at DESC)`
- `price_observation(item_variant_id, shop_id, observed_at DESC)`
- `price_observation(status, observed_at DESC)`
- `purchase(shop_id, purchased_at DESC)`
- `purchase_line(purchase_id)`

### User and personalization
- `watchlist_item(user_id)`
- `watchlist_shop(user_id)`
- `price_alert(user_id, is_active)`
- `notification(user_id, read_at)`

### Moderation and security
- `price_report(status, created_at DESC)`
- `moderation_decision(entity_type, entity_id, created_at DESC)`
- `security_event(user_id, created_at DESC)`
- `security_event(ip_address, created_at DESC)`

---

## 8. Suggested views and materialized views

These are not primary business tables, but they will make the product much faster.

### `v_latest_price_by_shop_variant`
Latest approved price for each `(shop_id, item_variant_id)`.

### `v_cheapest_current_price_by_item`
Cheapest current price among all approved latest prices for each item.

### `mv_daily_variant_price_stats`
Materialized view for analytics and graphs:

- item_variant_id
- shop_id
- price_date
- min_price
- max_price
- avg_price
- observation_count

### `v_user_trust_score`
Derived from accepted/rejected records, reports, and votes.

---

## 9. Mapping from the current schema to the proposed schema

| Current table/field | Proposed destination | Notes |
|---|---|---|
| `Item` | `item` | keep concept, rename fields |
| `Category` | `category` | rename `FatherID` -> `parent_category_id` |
| `Brand` | `brand` | rename `LocarionID` -> `headquarters_address_id` |
| `Shop` | `retailer` + `shop` | split chain from concrete branch |
| `ItemVarient` | `item_variant` | remove SKU and Website from this table |
| `ItemVarient.SKU` | `item_identifier` or `shop_listing.external_sku` | depends on whether identifier is global or shop-specific |
| `ItemVarient.Website` | `shop_listing.listing_url` | shop-specific |
| `Unit` | `unit_family` + `unit` | add conversion factor |
| `PurchaseRecord` | `purchase` | rename fields and use full timestamp |
| `PriceRecord` | `price_observation` + `purchase_line` | price fact and receipt line split |
| `PriceRecord.SourceID` | `price_observation.source_id` -> `data_source.id` | real source table |
| `DiscountType` | `discount_type` | fully define it |
| `Address` | `address` | keep concept, standardize id type |

---

## 10. Minimum viable schema if you want to keep it smaller

If you do not want the full expanded model yet, the minimum set I recommend is:

### Must-have business tables
- `category`
- `brand`
- `unit_family`
- `unit`
- `currency`
- `item`
- `item_variant`
- `item_identifier`
- `retailer`
- `shop`
- `address`
- `discount_type`
- `data_source`
- `price_observation`
- `purchase`
- `purchase_line`

### Must-have platform tables
- `user_account`
- `user_profile`
- `auth_identity`
- `role`
- `user_role`
- `user_setting`
- `user_session`
- `file_asset`

### Must-have user feature tables
- `watchlist_item`
- `price_alert`

Everything else can be added in phase 2.

---

## 11. Final recommendation

### Keep from the current design
- the core split between item, variant, purchase, price, shop, unit, and address
- the overall business intent of `SourceID`
- category hierarchy support

### Change immediately
- standardize naming
- convert all event dates to `TIMESTAMPTZ`
- split `Shop` into `retailer` and `shop`
- move `SKU` and `Website` out of `ItemVarient`
- give `price_observation` a direct `shop_id`
- define `data_source`
- define `discount_type`
- introduce user/auth/session/file/settings tables

### Add next
- watchlists and price alerts
- voting/reporting/moderation
- audit and security event logging
- derived views for latest price and cheapest price queries

This gives you a schema that is much closer to 3NF while still staying practical for a price-tracking product.
