# PriceTracker Database Design Review and Revised Schema (Revision 2)

## 1. Scope

This document updates the previous database review and revises the schema for three goals:

1. keep the **PriceTracker business model** normalized and practical;
2. extract the **user, auth, file, settings, and privacy modules** into a **general reusable platform template** that can be migrated to other projects; and
3. align the platform model with modern security and privacy requirements, including:
   - password history and password reuse prevention
   - per-secret salt
   - passkeys
   - multi-factor authentication (2FA/MFA)
   - recovery codes
   - multiple emails and phone numbers
   - many files attached to one business record
   - privacy, retention, cookie, and consent tracking

This design is relational-first and PostgreSQL-friendly.

> Important: the privacy/compliance sections below are a **technical design baseline**, not legal advice. They are intended to make compliance possible in the application and data layer.

---

## 2. What changed in this revision

Compared with the previous version, this update makes the following major changes:

- **User system is now a reusable platform template**
  - `user_account` becomes generic `account`
  - contacts are moved into `account_email` and `account_phone`
  - roles/permissions/settings are made reusable across projects

- **Authentication is now broken into reusable security tables**
  - `external_identity` for OAuth/OIDC/SAML-style providers
  - `authenticator` as the common parent for password, passkey, and TOTP
  - `password_credential`
  - `password_history`
  - `passkey_credential`
  - `totp_factor`
  - `recovery_code_set`
  - `recovery_code`

- **File system is now a reusable platform template**
  - one physical object can back one or more logical files
  - one logical file can be attached to many entities
  - one business record can have many files through `file_attachment`
  - receipt uploads no longer assume a single `receipt_file_id`

- **Privacy/compliance tables were added**
  - `privacy_notice_version`
  - `processing_purpose`
  - `consent_record`
  - `cookie_definition`
  - `cookie_consent`
  - `retention_policy`
  - `data_subject_request`
  - `legal_hold`

- **Barcode handling changed**
  - the previous document treated identifiers mostly as global identifiers
  - this revision treats barcode-like values as **scope-aware**
  - a barcode may belong to a **shop** or **retailer**, not necessarily a universal international standard
  - the old `item_identifier` becomes `variant_identifier`

---

## 3. Review of the current draft schema

The current schema still gives a useful business foundation, but several issues remain from the original draft:

| Area | Current issue | Why it matters | Recommended fix |
|---|---|---|---|
| Naming | Mixed naming and typos such as `ItemVarient`, `LocarionID`, `RecieptID`, `FatherID`, `CreateAt` | Causes migration mistakes and code friction | Standardize to snake_case and corrected domain names |
| Keys | Mostly `Varchar` IDs with one `SERIAL` address ID | Inconsistent identity strategy | Use `UUID` consistently |
| Timestamps | `CreateAt` stored as text; `RecordAt` and `PurchaseTime` stored as `Time` only | Loses date/timezone context | Use `TIMESTAMPTZ` |
| Purchase design | `PurchaseRecord.ShopID` is marked as PK even though `PurchaseRecord.ID` exists | Key confusion | Keep `purchase.id` as PK and `shop_id` as FK only |
| Variant identifiers | `SKU` and barcode-like values sit directly on `ItemVarient` | Not normalized; same variant may have many shop-specific codes | Replace with `variant_identifier` and `shop_listing` |
| Receipts | Only one `FileID` is supported on a purchase | One shopping may have multiple receipt photos | Use generic `file_attachment` |
| User/auth | User, security, settings, and cookies are only implied in the API notes | Not enough for implementation | Add reusable platform domain |
| Privacy | No retention, consent, cookie, or data subject request model | Hard to support compliance or auditing | Add privacy/compliance tables |

---

## 4. Design principles

1. **Separate reusable platform tables from PriceTracker business tables**
   - all projects can reuse `account`, `session`, `file_asset`, `consent_record`, etc.
   - only the product-specific tables should live in the PriceTracker domain

2. **Store one fact in one place**
   - multiple emails = child table, not repeated columns
   - password history = child table, not JSON on account
   - multiple attached files = attachment table, not `file_1`, `file_2`, `file_3`

3. **Model credentials by type**
   - password, passkey, TOTP, and recovery codes do not belong in one overloaded row

4. **Treat identifiers as scoped, not automatically global**
   - barcode/SKU values may be global, retailer-specific, or shop-specific

5. **Privacy by design**
   - collect the minimum PII needed
   - support retention, deletion, consent, cookie preferences, and access/export workflows at the schema level

6. **Prefer append-only security/audit records**
   - auth history, consent history, and audit records should be hard to tamper with

---

## 5. Target schema split

### 5.1 Reusable platform template domain

- `account`
- `account_profile`
- `account_email`
- `account_phone`
- `external_identity`
- `authenticator`
- `password_credential`
- `password_history`
- `passkey_credential`
- `totp_factor`
- `recovery_code_set`
- `recovery_code`
- `session`
- `role`
- `permission`
- `role_permission`
- `account_role`
- `setting_definition`
- `account_setting`
- `system_setting`
- `storage_object`
- `file_asset`
- `file_derivative`
- `file_scan_result`
- `file_attachment`
- `privacy_notice_version`
- `processing_purpose`
- `consent_record`
- `cookie_definition`
- `cookie_consent`
- `retention_policy`
- `data_subject_request`
- `legal_hold`
- `notification`
- `audit_log`
- `security_event`
- `account_suspension`

### 5.2 PriceTracker domain

- `category`
- `brand`
- `unit_family`
- `unit`
- `currency`
- `address`
- `item`
- `item_variant`
- `variant_identifier`
- `retailer`
- `shop`
- `shop_listing`
- `discount_type`
- `data_source`
- `price_observation`
- `purchase`
- `purchase_line`
- `watchlist_item`
- `watchlist_shop`
- `price_alert`
- `price_vote`
- `price_report`
- `moderation_decision`

---

## 6. Reusable platform template

## 6.1 Account and profile

