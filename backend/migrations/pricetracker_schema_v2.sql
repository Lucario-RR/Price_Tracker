-- PriceTracker Database Schema v2
-- Target: PostgreSQL
-- This script creates the schema described in price_tracker_database_doc_v2.md
-- Run against an existing database, for example:
--   psql -d pricetracker -f pricetracker_schema_v2.sql

BEGIN;

SET search_path TO public;

-- =========================================================
-- 1. Reusable platform template domain
-- =========================================================

CREATE TABLE account (
    id UUID NOT NULL,
    public_handle VARCHAR(80),
    account_status VARCHAR(30) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    last_active_at TIMESTAMPTZ,
    CONSTRAINT pk_account PRIMARY KEY (id),
    CONSTRAINT uq_account_public_handle UNIQUE (public_handle)
);

CREATE TABLE account_profile (
    account_id UUID NOT NULL,
    display_name VARCHAR(160),
    locale VARCHAR(20),
    timezone_name VARCHAR(80),
    preferred_currency_code CHAR(3),
    profile_bio TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_account_profile PRIMARY KEY (account_id)
);

CREATE TABLE account_email (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    email VARCHAR(320) NOT NULL,
    normalized_email VARCHAR(320) NOT NULL,
    email_role VARCHAR(30) NOT NULL,
    is_login_enabled BOOLEAN NOT NULL,
    is_primary_for_account BOOLEAN NOT NULL,
    verified_at TIMESTAMPTZ,
    verification_method VARCHAR(30),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT pk_account_email PRIMARY KEY (id)
);

CREATE TABLE account_phone (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    e164_phone_number VARCHAR(20) NOT NULL,
    extension VARCHAR(20),
    phone_role VARCHAR(30) NOT NULL,
    is_sms_enabled BOOLEAN NOT NULL,
    is_voice_enabled BOOLEAN NOT NULL,
    is_primary_for_account BOOLEAN NOT NULL,
    verified_at TIMESTAMPTZ,
    verification_method VARCHAR(30),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT pk_account_phone PRIMARY KEY (id)
);

CREATE TABLE external_identity (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    provider_code VARCHAR(40) NOT NULL,
    provider_subject VARCHAR(320) NOT NULL,
    provider_email VARCHAR(320),
    linked_at TIMESTAMPTZ NOT NULL,
    last_login_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    raw_claims_json JSONB,
    CONSTRAINT pk_external_identity PRIMARY KEY (id),
    CONSTRAINT uq_external_identity_provider UNIQUE (provider_code, provider_subject)
);

CREATE TABLE authenticator (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    authenticator_type VARCHAR(30) NOT NULL,
    usage_type VARCHAR(30) NOT NULL,
    display_label VARCHAR(120),
    status VARCHAR(30) NOT NULL,
    enrolled_at TIMESTAMPTZ NOT NULL,
    confirmed_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_authenticator PRIMARY KEY (id)
);

CREATE TABLE password_credential (
    authenticator_id UUID NOT NULL,
    password_hash TEXT NOT NULL,
    salt_value BYTEA NOT NULL,
    hash_algorithm VARCHAR(40) NOT NULL,
    hash_parameters_json JSONB NOT NULL,
    password_version INTEGER NOT NULL,
    changed_at TIMESTAMPTZ NOT NULL,
    must_rotate BOOLEAN NOT NULL,
    compromised_at TIMESTAMPTZ,
    CONSTRAINT pk_password_credential PRIMARY KEY (authenticator_id),
    CONSTRAINT chk_password_credential_version CHECK (password_version > 0)
);

CREATE TABLE password_history (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    password_hash TEXT NOT NULL,
    salt_value BYTEA NOT NULL,
    hash_algorithm VARCHAR(40) NOT NULL,
    hash_parameters_json JSONB NOT NULL,
    password_version INTEGER NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_to TIMESTAMPTZ NOT NULL,
    stored_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_password_history PRIMARY KEY (id),
    CONSTRAINT uq_password_history_account_version UNIQUE (account_id, password_version)
);

CREATE TABLE passkey_credential (
    authenticator_id UUID NOT NULL,
    rp_id VARCHAR(255) NOT NULL,
    webauthn_user_handle BYTEA NOT NULL,
    credential_id BYTEA NOT NULL,
    public_key_cose BYTEA NOT NULL,
    aaguid UUID,
    sign_count BIGINT,
    transports_json JSONB,
    attestation_format VARCHAR(80),
    credential_device_type VARCHAR(30),
    is_backup_eligible BOOLEAN,
    is_backed_up BOOLEAN,
    user_verification_policy VARCHAR(30),
    CONSTRAINT pk_passkey_credential PRIMARY KEY (authenticator_id),
    CONSTRAINT uq_passkey_credential_rp_credential UNIQUE (rp_id, credential_id)
);

CREATE TABLE totp_factor (
    authenticator_id UUID NOT NULL,
    secret_ciphertext BYTEA NOT NULL,
    key_reference VARCHAR(120),
    otp_algorithm VARCHAR(20) NOT NULL,
    digits SMALLINT NOT NULL,
    period_seconds SMALLINT NOT NULL,
    issuer_label VARCHAR(120),
    confirmed_at TIMESTAMPTZ,
    CONSTRAINT pk_totp_factor PRIMARY KEY (authenticator_id),
    CONSTRAINT chk_totp_factor_digits CHECK (digits IN (6, 8)),
    CONSTRAINT chk_totp_factor_period_seconds CHECK (period_seconds > 0)
);