### `account`
Top-level account identity shared by all projects.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| public_handle | VARCHAR(80) | Yes |  | optional public username/handle |
| account_status | VARCHAR(30) | No |  | pending/active/suspended/deleted |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |
| deleted_at | TIMESTAMPTZ | Yes |  | soft delete |
| last_active_at | TIMESTAMPTZ | Yes |  | |

**Constraints**
- `UNIQUE(public_handle)` where `public_handle` is not null

**Notes**
- Keep `account` lean.
- Do not place password hashes, phone numbers, or multiple email columns here.

---

### `account_profile`
Non-auth profile information.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| account_id | UUID | No | PK/FK -> account.id | one-to-one |
| display_name | VARCHAR(160) | Yes |  | |
| locale | VARCHAR(20) | Yes |  | e.g. en-GB |
| timezone_name | VARCHAR(80) | Yes |  | |
| preferred_currency_code | CHAR(3) | Yes | FK -> currency.code | reusable in commerce apps |
| profile_bio | TEXT | Yes |  | optional |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Notes**
- Avoid collecting DOB, gender, national IDs, or other high-risk personal data unless there is a real business requirement.
- Avatar/profile photos should be linked using `file_attachment` with role `AVATAR`.

---

## 6.2 Contact points

### `account_email`
Supports multiple emails per account, including primary, secondary, backup, and recovery emails.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| email | VARCHAR(320) | No |  | original form |
| normalized_email | VARCHAR(320) | No |  | lowercase/normalized for lookup |
| email_role | VARCHAR(30) | No |  | PRIMARY, SECONDARY, BACKUP, RECOVERY |
| is_login_enabled | BOOLEAN | No |  | whether this email may be used to sign in |
| is_primary_for_account | BOOLEAN | No |  | exactly one active primary per account |
| verified_at | TIMESTAMPTZ | Yes |  | |
| verification_method | VARCHAR(30) | Yes |  | link, otp, admin |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |
| deleted_at | TIMESTAMPTZ | Yes |  | soft delete |

**Constraints**
- `UNIQUE(account_id, normalized_email)` where `deleted_at` is null
- add a partial unique index for one active primary email per account
- optionally add a global unique index on `normalized_email` if the product should prevent email sharing across accounts

---

### `account_phone`
Supports multiple phone numbers per account.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| e164_phone_number | VARCHAR(20) | No |  | normalized E.164 |
| extension | VARCHAR(20) | Yes |  | |
| phone_role | VARCHAR(30) | No |  | PRIMARY, SECONDARY, BACKUP, RECOVERY |
| is_sms_enabled | BOOLEAN | No |  | |
| is_voice_enabled | BOOLEAN | No |  | |
| is_primary_for_account | BOOLEAN | No |  | exactly one active primary per account if desired |
| verified_at | TIMESTAMPTZ | Yes |  | |
| verification_method | VARCHAR(30) | Yes |  | sms_otp, voice_call, admin |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |
| deleted_at | TIMESTAMPTZ | Yes |  | soft delete |

**Constraints**
- `UNIQUE(account_id, e164_phone_number)` where `deleted_at` is null
- add a partial unique index for one active primary phone per account if required

**Notes**
- Do not assume phone numbers are globally unique forever.
- Numbers are recycled by carriers, so verification timestamps matter.

---

## 6.3 Authentication and credential tables

### `external_identity`
External login identity linked to the account.

Examples: Google, Apple, GitHub, Azure AD, enterprise SSO.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| provider_code | VARCHAR(40) | No |  | google/apple/github/oidc/saml |
| provider_subject | VARCHAR(320) | No |  | stable provider-side subject/identifier |
| provider_email | VARCHAR(320) | Yes |  | email claim from provider |
| linked_at | TIMESTAMPTZ | No |  | |
| last_login_at | TIMESTAMPTZ | Yes |  | |
| is_active | BOOLEAN | No |  | |
| raw_claims_json | JSONB | Yes |  | optional limited claim snapshot |

**Constraints**
- `UNIQUE(provider_code, provider_subject)`

---

### `authenticator`
Common parent row for authenticators.

This keeps password, passkey, and TOTP enrollment lifecycle in one reusable pattern.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| authenticator_type | VARCHAR(30) | No |  | PASSWORD, PASSKEY, TOTP |
| usage_type | VARCHAR(30) | No |  | PRIMARY, MFA |
| display_label | VARCHAR(120) | Yes |  | e.g. "MacBook passkey", "Authenticator app" |
| status | VARCHAR(30) | No |  | pending/active/revoked/lost |
| enrolled_at | TIMESTAMPTZ | No |  | |
| confirmed_at | TIMESTAMPTZ | Yes |  | |
| last_used_at | TIMESTAMPTZ | Yes |  | |
| revoked_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- add a partial unique index if only one active password authenticator is allowed per account

---

### `password_credential`
Current password verifier for the account.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| authenticator_id | UUID | No | PK/FK -> authenticator.id | must point to a PASSWORD authenticator |
| password_hash | TEXT | No |  | store only hash |
| salt_value | BYTEA | No |  | explicit random salt per password |
| hash_algorithm | VARCHAR(40) | No |  | e.g. ARGON2ID |
| hash_parameters_json | JSONB | No |  | memory cost, iterations, lanes, version |
| password_version | INTEGER | No |  | increments on change |
| changed_at | TIMESTAMPTZ | No |  | |
| must_rotate | BOOLEAN | No |  | force reset after incident/breach |
| compromised_at | TIMESTAMPTZ | Yes |  | if compromise is detected |

**Constraints**
- `CHECK(password_version > 0)`

**Notes**
- Salt is intentionally explicit because you requested it.
- In some implementations the salt is also embedded inside the encoded hash string; the separate column still keeps the schema clear and portable.

---

### `password_history`
Historical password verifiers used to prevent password reuse.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| password_hash | TEXT | No |  | previous hash only |
| salt_value | BYTEA | No |  | |
| hash_algorithm | VARCHAR(40) | No |  | |
| hash_parameters_json | JSONB | No |  | |
| password_version | INTEGER | No |  | version being retired |
| valid_from | TIMESTAMPTZ | No |  | |
| valid_to | TIMESTAMPTZ | No |  | |
| stored_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(account_id, password_version)`

**Notes**
- On password change, copy the current password verifier into this table before replacing it.
- Reuse prevention should compare a candidate password against the most recent N historical hashes according to policy.
- Never store raw or reversibly encrypted old passwords.

---

### `passkey_credential`
WebAuthn/FIDO passkey credential.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| authenticator_id | UUID | No | PK/FK -> authenticator.id | must point to a PASSKEY authenticator |
| rp_id | VARCHAR(255) | No |  | relying party ID |
| webauthn_user_handle | BYTEA | No |  | stable user handle |
| credential_id | BYTEA | No |  | unique credential identifier |
| public_key_cose | BYTEA | No |  | stored public key |
| aaguid | UUID | Yes |  | authenticator model identifier |
| sign_count | BIGINT | Yes |  | WebAuthn signature counter |
| transports_json | JSONB | Yes |  | internal, usb, nfc, ble, hybrid |
| attestation_format | VARCHAR(80) | Yes |  | |
| credential_device_type | VARCHAR(30) | Yes |  | single_device or multi_device/syncable |
| is_backup_eligible | BOOLEAN | Yes |  | |
| is_backed_up | BOOLEAN | Yes |  | |
| user_verification_policy | VARCHAR(30) | Yes |  | preferred/required/discouraged |

**Constraints**
- `UNIQUE(rp_id, credential_id)`

**Notes**
- Store only the public key and WebAuthn metadata.
- Passkeys can be used as a primary authenticator and may satisfy MFA requirements depending on policy.

---

### `totp_factor`
TOTP-based second factor.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| authenticator_id | UUID | No | PK/FK -> authenticator.id | must point to a TOTP authenticator |
| secret_ciphertext | BYTEA | No |  | encrypted at rest, not hashed |
| key_reference | VARCHAR(120) | Yes |  | KMS/HSM key reference |
| otp_algorithm | VARCHAR(20) | No |  | SHA1/SHA256/SHA512 |
| digits | SMALLINT | No |  | usually 6 or 8 |
| period_seconds | SMALLINT | No |  | usually 30 |
| issuer_label | VARCHAR(120) | Yes |  | shown in authenticator app |
| confirmed_at | TIMESTAMPTZ | Yes |  | successful verification after enrollment |

**Constraints**
- `CHECK(digits IN (6, 8))`
- `CHECK(period_seconds > 0)`

**Notes**
- TOTP secrets must be encrypted because the server needs them for verification.
- Do not store them in plain text.

---

### `recovery_code_set`
A batch/set of recovery codes issued to an account.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| code_count | SMALLINT | No |  | |
| status | VARCHAR(30) | No |  | active/replaced/revoked/exhausted |
| issued_at | TIMESTAMPTZ | No |  | |
| replaced_by_set_id | UUID | Yes | FK -> recovery_code_set.id | |
| revoked_at | TIMESTAMPTZ | Yes |  | |

**Constraints**
- `CHECK(code_count > 0)`

---

### `recovery_code`
Individual recovery code, stored hashed.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| recovery_code_set_id | UUID | No | FK -> recovery_code_set.id | |
| sequence_number | SMALLINT | No |  | display order only |
| code_hash | TEXT | No |  | hashed, not raw |
| salt_value | BYTEA | No |  | |
| hash_algorithm | VARCHAR(40) | No |  | |
| used_at | TIMESTAMPTZ | Yes |  | one-time use |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(recovery_code_set_id, sequence_number)`

**Notes**
- Recovery codes should be treated like one-time passwords with hashing and auditability.
- When a new set is issued, the previous active set should be revoked or replaced.

---

### `session`
Server-side session / refresh-token session / device session.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| session_token_hash | TEXT | Yes |  | if a session cookie identifier is used |
| refresh_token_hash | TEXT | Yes |  | if refresh-token auth is used |
| authenticated_aal | SMALLINT | No |  | 1, 2, or 3 style internal level |
| remember_me | BOOLEAN | No |  | |
| user_agent | TEXT | Yes |  | |
| ip_address | INET | Yes |  | |
| device_label | VARCHAR(120) | Yes |  | optional user-facing label |
| created_at | TIMESTAMPTZ | No |  | |
| last_seen_at | TIMESTAMPTZ | Yes |  | |
| expires_at | TIMESTAMPTZ | No |  | |
| revoked_at | TIMESTAMPTZ | Yes |  | |

**Notes**
- Store token hashes only, never raw refresh/session tokens.
- Session cookies and cookie consent are different concerns and must not be merged in the schema.

---

## 6.4 Authorization and settings

### `role`
Reusable application role.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| code | VARCHAR(40) | No |  | USER, MODERATOR, ADMIN |
| name | VARCHAR(80) | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(code)`

---

### `permission`
Reusable permission catalogue.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| code | VARCHAR(80) | No |  | PRICE_EDIT_ANY, ACCOUNT_SUSPEND, FILE_DELETE |
| name | VARCHAR(120) | No |  | |
| description | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(code)`

---

### `role_permission`
Role-permission mapping.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| role_id | UUID | No | PK/FK -> role.id | |
| permission_id | UUID | No | PK/FK -> permission.id | |
| granted_at | TIMESTAMPTZ | No |  | |

---

### `account_role`
Account-role mapping.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| account_id | UUID | No | PK/FK -> account.id | |
| role_id | UUID | No | PK/FK -> role.id | |
| granted_by_account_id | UUID | Yes | FK -> account.id | |
| granted_at | TIMESTAMPTZ | No |  | |

---