CREATE TABLE recovery_code_set (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    code_count SMALLINT NOT NULL,
    status VARCHAR(30) NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL,
    replaced_by_set_id UUID,
    revoked_at TIMESTAMPTZ,
    CONSTRAINT pk_recovery_code_set PRIMARY KEY (id),
    CONSTRAINT chk_recovery_code_set_count CHECK (code_count > 0)
);

CREATE TABLE recovery_code (
    id UUID NOT NULL,
    recovery_code_set_id UUID NOT NULL,
    sequence_number SMALLINT NOT NULL,
    code_hash TEXT NOT NULL,
    salt_value BYTEA NOT NULL,
    hash_algorithm VARCHAR(40) NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_recovery_code PRIMARY KEY (id),
    CONSTRAINT uq_recovery_code_set_sequence UNIQUE (recovery_code_set_id, sequence_number)
);

CREATE TABLE "session" (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    session_token_hash TEXT,
    refresh_token_hash TEXT,
    authenticated_aal SMALLINT NOT NULL,
    remember_me BOOLEAN NOT NULL,
    user_agent TEXT,
    ip_address INET,
    device_label VARCHAR(120),
    created_at TIMESTAMPTZ NOT NULL,
    last_seen_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    CONSTRAINT pk_session PRIMARY KEY (id)
);

CREATE TABLE "role" (
    id UUID NOT NULL,
    code VARCHAR(40) NOT NULL,
    name VARCHAR(80) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_role PRIMARY KEY (id),
    CONSTRAINT uq_role_code UNIQUE (code)
);

CREATE TABLE permission (
    id UUID NOT NULL,
    code VARCHAR(80) NOT NULL,
    name VARCHAR(120) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_permission PRIMARY KEY (id),
    CONSTRAINT uq_permission_code UNIQUE (code)
);

CREATE TABLE role_permission (
    role_id UUID NOT NULL,
    permission_id UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_role_permission PRIMARY KEY (role_id, permission_id)
);

CREATE TABLE account_role (
    account_id UUID NOT NULL,
    role_id UUID NOT NULL,
    granted_by_account_id UUID,
    granted_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_account_role PRIMARY KEY (account_id, role_id)
);

CREATE TABLE setting_definition (
    id UUID NOT NULL,
    setting_key VARCHAR(120) NOT NULL,
    scope_type VARCHAR(20) NOT NULL,
    value_type VARCHAR(20) NOT NULL,
    default_value_json JSONB,
    is_sensitive BOOLEAN NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_setting_definition PRIMARY KEY (id),
    CONSTRAINT uq_setting_definition_scope_key UNIQUE (scope_type, setting_key)
);

CREATE TABLE account_setting (
    account_id UUID NOT NULL,
    setting_definition_id UUID NOT NULL,
    setting_value_json JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_account_setting PRIMARY KEY (account_id, setting_definition_id)
);

CREATE TABLE system_setting (
    setting_definition_id UUID NOT NULL,
    setting_value_json JSONB NOT NULL,
    updated_by_account_id UUID,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_system_setting PRIMARY KEY (setting_definition_id)
);

CREATE TABLE storage_object (
    id UUID NOT NULL,
    storage_provider VARCHAR(30) NOT NULL,
    bucket_name VARCHAR(120) NOT NULL,
    object_key TEXT NOT NULL,
    checksum_sha256 CHAR(64) NOT NULL,
    size_bytes BIGINT NOT NULL,
    encryption_key_ref VARCHAR(120),
    created_at TIMESTAMPTZ NOT NULL,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT pk_storage_object PRIMARY KEY (id),
    CONSTRAINT uq_storage_object_location UNIQUE (storage_provider, bucket_name, object_key),
    CONSTRAINT chk_storage_object_size_bytes CHECK (size_bytes >= 0)
);

CREATE TABLE file_asset (
    id UUID NOT NULL,
    storage_object_id UUID NOT NULL,
    owner_account_id UUID,
    original_filename VARCHAR(255) NOT NULL,
    mime_type VARCHAR(120) NOT NULL,
    file_extension VARCHAR(20),
    purpose_code VARCHAR(40) NOT NULL,
    classification_code VARCHAR(40) NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_file_asset PRIMARY KEY (id)
);

CREATE TABLE file_derivative (
    id UUID NOT NULL,
    parent_file_asset_id UUID NOT NULL,
    derivative_type VARCHAR(40) NOT NULL,
    storage_object_id UUID NOT NULL,
    created_by_process VARCHAR(80),
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_file_derivative PRIMARY KEY (id),
    CONSTRAINT uq_file_derivative_parent_type UNIQUE (parent_file_asset_id, derivative_type)
);

CREATE TABLE file_scan_result (
    id UUID NOT NULL,
    file_asset_id UUID NOT NULL,
    scan_type VARCHAR(40) NOT NULL,
    scan_status VARCHAR(30) NOT NULL,
    scanner_name VARCHAR(80),
    result_json JSONB,
    scanned_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_file_scan_result PRIMARY KEY (id)
);