### `setting_definition`
Defines reusable settings.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| setting_key | VARCHAR(120) | No |  | |
| scope_type | VARCHAR(20) | No |  | ACCOUNT or SYSTEM |
| value_type | VARCHAR(20) | No |  | STRING, NUMBER, BOOLEAN, JSON |
| default_value_json | JSONB | Yes |  | |
| is_sensitive | BOOLEAN | No |  | avoid exposing in normal APIs |
| description | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(scope_type, setting_key)`

---

### `account_setting`
Per-account settings value.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| account_id | UUID | No | PK/FK -> account.id | composite PK part |
| setting_definition_id | UUID | No | PK/FK -> setting_definition.id | |
| setting_value_json | JSONB | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

### `system_setting`
System-wide settings value.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| setting_definition_id | UUID | No | PK/FK -> setting_definition.id | |
| setting_value_json | JSONB | No |  | |
| updated_by_account_id | UUID | Yes | FK -> account.id | |
| updated_at | TIMESTAMPTZ | No |  | |

---

## 6.5 Reusable file system

### Why the file system changes

The old design only supported a single `FileID` on `PurchaseRecord`. That is not enough for:

- two or more receipt photos for one purchase
- thumbnail / redacted / OCR derivatives
- avatar reuse
- moderation evidence
- future document uploads

The new file system separates:

1. **physical stored blob** (`storage_object`)
2. **logical file metadata** (`file_asset`)
3. **attachment to one or more business entities** (`file_attachment`)

---

### `storage_object`
Represents the physical stored blob/object in storage.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| storage_provider | VARCHAR(30) | No |  | s3, gcs, azure_blob, local |
| bucket_name | VARCHAR(120) | No |  | |
| object_key | TEXT | No |  | object path/key |
| checksum_sha256 | CHAR(64) | No |  | dedupe/integrity |
| size_bytes | BIGINT | No |  | |
| encryption_key_ref | VARCHAR(120) | Yes |  | KMS/HSM reference if used |
| created_at | TIMESTAMPTZ | No |  | |
| deleted_at | TIMESTAMPTZ | Yes |  | logical removal |

**Constraints**
- `UNIQUE(storage_provider, bucket_name, object_key)`
- `CHECK(size_bytes >= 0)`

---

### `file_asset`
Logical file record and metadata.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| storage_object_id | UUID | No | FK -> storage_object.id | |
| owner_account_id | UUID | Yes | FK -> account.id | |
| original_filename | VARCHAR(255) | No |  | |
| mime_type | VARCHAR(120) | No |  | |
| file_extension | VARCHAR(20) | Yes |  | |
| purpose_code | VARCHAR(40) | No |  | RECEIPT_PHOTO, AVATAR, IMPORT, EVIDENCE |
| classification_code | VARCHAR(40) | No |  | PUBLIC, INTERNAL, CONFIDENTIAL, SENSITIVE_PII |
| is_deleted | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `file_derivative`
Derived file generated from another file.

Examples: thumbnail, OCR text package, redacted copy, normalized image.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| parent_file_asset_id | UUID | No | FK -> file_asset.id | |
| derivative_type | VARCHAR(40) | No |  | THUMBNAIL, REDACTED, OCR_EXPORT |
| storage_object_id | UUID | No | FK -> storage_object.id | |
| created_by_process | VARCHAR(80) | Yes |  | OCR_PIPELINE, IMAGE_RESIZER |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(parent_file_asset_id, derivative_type)`

---

### `file_scan_result`
Security/content scan records for uploaded files.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| file_asset_id | UUID | No | FK -> file_asset.id | |
| scan_type | VARCHAR(40) | No |  | MALWARE, MIME_VALIDATION, PII_DETECTION |
| scan_status | VARCHAR(30) | No |  | pending/passed/failed/quarantined |
| scanner_name | VARCHAR(80) | Yes |  | |
| result_json | JSONB | Yes |  | |
| scanned_at | TIMESTAMPTZ | No |  | |

---

### `file_attachment`
Generic many-to-many attachment table.

This is what allows **many files for one entry**.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| file_asset_id | UUID | No | FK -> file_asset.id | |
| entity_type | VARCHAR(60) | No |  | purchase, price_observation, account_profile, item_variant |
| entity_id | UUID | No |  | attached row id |
| attachment_role | VARCHAR(40) | No |  | RECEIPT_PHOTO, AVATAR, GALLERY, EVIDENCE |
| sort_order | INTEGER | No |  | display order |
| is_primary | BOOLEAN | No |  | |
| attached_by_account_id | UUID | Yes | FK -> account.id | |
| metadata_json | JSONB | Yes |  | optional crop/orientation/client info |
| created_at | TIMESTAMPTZ | No |  | |
| removed_at | TIMESTAMPTZ | Yes |  | soft detach |

**Constraints**
- `CHECK(sort_order >= 0)`

**Notes**
- Example: one shopping with two receipt photos = two rows pointing to the same `purchase.id`.
- This generic model is highly reusable across projects.
- Trade-off: generic `entity_type/entity_id` links are flexible but cannot enforce strict DB-level foreign keys to every possible target table.
- If strict FK enforcement is required for a specific domain, add dedicated join tables on top of this template, for example:
  - `purchase_file`
  - `item_variant_file`
  - `account_avatar`

---

## 6.6 Privacy, data collection, retention, and cookies

### `privacy_notice_version`
Versioned privacy/cookie/terms notices.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| notice_kind | VARCHAR(30) | No |  | PRIVACY, COOKIE, TERMS |
| version_label | VARCHAR(40) | No |  | e.g. 2026-04 |
| locale | VARCHAR(20) | No |  | |
| content_hash | CHAR(64) | No |  | proof of exact text version |
| published_at | TIMESTAMPTZ | No |  | |
| retired_at | TIMESTAMPTZ | Yes |  | |

**Constraints**
- `UNIQUE(notice_kind, version_label, locale)`

---

### `retention_policy`
Retention rule catalog.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| entity_type | VARCHAR(60) | No |  | account_email, session, file_asset, purchase |
| trigger_event | VARCHAR(40) | No |  | created_at, last_active_at, deleted_at, used_at |
| retain_days | INTEGER | No |  | |
| archive_after_days | INTEGER | Yes |  | optional |
| delete_after_days | INTEGER | Yes |  | optional hard-delete timeline |
| legal_basis_note | TEXT | Yes |  | contract, legal obligation, legitimate interest, etc. |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(retain_days >= 0)`

---

### `processing_purpose`
Purpose register for personal data processing.

This is more general than a pure "consent purpose" table because not all processing relies on consent.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| code | VARCHAR(60) | No |  | ACCOUNT_SECURITY, PRICE_ALERTS, ANALYTICS |
| name | VARCHAR(120) | No |  | |
| lawful_basis | VARCHAR(40) | No |  | consent/contract/legal_obligation/legitimate_interest |
| consent_required | BOOLEAN | No |  | true when consent is the chosen basis |
| retention_policy_id | UUID | Yes | FK -> retention_policy.id | |
| description | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(code)`