CREATE TABLE file_attachment (
    id UUID NOT NULL,
    file_asset_id UUID NOT NULL,
    entity_type VARCHAR(60) NOT NULL,
    entity_id UUID NOT NULL,
    attachment_role VARCHAR(40) NOT NULL,
    sort_order INTEGER NOT NULL,
    is_primary BOOLEAN NOT NULL,
    attached_by_account_id UUID,
    metadata_json JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    removed_at TIMESTAMPTZ,
    CONSTRAINT pk_file_attachment PRIMARY KEY (id),
    CONSTRAINT chk_file_attachment_sort_order CHECK (sort_order >= 0)
);

CREATE TABLE privacy_notice_version (
    id UUID NOT NULL,
    notice_kind VARCHAR(30) NOT NULL,
    version_label VARCHAR(40) NOT NULL,
    locale VARCHAR(20) NOT NULL,
    content_hash CHAR(64) NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    retired_at TIMESTAMPTZ,
    CONSTRAINT pk_privacy_notice_version PRIMARY KEY (id),
    CONSTRAINT uq_privacy_notice_version_kind_label_locale UNIQUE (notice_kind, version_label, locale)
);

CREATE TABLE retention_policy (
    id UUID NOT NULL,
    entity_type VARCHAR(60) NOT NULL,
    trigger_event VARCHAR(40) NOT NULL,
    retain_days INTEGER NOT NULL,
    archive_after_days INTEGER,
    delete_after_days INTEGER,
    legal_basis_note TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_retention_policy PRIMARY KEY (id),
    CONSTRAINT chk_retention_policy_retain_days CHECK (retain_days >= 0)
);

CREATE TABLE processing_purpose (
    id UUID NOT NULL,
    code VARCHAR(60) NOT NULL,
    name VARCHAR(120) NOT NULL,
    lawful_basis VARCHAR(40) NOT NULL,
    consent_required BOOLEAN NOT NULL,
    retention_policy_id UUID,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_processing_purpose PRIMARY KEY (id),
    CONSTRAINT uq_processing_purpose_code UNIQUE (code)
);

CREATE TABLE consent_record (
    id UUID NOT NULL,
    account_id UUID,
    anonymous_subject_token_hash CHAR(64),
    processing_purpose_id UUID NOT NULL,
    notice_version_id UUID,
    consent_status VARCHAR(30) NOT NULL,
    captured_via VARCHAR(30) NOT NULL,
    evidence_json JSONB,
    captured_at TIMESTAMPTZ NOT NULL,
    withdrawn_at TIMESTAMPTZ,
    CONSTRAINT pk_consent_record PRIMARY KEY (id)
);

CREATE TABLE cookie_definition (
    id UUID NOT NULL,
    cookie_name VARCHAR(120) NOT NULL,
    provider_name VARCHAR(120),
    cookie_category VARCHAR(30) NOT NULL,
    is_strictly_necessary BOOLEAN NOT NULL,
    duration_seconds BIGINT,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    retired_at TIMESTAMPTZ,
    CONSTRAINT pk_cookie_definition PRIMARY KEY (id),
    CONSTRAINT uq_cookie_definition_name_provider UNIQUE (cookie_name, provider_name)
);

CREATE TABLE cookie_consent (
    id UUID NOT NULL,
    account_id UUID,
    anonymous_subject_token_hash CHAR(64),
    notice_version_id UUID,
    preferences_allowed BOOLEAN NOT NULL,
    analytics_allowed BOOLEAN NOT NULL,
    marketing_allowed BOOLEAN NOT NULL,
    captured_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    withdrawn_at TIMESTAMPTZ,
    CONSTRAINT pk_cookie_consent PRIMARY KEY (id)
);

CREATE TABLE data_subject_request (
    id UUID NOT NULL,
    account_id UUID,
    request_type VARCHAR(30) NOT NULL,
    status VARCHAR(30) NOT NULL,
    verified_by_account_id UUID,
    export_file_asset_id UUID,
    requested_at TIMESTAMPTZ NOT NULL,
    due_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    notes TEXT,
    CONSTRAINT pk_data_subject_request PRIMARY KEY (id)
);

CREATE TABLE legal_hold (
    id UUID NOT NULL,
    entity_type VARCHAR(60) NOT NULL,
    entity_id UUID NOT NULL,
    reason TEXT NOT NULL,
    placed_by_account_id UUID,
    placed_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ,
    CONSTRAINT pk_legal_hold PRIMARY KEY (id)
);

CREATE TABLE notification (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    notification_type VARCHAR(40) NOT NULL,
    payload_json JSONB NOT NULL,
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_notification PRIMARY KEY (id)
);

CREATE TABLE audit_log (
    id UUID NOT NULL,
    actor_account_id UUID,
    action_code VARCHAR(80) NOT NULL,
    entity_type VARCHAR(60) NOT NULL,
    entity_id UUID,
    request_id UUID,
    old_value_json JSONB,
    new_value_json JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_audit_log PRIMARY KEY (id)
);

CREATE TABLE security_event (
    id UUID NOT NULL,
    account_id UUID,
    session_id UUID,
    ip_address INET,
    severity VARCHAR(20) NOT NULL,
    event_type VARCHAR(60) NOT NULL,
    details_json JSONB,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_security_event PRIMARY KEY (id)
);

CREATE TABLE account_suspension (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    reason TEXT NOT NULL,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ,
    created_by_account_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_account_suspension PRIMARY KEY (id)
);

-- =========================================================
-- 2. PriceTracker business domain
-- =========================================================

CREATE TABLE category (
    id UUID NOT NULL,
    parent_category_id UUID,
    name VARCHAR(120) NOT NULL,
    normalized_name VARCHAR(120) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_category PRIMARY KEY (id),
    CONSTRAINT uq_category_parent_normalized_name UNIQUE (parent_category_id, normalized_name)
);

CREATE TABLE address (
    id UUID NOT NULL,
    country_code CHAR(2) NOT NULL,
    building_number VARCHAR(40),
    street_name VARCHAR(200),
    street_line2 VARCHAR(200),
    unit VARCHAR(80),
    floor VARCHAR(40),
    building_name VARCHAR(200),
    district VARCHAR(120),
    city VARCHAR(120),
    state_region VARCHAR(120),
    postal_code VARCHAR(40),
    landmark VARCHAR(200),
    full_text TEXT,
    latitude NUMERIC(10,7),
    longitude NUMERIC(10,7),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_address PRIMARY KEY (id)
);

CREATE TABLE brand (
    id UUID NOT NULL,
    name VARCHAR(160) NOT NULL,
    normalized_name VARCHAR(160) NOT NULL,
    country_code CHAR(2),
    website_url TEXT,
    headquarters_address_id UUID,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_brand PRIMARY KEY (id),
    CONSTRAINT uq_brand_normalized_name UNIQUE (normalized_name)
);

CREATE TABLE unit_family (
    id UUID NOT NULL,
    code VARCHAR(40) NOT NULL,
    name VARCHAR(80) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_unit_family PRIMARY KEY (id),
    CONSTRAINT uq_unit_family_code UNIQUE (code)
);

CREATE TABLE unit (
    id UUID NOT NULL,
    unit_family_id UUID NOT NULL,
    code VARCHAR(40) NOT NULL,
    name VARCHAR(80) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    factor_to_base NUMERIC(20,8) NOT NULL,
    is_base_unit BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_unit PRIMARY KEY (id),
    CONSTRAINT uq_unit_unit_family_code UNIQUE (unit_family_id, code),
    CONSTRAINT chk_unit_factor_to_base CHECK (factor_to_base > 0)
);

CREATE TABLE currency (
    code CHAR(3) NOT NULL,
    name VARCHAR(80) NOT NULL,
    symbol VARCHAR(10),
    minor_unit SMALLINT NOT NULL,
    is_active BOOLEAN NOT NULL,
    CONSTRAINT pk_currency PRIMARY KEY (code)
);

CREATE TABLE item (
    id UUID NOT NULL,
    category_id UUID,
    canonical_name VARCHAR(200) NOT NULL,
    normalized_name VARCHAR(200) NOT NULL,
    specification_text TEXT,
    description TEXT,
    status VARCHAR(30) NOT NULL,
    created_by_account_id UUID,
    approved_by_account_id UUID,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_item PRIMARY KEY (id)
);

CREATE TABLE item_variant (
    id UUID NOT NULL,
    item_id UUID NOT NULL,
    brand_id UUID,
    variant_name VARCHAR(160),
    package_quantity NUMERIC(18,6) NOT NULL,
    package_unit_id UUID NOT NULL,
    pack_count INTEGER NOT NULL DEFAULT 1,
    normalized_content_quantity NUMERIC(18,6),
    normalized_content_unit_id UUID,
    status VARCHAR(30) NOT NULL,
    created_by_account_id UUID,
    approved_by_account_id UUID,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_item_variant PRIMARY KEY (id),
    CONSTRAINT chk_item_variant_package_quantity CHECK (package_quantity > 0),
    CONSTRAINT chk_item_variant_pack_count CHECK (pack_count > 0)
);

CREATE TABLE retailer (
    id UUID NOT NULL,
    name VARCHAR(160) NOT NULL,
    normalized_name VARCHAR(160) NOT NULL,
    retailer_type VARCHAR(30) NOT NULL,
    website_url TEXT,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_retailer PRIMARY KEY (id),
    CONSTRAINT uq_retailer_normalized_name UNIQUE (normalized_name)
);

CREATE TABLE shop (
    id UUID NOT NULL,
    retailer_id UUID NOT NULL,
    name VARCHAR(200) NOT NULL,
    address_id UUID,
    phone_number VARCHAR(50),
    is_online BOOLEAN NOT NULL,
    latitude NUMERIC(10,7),
    longitude NUMERIC(10,7),
    timezone_name VARCHAR(80),
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_shop PRIMARY KEY (id)
);

CREATE TABLE variant_identifier (
    id UUID NOT NULL,
    item_variant_id UUID NOT NULL,
    identifier_type VARCHAR(40) NOT NULL,
    scope_type VARCHAR(20) NOT NULL,
    retailer_id UUID,
    shop_id UUID,
    identifier_value VARCHAR(200) NOT NULL,
    is_primary BOOLEAN NOT NULL,
    valid_from TIMESTAMPTZ,
    valid_to TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_variant_identifier PRIMARY KEY (id),
    CONSTRAINT chk_variant_identifier_scope_refs CHECK (
        (scope_type <> 'SHOP' OR shop_id IS NOT NULL)
        AND
        (scope_type <> 'RETAILER' OR retailer_id IS NOT NULL)
    )
);