---

### `consent_record`
Time-versioned consent event log.

Works for logged-in users and guests.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | Yes | FK -> account.id | nullable for guest consent |
| anonymous_subject_token_hash | CHAR(64) | Yes |  | hashed anonymous consent token/device reference |
| processing_purpose_id | UUID | No | FK -> processing_purpose.id | |
| notice_version_id | UUID | Yes | FK -> privacy_notice_version.id | |
| consent_status | VARCHAR(30) | No |  | granted/withdrawn/denied |
| captured_via | VARCHAR(30) | No |  | web, mobile, api, support |
| evidence_json | JSONB | Yes |  | banner version, IP prefix, UX action |
| captured_at | TIMESTAMPTZ | No |  | |
| withdrawn_at | TIMESTAMPTZ | Yes |  | |

**Notes**
- This table should be append-only in practice.
- For core security processing such as login sessions, do not rely on consent if the lawful basis is not consent.

---

### `cookie_definition`
Cookie or similar technology catalogue.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| cookie_name | VARCHAR(120) | No |  | |
| provider_name | VARCHAR(120) | Yes |  | first or third party provider |
| cookie_category | VARCHAR(30) | No |  | ESSENTIAL, PREFERENCES, ANALYTICS, MARKETING |
| is_strictly_necessary | BOOLEAN | No |  | |
| duration_seconds | BIGINT | Yes |  | optional TTL |
| description | TEXT | Yes |  | what it does |
| created_at | TIMESTAMPTZ | No |  | |
| retired_at | TIMESTAMPTZ | Yes |  | |

**Constraints**
- `UNIQUE(cookie_name, provider_name)`

---

### `cookie_consent`
Current cookie preference snapshot per anonymous subject or account.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | Yes | FK -> account.id | nullable for guest |
| anonymous_subject_token_hash | CHAR(64) | Yes |  | guest/device/browser level |
| notice_version_id | UUID | Yes | FK -> privacy_notice_version.id | cookie notice used |
| preferences_allowed | BOOLEAN | No |  | |
| analytics_allowed | BOOLEAN | No |  | |
| marketing_allowed | BOOLEAN | No |  | |
| captured_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |
| withdrawn_at | TIMESTAMPTZ | Yes |  | |

**Notes**
- Essential/security cookies are normally handled outside opt-in categories when legally exempt.
- This table records the user's effective preference state; detailed history lives in `consent_record`.

---

### `data_subject_request`
Tracks privacy rights requests.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | Yes | FK -> account.id | nullable if requester is not yet mapped |
| request_type | VARCHAR(30) | No |  | ACCESS, ERASURE, RECTIFICATION, PORTABILITY, OBJECTION |
| status | VARCHAR(30) | No |  | open/in_review/completed/rejected |
| verified_by_account_id | UUID | Yes | FK -> account.id | support/admin verifier |
| export_file_asset_id | UUID | Yes | FK -> file_asset.id | optional export package |
| requested_at | TIMESTAMPTZ | No |  | |
| due_at | TIMESTAMPTZ | Yes |  | |
| completed_at | TIMESTAMPTZ | Yes |  | |
| notes | TEXT | Yes |  | |

---

### `legal_hold`
Overrides retention when deletion must be paused.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| entity_type | VARCHAR(60) | No |  | |
| entity_id | UUID | No |  | |
| reason | TEXT | No |  | |
| placed_by_account_id | UUID | Yes | FK -> account.id | |
| placed_at | TIMESTAMPTZ | No |  | |
| released_at | TIMESTAMPTZ | Yes |  | |

---

## 6.7 Notifications, audit, and security

### `notification`
Reusable user-facing notification record.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| notification_type | VARCHAR(40) | No |  | price_alert, moderation, security |
| payload_json | JSONB | No |  | |
| read_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `audit_log`
Immutable audit log.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| actor_account_id | UUID | Yes | FK -> account.id | nullable for system events |
| action_code | VARCHAR(80) | No |  | |
| entity_type | VARCHAR(60) | No |  | |
| entity_id | UUID | Yes |  | |
| request_id | UUID | Yes |  | request tracing |
| old_value_json | JSONB | Yes |  | |
| new_value_json | JSONB | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `security_event`
Authentication and abuse-related event log.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | Yes | FK -> account.id | |
| session_id | UUID | Yes | FK -> session.id | |
| ip_address | INET | Yes |  | |
| severity | VARCHAR(20) | No |  | info/warn/high/critical |
| event_type | VARCHAR(60) | No |  | LOGIN_FAILED, PASSKEY_ADDED, TOKEN_REVOKED |
| details_json | JSONB | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `account_suspension`
Generic moderation/abuse suspension table.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| reason | TEXT | No |  | |
| starts_at | TIMESTAMPTZ | No |  | |
| ends_at | TIMESTAMPTZ | Yes |  | null for indefinite |
| created_by_account_id | UUID | No | FK -> account.id | moderator/admin |
| created_at | TIMESTAMPTZ | No |  | |

---

## 7. PriceTracker business schema

## 7.1 Reference and catalog tables

### `category`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| parent_category_id | UUID | Yes | FK -> category.id | adjacency-list hierarchy |
| name | VARCHAR(120) | No |  | |
| normalized_name | VARCHAR(120) | No |  | |
| description | TEXT | Yes |  | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(parent_category_id, normalized_name)`

---

### `brand`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| name | VARCHAR(160) | No |  | |
| normalized_name | VARCHAR(160) | No |  | |
| country_code | CHAR(2) | Yes |  | |
| website_url | TEXT | Yes |  | |
| headquarters_address_id | UUID | Yes | FK -> address.id | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(normalized_name)`

---