CREATE TABLE shop_listing (
    id UUID NOT NULL,
    shop_id UUID NOT NULL,
    item_variant_id UUID NOT NULL,
    external_sku VARCHAR(200),
    listing_url TEXT,
    first_seen_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_shop_listing PRIMARY KEY (id),
    CONSTRAINT uq_shop_listing_shop_variant UNIQUE (shop_id, item_variant_id)
);

CREATE TABLE discount_type (
    id UUID NOT NULL,
    code VARCHAR(50) NOT NULL,
    name VARCHAR(120) NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_discount_type PRIMARY KEY (id),
    CONSTRAINT uq_discount_type_code UNIQUE (code)
);

CREATE TABLE data_source (
    id UUID NOT NULL,
    source_type VARCHAR(30) NOT NULL,
    account_id UUID,
    source_name VARCHAR(160),
    trust_score NUMERIC(5,2),
    is_verified BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_data_source PRIMARY KEY (id)
);

CREATE TABLE price_observation (
    id UUID NOT NULL,
    item_variant_id UUID NOT NULL,
    shop_id UUID NOT NULL,
    shop_listing_id UUID,
    source_id UUID NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL,
    currency_code CHAR(3) NOT NULL,
    list_price_amount NUMERIC(14,4),
    final_price_amount NUMERIC(14,4) NOT NULL,
    discount_amount NUMERIC(14,4),
    discount_type_id UUID,
    unit_price_amount NUMERIC(14,6),
    unit_price_unit_id UUID,
    status VARCHAR(30) NOT NULL,
    confidence_score NUMERIC(5,2),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_price_observation PRIMARY KEY (id),
    CONSTRAINT chk_price_observation_final_price CHECK (final_price_amount >= 0),
    CONSTRAINT chk_price_observation_list_price CHECK (list_price_amount IS NULL OR list_price_amount >= 0),
    CONSTRAINT chk_price_observation_discount_amount CHECK (discount_amount IS NULL OR discount_amount >= 0),
    CONSTRAINT chk_price_observation_list_ge_final CHECK (list_price_amount IS NULL OR list_price_amount >= final_price_amount)
);

CREATE TABLE purchase (
    id UUID NOT NULL,
    purchaser_account_id UUID,
    shop_id UUID NOT NULL,
    purchased_at TIMESTAMPTZ NOT NULL,
    currency_code CHAR(3) NOT NULL,
    receipt_number VARCHAR(200),
    seller_tax_identifier VARCHAR(200),
    total_amount NUMERIC(14,4),
    tax_amount NUMERIC(14,4),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_purchase PRIMARY KEY (id)
);

CREATE TABLE purchase_line (
    id UUID NOT NULL,
    purchase_id UUID NOT NULL,
    price_observation_id UUID NOT NULL,
    line_number INTEGER,
    quantity_purchased NUMERIC(18,6) NOT NULL,
    batch_code VARCHAR(200),
    serial_number VARCHAR(200),
    vat_rate NUMERIC(6,3),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_purchase_line PRIMARY KEY (id),
    CONSTRAINT chk_purchase_line_quantity CHECK (quantity_purchased > 0),
    CONSTRAINT uq_purchase_line_price_observation UNIQUE (price_observation_id)
);

CREATE TABLE watchlist_item (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    item_variant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_watchlist_item PRIMARY KEY (id),
    CONSTRAINT uq_watchlist_item_account_variant UNIQUE (account_id, item_variant_id)
);

CREATE TABLE watchlist_shop (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    shop_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_watchlist_shop PRIMARY KEY (id),
    CONSTRAINT uq_watchlist_shop_account_shop UNIQUE (account_id, shop_id)
);

CREATE TABLE price_alert (
    id UUID NOT NULL,
    account_id UUID NOT NULL,
    item_variant_id UUID NOT NULL,
    shop_id UUID,
    target_price_amount NUMERIC(14,4) NOT NULL,
    currency_code CHAR(3) NOT NULL,
    is_active BOOLEAN NOT NULL,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_price_alert PRIMARY KEY (id)
);

CREATE TABLE price_vote (
    id UUID NOT NULL,
    price_observation_id UUID NOT NULL,
    account_id UUID NOT NULL,
    vote_value SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_price_vote PRIMARY KEY (id),
    CONSTRAINT chk_price_vote_value CHECK (vote_value IN (-1, 1)),
    CONSTRAINT uq_price_vote_observation_account UNIQUE (price_observation_id, account_id)
);

CREATE TABLE price_report (
    id UUID NOT NULL,
    price_observation_id UUID NOT NULL,
    reported_by_account_id UUID NOT NULL,
    reason_code VARCHAR(50) NOT NULL,
    details TEXT,
    status VARCHAR(30) NOT NULL,
    resolved_by_account_id UUID,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_price_report PRIMARY KEY (id)
);

CREATE TABLE moderation_decision (
    id UUID NOT NULL,
    entity_type VARCHAR(40) NOT NULL,
    entity_id UUID NOT NULL,
    decision VARCHAR(30) NOT NULL,
    moderator_account_id UUID NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT pk_moderation_decision PRIMARY KEY (id)
);

-- =========================================================
-- 3. Foreign keys
-- =========================================================