### `unit_family`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| code | VARCHAR(40) | No |  | MASS, VOLUME, COUNT |
| name | VARCHAR(80) | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(code)`

---

### `unit`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| unit_family_id | UUID | No | FK -> unit_family.id | |
| code | VARCHAR(40) | No |  | g, kg, ml, l, each |
| name | VARCHAR(80) | No |  | |
| symbol | VARCHAR(20) | No |  | |
| factor_to_base | NUMERIC(20,8) | No |  | |
| is_base_unit | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(unit_family_id, code)`
- `CHECK(factor_to_base > 0)`

---

### `currency`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| code | CHAR(3) | No | PK | ISO 4217 |
| name | VARCHAR(80) | No |  | |
| symbol | VARCHAR(10) | Yes |  | |
| minor_unit | SMALLINT | No |  | |
| is_active | BOOLEAN | No |  | |

---

### `address`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| country_code | CHAR(2) | No |  | |
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
| full_text | TEXT | Yes |  | |
| latitude | NUMERIC(10,7) | Yes |  | |
| longitude | NUMERIC(10,7) | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

### `item`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| category_id | UUID | Yes | FK -> category.id | |
| canonical_name | VARCHAR(200) | No |  | |
| normalized_name | VARCHAR(200) | No |  | |
| specification_text | TEXT | Yes |  | not package size |
| description | TEXT | Yes |  | |
| status | VARCHAR(30) | No |  | draft/pending/approved/rejected/archived |
| created_by_account_id | UUID | Yes | FK -> account.id | |
| approved_by_account_id | UUID | Yes | FK -> account.id | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

### `item_variant`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| item_id | UUID | No | FK -> item.id | |
| brand_id | UUID | Yes | FK -> brand.id | |
| variant_name | VARCHAR(160) | Yes |  | |
| package_quantity | NUMERIC(18,6) | No |  | |
| package_unit_id | UUID | No | FK -> unit.id | |
| pack_count | INTEGER | No |  | default 1 |
| normalized_content_quantity | NUMERIC(18,6) | Yes |  | cache/derived |
| normalized_content_unit_id | UUID | Yes | FK -> unit.id | |
| status | VARCHAR(30) | No |  | |
| created_by_account_id | UUID | Yes | FK -> account.id | |
| approved_by_account_id | UUID | Yes | FK -> account.id | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(package_quantity > 0)`
- `CHECK(pack_count > 0)`

---

## 7.2 Identifier, retailer, and listing tables

### `variant_identifier`
Scoped product identifier table.

This replaces the old `item_identifier` concept and avoids assuming that a barcode is always a universal international identifier.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| identifier_type | VARCHAR(40) | No |  | BARCODE, SKU, GTIN, UPC, EAN, PLU, INTERNAL |
| scope_type | VARCHAR(20) | No |  | GLOBAL, RETAILER, SHOP, BRAND, INTERNAL |
| retailer_id | UUID | Yes | FK -> retailer.id | required for RETAILER scope |
| shop_id | UUID | Yes | FK -> shop.id | required for SHOP scope |
| identifier_value | VARCHAR(200) | No |  | |
| is_primary | BOOLEAN | No |  | |
| valid_from | TIMESTAMPTZ | Yes |  | |
| valid_to | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- for `scope_type = 'SHOP'`, `shop_id` must be present
- for `scope_type = 'RETAILER'`, `retailer_id` must be present
- use partial unique indexes by scope, for example:
  - `(identifier_type, identifier_value)` for GLOBAL identifiers
  - `(retailer_id, identifier_type, identifier_value)` for RETAILER identifiers
  - `(shop_id, identifier_type, identifier_value)` for SHOP identifiers

**Examples**
- true GTIN/EAN known globally -> `scope_type = GLOBAL`
- shop’s own barcode sticker -> `scope_type = SHOP`
- retailer-specific online SKU -> usually better stored in `shop_listing.external_sku`

---

### `retailer`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| name | VARCHAR(160) | No |  | |
| normalized_name | VARCHAR(160) | No |  | |
| retailer_type | VARCHAR(30) | No |  | supermarket/marketplace/local_store |
| website_url | TEXT | Yes |  | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(normalized_name)`

---

### `shop`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| retailer_id | UUID | No | FK -> retailer.id | |
| name | VARCHAR(200) | No |  | |
| address_id | UUID | Yes | FK -> address.id | nullable for online-only shops |
| phone_number | VARCHAR(50) | Yes |  | public shop phone |
| is_online | BOOLEAN | No |  | |
| latitude | NUMERIC(10,7) | Yes |  | |
| longitude | NUMERIC(10,7) | Yes |  | |
| timezone_name | VARCHAR(80) | Yes |  | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

### `shop_listing`
Listing between a `shop` and an `item_variant`.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| shop_id | UUID | No | FK -> shop.id | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| external_sku | VARCHAR(200) | Yes |  | shop-specific |
| listing_url | TEXT | Yes |  | |
| first_seen_at | TIMESTAMPTZ | Yes |  | |
| last_seen_at | TIMESTAMPTZ | Yes |  | |
| is_active | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(shop_id, item_variant_id)`
- `UNIQUE(shop_id, external_sku)` where `external_sku` is not null

---

### `discount_type`

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

## 7.3 Price and purchase tables

### `data_source`
Who or what created a record.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| source_type | VARCHAR(30) | No |  | ACCOUNT, ADMIN, IMPORT, CRAWLER, API |
| account_id | UUID | Yes | FK -> account.id | nullable for non-user sources |
| source_name | VARCHAR(160) | Yes |  | import name / crawler job name |
| trust_score | NUMERIC(5,2) | Yes |  | |
| is_verified | BOOLEAN | No |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `price_observation`
Canonical observed price fact.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| shop_id | UUID | No | FK -> shop.id | |
| shop_listing_id | UUID | Yes | FK -> shop_listing.id | optional listing source |
| source_id | UUID | No | FK -> data_source.id | |
| observed_at | TIMESTAMPTZ | No |  | |
| currency_code | CHAR(3) | No | FK -> currency.code | |
| list_price_amount | NUMERIC(14,4) | Yes |  | pre-discount |
| final_price_amount | NUMERIC(14,4) | No |  | actual displayed/paid amount |
| discount_amount | NUMERIC(14,4) | Yes |  | |
| discount_type_id | UUID | Yes | FK -> discount_type.id | |
| unit_price_amount | NUMERIC(14,6) | Yes |  | cache |
| unit_price_unit_id | UUID | Yes | FK -> unit.id | |
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
Receipt/purchase header.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| purchaser_account_id | UUID | Yes | FK -> account.id | |
| shop_id | UUID | No | FK -> shop.id | |
| purchased_at | TIMESTAMPTZ | No |  | |
| currency_code | CHAR(3) | No | FK -> currency.code | |
| receipt_number | VARCHAR(200) | Yes |  | |
| seller_tax_identifier | VARCHAR(200) | Yes |  | |
| total_amount | NUMERIC(14,4) | Yes |  | |
| tax_amount | NUMERIC(14,4) | Yes |  | |
| notes | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

**Important**
- Receipt photos are **not** stored as a single FK on this table anymore.
- Attach one or many receipt files using `file_attachment`:
  - `entity_type = 'purchase'`
  - `entity_id = purchase.id`
  - `attachment_role = 'RECEIPT_PHOTO'`

---

### `purchase_line`
Line-level receipt detail.

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| purchase_id | UUID | No | FK -> purchase.id | |
| price_observation_id | UUID | No | FK -> price_observation.id | unique if each receipt-backed observation is distinct |
| line_number | INTEGER | Yes |  | |
| quantity_purchased | NUMERIC(18,6) | No |  | packs bought |
| batch_code | VARCHAR(200) | Yes |  | |
| serial_number | VARCHAR(200) | Yes |  | |
| vat_rate | NUMERIC(6,3) | Yes |  | |
| notes | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(quantity_purchased > 0)`
- `UNIQUE(price_observation_id)`

---

## 7.4 Personalization and community

### `watchlist_item`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(account_id, item_variant_id)`

---

### `watchlist_shop`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| shop_id | UUID | No | FK -> shop.id | |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `UNIQUE(account_id, shop_id)`

---

### `price_alert`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| account_id | UUID | No | FK -> account.id | |
| item_variant_id | UUID | No | FK -> item_variant.id | |
| shop_id | UUID | Yes | FK -> shop.id | nullable for any shop |
| target_price_amount | NUMERIC(14,4) | No |  | |
| currency_code | CHAR(3) | No | FK -> currency.code | |
| is_active | BOOLEAN | No |  | |
| last_triggered_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |
| updated_at | TIMESTAMPTZ | No |  | |

---

### `price_vote`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| price_observation_id | UUID | No | FK -> price_observation.id | |
| account_id | UUID | No | FK -> account.id | |
| vote_value | SMALLINT | No |  | 1 or -1 |
| created_at | TIMESTAMPTZ | No |  | |

**Constraints**
- `CHECK(vote_value IN (-1, 1))`
- `UNIQUE(price_observation_id, account_id)`

---

### `price_report`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| price_observation_id | UUID | No | FK -> price_observation.id | |
| reported_by_account_id | UUID | No | FK -> account.id | |
| reason_code | VARCHAR(50) | No |  | DUPLICATE, WRONG_ITEM, WRONG_PRICE, SPAM |
| details | TEXT | Yes |  | |
| status | VARCHAR(30) | No |  | open/reviewed/resolved/rejected |
| resolved_by_account_id | UUID | Yes | FK -> account.id | |
| resolved_at | TIMESTAMPTZ | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

### `moderation_decision`

| Column | Type | Null | Key | Notes |
|---|---|---:|---|---|
| id | UUID | No | PK | |
| entity_type | VARCHAR(40) | No |  | item/item_variant/brand/shop/price_observation |
| entity_id | UUID | No |  | |
| decision | VARCHAR(30) | No |  | approved/rejected/merged/superseded |
| moderator_account_id | UUID | No | FK -> account.id | |
| notes | TEXT | Yes |  | |
| created_at | TIMESTAMPTZ | No |  | |

---

## 8. Why this design is more normalized

### 8.1 Multiple emails and phones
Using `account_email` and `account_phone` avoids repeating columns like:

- `primary_email`
- `secondary_email`
- `backup_email`
- `phone_1`
- `phone_2`

This is cleaner 1:N modeling and supports verification state per contact point.

### 8.2 Password history
Password history is a repeating attribute, so it belongs in its own table.
`password_history` keeps reuse prevention separate from the current active verifier.

### 8.3 Passkeys and TOTP
Passkeys, passwords, and TOTP secrets do not share the same attributes.
Subtype tables under `authenticator` avoid sparse nullable columns and keep each credential model correct.

### 8.4 Many files per record
`file_attachment` removes the anti-pattern of fixed file columns.
A purchase can now have zero, one, or many receipt photos without schema changes.

### 8.5 Scoped identifiers
A barcode is not automatically a universal identifier.
`variant_identifier` normalizes this by storing scope explicitly and avoids false uniqueness assumptions.

### 8.6 Cookie consent vs session auth
A session cookie and a cookie-preference decision are not the same fact.
Separating `session`, `cookie_consent`, and `consent_record` avoids mixing operational auth data with privacy preference data.

---

## 9. Recommended indexes

### Account and auth
- `account(public_handle)`
- `account_email(normalized_email)` with active partial index
- `account_phone(e164_phone_number)` with active partial index
- `external_identity(provider_code, provider_subject)`
- `authenticator(account_id, authenticator_type, status)`
- `session(account_id, expires_at DESC)`
- `security_event(account_id, created_at DESC)`
- `security_event(ip_address, created_at DESC)`

### File system
- `file_asset(owner_account_id, created_at DESC)`
- `file_attachment(entity_type, entity_id, created_at DESC)`
- `file_attachment(file_asset_id)`
- `file_scan_result(file_asset_id, scanned_at DESC)`

### Privacy/compliance
- `consent_record(account_id, processing_purpose_id, captured_at DESC)`
- `cookie_consent(account_id, updated_at DESC)`
- `data_subject_request(account_id, requested_at DESC)`
- `retention_policy(entity_type)`