ALTER TABLE account_profile
    ADD CONSTRAINT fk_account_profile_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_account_profile_preferred_currency
        FOREIGN KEY (preferred_currency_code) REFERENCES currency(code);

ALTER TABLE account_email
    ADD CONSTRAINT fk_account_email_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE account_phone
    ADD CONSTRAINT fk_account_phone_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE external_identity
    ADD CONSTRAINT fk_external_identity_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE authenticator
    ADD CONSTRAINT fk_authenticator_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE password_credential
    ADD CONSTRAINT fk_password_credential_authenticator
        FOREIGN KEY (authenticator_id) REFERENCES authenticator(id);

ALTER TABLE password_history
    ADD CONSTRAINT fk_password_history_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE passkey_credential
    ADD CONSTRAINT fk_passkey_credential_authenticator
        FOREIGN KEY (authenticator_id) REFERENCES authenticator(id);

ALTER TABLE totp_factor
    ADD CONSTRAINT fk_totp_factor_authenticator
        FOREIGN KEY (authenticator_id) REFERENCES authenticator(id);

ALTER TABLE recovery_code_set
    ADD CONSTRAINT fk_recovery_code_set_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_recovery_code_set_replaced_by
        FOREIGN KEY (replaced_by_set_id) REFERENCES recovery_code_set(id);

ALTER TABLE recovery_code
    ADD CONSTRAINT fk_recovery_code_set
        FOREIGN KEY (recovery_code_set_id) REFERENCES recovery_code_set(id);

ALTER TABLE "session"
    ADD CONSTRAINT fk_session_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE role_permission
    ADD CONSTRAINT fk_role_permission_role
        FOREIGN KEY (role_id) REFERENCES "role"(id),
    ADD CONSTRAINT fk_role_permission_permission
        FOREIGN KEY (permission_id) REFERENCES permission(id);

ALTER TABLE account_role
    ADD CONSTRAINT fk_account_role_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_account_role_role
        FOREIGN KEY (role_id) REFERENCES "role"(id),
    ADD CONSTRAINT fk_account_role_granted_by_account
        FOREIGN KEY (granted_by_account_id) REFERENCES account(id);

ALTER TABLE account_setting
    ADD CONSTRAINT fk_account_setting_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_account_setting_definition
        FOREIGN KEY (setting_definition_id) REFERENCES setting_definition(id);

ALTER TABLE system_setting
    ADD CONSTRAINT fk_system_setting_definition
        FOREIGN KEY (setting_definition_id) REFERENCES setting_definition(id),
    ADD CONSTRAINT fk_system_setting_updated_by_account
        FOREIGN KEY (updated_by_account_id) REFERENCES account(id);

ALTER TABLE file_asset
    ADD CONSTRAINT fk_file_asset_storage_object
        FOREIGN KEY (storage_object_id) REFERENCES storage_object(id),
    ADD CONSTRAINT fk_file_asset_owner_account
        FOREIGN KEY (owner_account_id) REFERENCES account(id);

ALTER TABLE file_derivative
    ADD CONSTRAINT fk_file_derivative_parent_file_asset
        FOREIGN KEY (parent_file_asset_id) REFERENCES file_asset(id),
    ADD CONSTRAINT fk_file_derivative_storage_object
        FOREIGN KEY (storage_object_id) REFERENCES storage_object(id);

ALTER TABLE file_scan_result
    ADD CONSTRAINT fk_file_scan_result_file_asset
        FOREIGN KEY (file_asset_id) REFERENCES file_asset(id);

ALTER TABLE file_attachment
    ADD CONSTRAINT fk_file_attachment_file_asset
        FOREIGN KEY (file_asset_id) REFERENCES file_asset(id),
    ADD CONSTRAINT fk_file_attachment_attached_by_account
        FOREIGN KEY (attached_by_account_id) REFERENCES account(id);

ALTER TABLE processing_purpose
    ADD CONSTRAINT fk_processing_purpose_retention_policy
        FOREIGN KEY (retention_policy_id) REFERENCES retention_policy(id);

ALTER TABLE consent_record
    ADD CONSTRAINT fk_consent_record_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_consent_record_processing_purpose
        FOREIGN KEY (processing_purpose_id) REFERENCES processing_purpose(id),
    ADD CONSTRAINT fk_consent_record_notice_version
        FOREIGN KEY (notice_version_id) REFERENCES privacy_notice_version(id);

ALTER TABLE cookie_consent
    ADD CONSTRAINT fk_cookie_consent_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_cookie_consent_notice_version
        FOREIGN KEY (notice_version_id) REFERENCES privacy_notice_version(id);