### Catalog and price
- `item(normalized_name)`
- `item_variant(item_id)`
- `variant_identifier(item_variant_id)`
- partial unique indexes on `variant_identifier` by scope
- `retailer(normalized_name)`
- `shop(retailer_id)`
- `shop_listing(shop_id, item_variant_id)`
- `price_observation(item_variant_id, observed_at DESC)`
- `price_observation(shop_id, observed_at DESC)`
- `price_observation(item_variant_id, shop_id, observed_at DESC)`
- `purchase(shop_id, purchased_at DESC)`
- `purchase_line(purchase_id)`

---

## 10. Privacy and security implementation notes

### 10.1 Password policy
Recommended application behavior:

- use a modern password hashing algorithm such as Argon2id
- generate a **unique random salt per password**
- keep password history rows to prevent reuse of the last N passwords
- compare new password candidates against history during password change
- never store password hints
- never store raw recovery codes after display

### 10.2 Passkeys
Recommended application behavior:

- support passkey registration after normal sign-in or during secure onboarding
- allow multiple passkeys per account
- keep passkeys revocable individually
- treat passkeys as preferred phishing-resistant authenticators

### 10.3 2FA / MFA
Recommended application behavior:

- allow TOTP enrollment as a baseline second factor
- allow passkeys to satisfy stronger auth requirements where policy permits
- keep one or more recovery methods available:
  - recovery codes
  - verified backup email(s)
  - verified backup phone number(s)
- notify the account on credential enrollment, recovery, reset, and suspicious events

### 10.4 File privacy
Receipt images often contain more data than the user intended to share.

Recommended controls:

- classify receipt uploads as at least `CONFIDENTIAL`, often `SENSITIVE_PII`
- run malware and type validation scans on upload
- create a redacted derivative when receipts are displayed back to other users or moderators
- avoid exposing raw storage keys in public APIs
- apply retention rules to uploads and derivatives

### 10.5 Data minimization
Do not collect these fields unless the product truly needs them:

- date of birth
- gender
- national ID numbers
- exact location history
- payment card details
- unnecessary device fingerprinting

### 10.6 Cookie and consent handling
Recommended split:

- strictly necessary operational cookies -> handled by auth/session/security flows
- optional analytics/marketing/preferences cookies -> governed through cookie banner/settings and stored in `cookie_consent` plus `consent_record`

### 10.7 Deletion and retention
Soft delete alone is not enough.

Recommended workflow:

1. mark record as deleted or inactive where needed
2. evaluate `retention_policy`
3. skip deletion if `legal_hold` exists
4. hard-delete or anonymize when policy allows
5. log the action in `audit_log`

---

## 11. Mapping from the current draft schema

| Current table/field | Revised destination | Notes |
|---|---|---|
| `Item` | `item` | keep concept, normalize names |
| `Category.FatherID` | `category.parent_category_id` | corrected hierarchy field |
| `Brand.LocarionID` | `brand.headquarters_address_id` | corrected typo and meaning |
| `Shop` | `retailer` + `shop` | split brand/chain from concrete store |
| `ItemVarient` | `item_variant` | corrected name |
| `ItemVarient.SKU` | `variant_identifier` or `shop_listing.external_sku` | depends on scope |
| `ItemVarient` barcode-ish field | `variant_identifier` | now scope-aware; can be shop-specific |
| `ItemVarient.Website` | `shop_listing.listing_url` | shop/listing-specific |
| `Unit.BaseUnitID` | `unit_family` + `unit.factor_to_base` | conversion becomes explicit |
| `PurchaseRecord` | `purchase` | use full timestamp |
| `PurchaseRecord.FileID` | `file_attachment` | supports multiple files |
| `PriceRecord` | `price_observation` + `purchase_line` | separated fact/header-line model |
| `PriceRecord.SourceID` | `data_source.account_id` / `data_source.id` | explicit source entity |
| old user module idea | `account` + contacts + auth tables | reusable platform template |
| old security module idea | `external_identity`, `authenticator`, `session`, `security_event` | reusable platform template |
| old file module idea | `storage_object`, `file_asset`, `file_attachment` | reusable platform template |

---

## 12. Minimum viable schema

If you want a phased rollout, build in this order.

### Phase 1: must-have platform tables
- `account`
- `account_profile`
- `account_email`
- `external_identity`
- `authenticator`
- `password_credential`
- `password_history`
- `session`
- `role`
- `account_role`
- `storage_object`
- `file_asset`
- `file_attachment`
- `security_event`
- `audit_log`

### Phase 2: security hardening
- `account_phone`
- `passkey_credential`
- `totp_factor`
- `recovery_code_set`
- `recovery_code`
- `permission`
- `role_permission`

### Phase 3: privacy/compliance
- `privacy_notice_version`
- `processing_purpose`
- `consent_record`
- `cookie_definition`
- `cookie_consent`
- `retention_policy`
- `data_subject_request`
- `legal_hold`

### Phase 4: PriceTracker business domain
- `category`
- `brand`
- `unit_family`
- `unit`
- `currency`
- `address`
- `item`
- `item_variant`
- `variant_identifier`
- `retailer`
- `shop`
- `shop_listing`
- `discount_type`
- `data_source`
- `price_observation`
- `purchase`
- `purchase_line`
- `watchlist_item`
- `watchlist_shop`
- `price_alert`
- `price_vote`
- `price_report`
- `moderation_decision`

---

## 13. Final recommendation

### Keep
- the split between item, variant, price, purchase, shop, unit, and address
- the idea of source tracking
- watchlists, alerts, moderation, and auditability

### Change immediately
- replace user-specific tables with the reusable platform account model
- replace single-email and single-phone assumptions with child tables
- add password history and explicit salt support
- add passkey, TOTP, and recovery-code support
- replace single-file receipt modeling with generic attachments
- replace `item_identifier` with `variant_identifier` using explicit scope
- separate session/auth cookies from privacy/cookie consent records

### Add next
- privacy notice versioning
- consent and cookie preference history
- retention and deletion workflow support
- data subject request tracking
- redacted receipt derivatives and file scanning

This revision gives you a schema that is closer to 3NF, more reusable across projects, and much more ready for a modern security and privacy baseline.