ALTER TABLE data_subject_request
    ADD CONSTRAINT fk_data_subject_request_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_data_subject_request_verified_by_account
        FOREIGN KEY (verified_by_account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_data_subject_request_export_file_asset
        FOREIGN KEY (export_file_asset_id) REFERENCES file_asset(id);

ALTER TABLE legal_hold
    ADD CONSTRAINT fk_legal_hold_placed_by_account
        FOREIGN KEY (placed_by_account_id) REFERENCES account(id);

ALTER TABLE notification
    ADD CONSTRAINT fk_notification_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE audit_log
    ADD CONSTRAINT fk_audit_log_actor_account
        FOREIGN KEY (actor_account_id) REFERENCES account(id);

ALTER TABLE security_event
    ADD CONSTRAINT fk_security_event_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_security_event_session
        FOREIGN KEY (session_id) REFERENCES "session"(id);

ALTER TABLE account_suspension
    ADD CONSTRAINT fk_account_suspension_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_account_suspension_created_by_account
        FOREIGN KEY (created_by_account_id) REFERENCES account(id);

ALTER TABLE category
    ADD CONSTRAINT fk_category_parent_category
        FOREIGN KEY (parent_category_id) REFERENCES category(id);

ALTER TABLE brand
    ADD CONSTRAINT fk_brand_headquarters_address
        FOREIGN KEY (headquarters_address_id) REFERENCES address(id);

ALTER TABLE unit
    ADD CONSTRAINT fk_unit_family
        FOREIGN KEY (unit_family_id) REFERENCES unit_family(id);

ALTER TABLE item
    ADD CONSTRAINT fk_item_category
        FOREIGN KEY (category_id) REFERENCES category(id),
    ADD CONSTRAINT fk_item_created_by_account
        FOREIGN KEY (created_by_account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_item_approved_by_account
        FOREIGN KEY (approved_by_account_id) REFERENCES account(id);

ALTER TABLE item_variant
    ADD CONSTRAINT fk_item_variant_item
        FOREIGN KEY (item_id) REFERENCES item(id),
    ADD CONSTRAINT fk_item_variant_brand
        FOREIGN KEY (brand_id) REFERENCES brand(id),
    ADD CONSTRAINT fk_item_variant_package_unit
        FOREIGN KEY (package_unit_id) REFERENCES unit(id),
    ADD CONSTRAINT fk_item_variant_normalized_content_unit
        FOREIGN KEY (normalized_content_unit_id) REFERENCES unit(id),
    ADD CONSTRAINT fk_item_variant_created_by_account
        FOREIGN KEY (created_by_account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_item_variant_approved_by_account
        FOREIGN KEY (approved_by_account_id) REFERENCES account(id);

ALTER TABLE shop
    ADD CONSTRAINT fk_shop_retailer
        FOREIGN KEY (retailer_id) REFERENCES retailer(id),
    ADD CONSTRAINT fk_shop_address
        FOREIGN KEY (address_id) REFERENCES address(id);

ALTER TABLE variant_identifier
    ADD CONSTRAINT fk_variant_identifier_item_variant
        FOREIGN KEY (item_variant_id) REFERENCES item_variant(id),
    ADD CONSTRAINT fk_variant_identifier_retailer
        FOREIGN KEY (retailer_id) REFERENCES retailer(id),
    ADD CONSTRAINT fk_variant_identifier_shop
        FOREIGN KEY (shop_id) REFERENCES shop(id);

ALTER TABLE shop_listing
    ADD CONSTRAINT fk_shop_listing_shop
        FOREIGN KEY (shop_id) REFERENCES shop(id),
    ADD CONSTRAINT fk_shop_listing_item_variant
        FOREIGN KEY (item_variant_id) REFERENCES item_variant(id);

ALTER TABLE data_source
    ADD CONSTRAINT fk_data_source_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE price_observation
    ADD CONSTRAINT fk_price_observation_item_variant
        FOREIGN KEY (item_variant_id) REFERENCES item_variant(id),
    ADD CONSTRAINT fk_price_observation_shop
        FOREIGN KEY (shop_id) REFERENCES shop(id),
    ADD CONSTRAINT fk_price_observation_shop_listing
        FOREIGN KEY (shop_listing_id) REFERENCES shop_listing(id),
    ADD CONSTRAINT fk_price_observation_source
        FOREIGN KEY (source_id) REFERENCES data_source(id),
    ADD CONSTRAINT fk_price_observation_currency
        FOREIGN KEY (currency_code) REFERENCES currency(code),
    ADD CONSTRAINT fk_price_observation_discount_type
        FOREIGN KEY (discount_type_id) REFERENCES discount_type(id),
    ADD CONSTRAINT fk_price_observation_unit_price_unit
        FOREIGN KEY (unit_price_unit_id) REFERENCES unit(id);

ALTER TABLE purchase
    ADD CONSTRAINT fk_purchase_purchaser_account
        FOREIGN KEY (purchaser_account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_purchase_shop
        FOREIGN KEY (shop_id) REFERENCES shop(id),
    ADD CONSTRAINT fk_purchase_currency
        FOREIGN KEY (currency_code) REFERENCES currency(code);

ALTER TABLE purchase_line
    ADD CONSTRAINT fk_purchase_line_purchase
        FOREIGN KEY (purchase_id) REFERENCES purchase(id),
    ADD CONSTRAINT fk_purchase_line_price_observation
        FOREIGN KEY (price_observation_id) REFERENCES price_observation(id);

ALTER TABLE watchlist_item
    ADD CONSTRAINT fk_watchlist_item_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_watchlist_item_item_variant
        FOREIGN KEY (item_variant_id) REFERENCES item_variant(id);

ALTER TABLE watchlist_shop
    ADD CONSTRAINT fk_watchlist_shop_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_watchlist_shop_shop
        FOREIGN KEY (shop_id) REFERENCES shop(id);

ALTER TABLE price_alert
    ADD CONSTRAINT fk_price_alert_account
        FOREIGN KEY (account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_price_alert_item_variant
        FOREIGN KEY (item_variant_id) REFERENCES item_variant(id),
    ADD CONSTRAINT fk_price_alert_shop
        FOREIGN KEY (shop_id) REFERENCES shop(id),
    ADD CONSTRAINT fk_price_alert_currency
        FOREIGN KEY (currency_code) REFERENCES currency(code);

ALTER TABLE price_vote
    ADD CONSTRAINT fk_price_vote_price_observation
        FOREIGN KEY (price_observation_id) REFERENCES price_observation(id),
    ADD CONSTRAINT fk_price_vote_account
        FOREIGN KEY (account_id) REFERENCES account(id);

ALTER TABLE price_report
    ADD CONSTRAINT fk_price_report_price_observation
        FOREIGN KEY (price_observation_id) REFERENCES price_observation(id),
    ADD CONSTRAINT fk_price_report_reported_by_account
        FOREIGN KEY (reported_by_account_id) REFERENCES account(id),
    ADD CONSTRAINT fk_price_report_resolved_by_account
        FOREIGN KEY (resolved_by_account_id) REFERENCES account(id);

ALTER TABLE moderation_decision
    ADD CONSTRAINT fk_moderation_decision_moderator_account
        FOREIGN KEY (moderator_account_id) REFERENCES account(id);

-- =========================================================
-- 4. Partial unique indexes and recommended indexes
-- =========================================================

CREATE UNIQUE INDEX uq_account_email_account_normalized_email_active
    ON account_email (account_id, normalized_email)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX uq_account_email_one_active_primary
    ON account_email (account_id)
    WHERE deleted_at IS NULL AND is_primary_for_account IS TRUE;

CREATE INDEX idx_account_email_active_lookup
    ON account_email (normalized_email)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX uq_account_phone_account_e164_active
    ON account_phone (account_id, e164_phone_number)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX uq_account_phone_one_active_primary
    ON account_phone (account_id)
    WHERE deleted_at IS NULL AND is_primary_for_account IS TRUE;

CREATE INDEX idx_account_phone_active_lookup
    ON account_phone (e164_phone_number)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX uq_authenticator_one_active_password_per_account
    ON authenticator (account_id)
    WHERE authenticator_type = 'PASSWORD' AND status = 'active';

CREATE INDEX idx_authenticator_account_type_status
    ON authenticator (account_id, authenticator_type, status);

CREATE INDEX idx_session_account_expires_at
    ON "session" (account_id, expires_at DESC);

CREATE INDEX idx_security_event_account_created_at
    ON security_event (account_id, created_at DESC);

CREATE INDEX idx_security_event_ip_created_at
    ON security_event (ip_address, created_at DESC);

CREATE INDEX idx_file_asset_owner_created_at
    ON file_asset (owner_account_id, created_at DESC);

CREATE INDEX idx_file_attachment_entity_created_at
    ON file_attachment (entity_type, entity_id, created_at DESC);

CREATE INDEX idx_file_attachment_file_asset
    ON file_attachment (file_asset_id);

CREATE INDEX idx_file_scan_result_file_scanned_at
    ON file_scan_result (file_asset_id, scanned_at DESC);

CREATE INDEX idx_consent_record_account_purpose_captured_at
    ON consent_record (account_id, processing_purpose_id, captured_at DESC);

CREATE INDEX idx_cookie_consent_account_updated_at
    ON cookie_consent (account_id, updated_at DESC);

CREATE INDEX idx_data_subject_request_account_requested_at
    ON data_subject_request (account_id, requested_at DESC);

CREATE INDEX idx_retention_policy_entity_type
    ON retention_policy (entity_type);

CREATE INDEX idx_item_normalized_name
    ON item (normalized_name);

CREATE INDEX idx_item_variant_item_id
    ON item_variant (item_id);

CREATE INDEX idx_variant_identifier_item_variant_id
    ON variant_identifier (item_variant_id);

CREATE UNIQUE INDEX uq_variant_identifier_global_scope
    ON variant_identifier (identifier_type, identifier_value)
    WHERE scope_type = 'GLOBAL';

CREATE UNIQUE INDEX uq_variant_identifier_retailer_scope
    ON variant_identifier (retailer_id, identifier_type, identifier_value)
    WHERE scope_type = 'RETAILER';

CREATE UNIQUE INDEX uq_variant_identifier_shop_scope
    ON variant_identifier (shop_id, identifier_type, identifier_value)
    WHERE scope_type = 'SHOP';

CREATE INDEX idx_shop_retailer_id
    ON shop (retailer_id);

CREATE UNIQUE INDEX uq_shop_listing_shop_external_sku
    ON shop_listing (shop_id, external_sku)
    WHERE external_sku IS NOT NULL;

CREATE INDEX idx_price_observation_variant_observed_at
    ON price_observation (item_variant_id, observed_at DESC);

CREATE INDEX idx_price_observation_shop_observed_at
    ON price_observation (shop_id, observed_at DESC);

CREATE INDEX idx_price_observation_variant_shop_observed_at
    ON price_observation (item_variant_id, shop_id, observed_at DESC);

CREATE INDEX idx_purchase_shop_purchased_at
    ON purchase (shop_id, purchased_at DESC);

CREATE INDEX idx_purchase_line_purchase_id
    ON purchase_line (purchase_id);

COMMIT;
