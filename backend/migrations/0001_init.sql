CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE currency (
    code CHAR(3) PRIMARY KEY,
    name TEXT NOT NULL
);

CREATE TABLE category (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    parent_id UUID REFERENCES category(id) ON DELETE SET NULL
);

CREATE TABLE brand (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE unit_family (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE unit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    family_id UUID NOT NULL REFERENCES unit_family(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    base_factor NUMERIC(18,6) NOT NULL DEFAULT 1
);

CREATE TABLE discount_type (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE retailer (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE address (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    line_1 TEXT NOT NULL,
    line_2 TEXT,
    city TEXT NOT NULL,
    postcode TEXT,
    country_code CHAR(2) NOT NULL DEFAULT 'GB'
);

CREATE TABLE shop (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    retailer_id UUID REFERENCES retailer(id) ON DELETE SET NULL,
    address_id UUID REFERENCES address(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    display_address TEXT,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE account (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    public_handle VARCHAR(80),
    account_status VARCHAR(30) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    last_active_at TIMESTAMPTZ
);

CREATE TABLE account_profile (
    account_id UUID PRIMARY KEY REFERENCES account(id) ON DELETE CASCADE,
    display_name VARCHAR(160),
    locale VARCHAR(20),
    timezone_name VARCHAR(80),
    preferred_currency_code CHAR(3) REFERENCES currency(code),
    profile_bio TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE account_email (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    email VARCHAR(320) NOT NULL,
    normalized_email VARCHAR(320) NOT NULL,
    email_role VARCHAR(30) NOT NULL DEFAULT 'PRIMARY',
    is_login_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    is_primary_for_account BOOLEAN NOT NULL DEFAULT TRUE,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX account_email_normalized_unique
    ON account_email(normalized_email)
    WHERE deleted_at IS NULL;

CREATE TABLE account_phone (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    phone_number VARCHAR(20) NOT NULL,
    is_primary_for_account BOOLEAN NOT NULL DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE legal_document (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_key TEXT NOT NULL,
    version TEXT NOT NULL,
    title TEXT NOT NULL,
    content_url TEXT,
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE consent_record (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    document_key TEXT NOT NULL,
    version TEXT NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id UUID NOT NULL REFERENCES category(id),
    name TEXT NOT NULL,
    specification TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE item_variant (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id UUID NOT NULL REFERENCES item(id) ON DELETE CASCADE,
    brand_id UUID NOT NULL REFERENCES brand(id),
    unit_id UUID NOT NULL REFERENCES unit(id),
    quantity NUMERIC(18,4) NOT NULL,
    website TEXT
);

CREATE TABLE variant_identifier (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    variant_id UUID NOT NULL REFERENCES item_variant(id) ON DELETE CASCADE,
    code TEXT NOT NULL,
    code_type TEXT NOT NULL,
    scope TEXT NOT NULL DEFAULT 'global',
    shop_id UUID REFERENCES shop(id) ON DELETE SET NULL,
    label TEXT
);

CREATE TABLE file_asset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    purpose TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending_upload',
    metadata_stripped BOOLEAN NOT NULL DEFAULT FALSE,
    storage_key TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE purchase (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    shop_id UUID NOT NULL REFERENCES shop(id),
    purchase_time TIMESTAMPTZ NOT NULL,
    notes TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE file_attachment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL REFERENCES file_asset(id) ON DELETE CASCADE,
    attached_to_type TEXT NOT NULL,
    attached_to_id UUID NOT NULL,
    attachment_role TEXT NOT NULL DEFAULT 'evidence'
);

CREATE TABLE price_observation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    item_variant_id UUID NOT NULL REFERENCES item_variant(id),
    purchase_id UUID NOT NULL REFERENCES purchase(id),
    original_amount NUMERIC(18,4) NOT NULL,
    original_currency CHAR(3) NOT NULL REFERENCES currency(code),
    discount_amount NUMERIC(18,4),
    discount_currency CHAR(3) REFERENCES currency(code),
    discount_type_id UUID REFERENCES discount_type(id) ON DELETE SET NULL,
    final_amount NUMERIC(18,4) NOT NULL,
    submission_status TEXT NOT NULL DEFAULT 'submitted',
    visibility TEXT NOT NULL DEFAULT 'private',
    published BOOLEAN NOT NULL DEFAULT FALSE,
    recorded_at TIMESTAMPTZ NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE watchlist_item (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    item_variant_id UUID NOT NULL REFERENCES item_variant(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE price_alert (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    item_variant_id UUID NOT NULL REFERENCES item_variant(id),
    target_final_amount NUMERIC(18,4) NOT NULL,
    currency CHAR(3) NOT NULL REFERENCES currency(code),
    is_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO currency (code, name) VALUES
('GBP', 'British Pound'),
('EUR', 'Euro')
ON CONFLICT DO NOTHING;

WITH dairy AS (
    INSERT INTO category (name) VALUES ('Dairy') RETURNING id
),
family AS (
    INSERT INTO unit_family (name) VALUES ('volume') RETURNING id
),
litre AS (
    INSERT INTO unit (family_id, name, symbol, base_factor)
    SELECT id, 'Litre', 'L', 1 FROM family
    RETURNING id
),
retailer_seed AS (
    INSERT INTO retailer (name) VALUES ('Tesco') RETURNING id
),
address_seed AS (
    INSERT INTO address (line_1, city, postcode, country_code)
    VALUES ('12 High Street', 'Bristol', 'BS1 1AA', 'GB')
    RETURNING id
),
shop_seed AS (
    INSERT INTO shop (retailer_id, address_id, name, display_address, is_verified)
    SELECT retailer_seed.id, address_seed.id, 'Tesco Extra', '12 High Street, Bristol', TRUE
    FROM retailer_seed, address_seed
    RETURNING id
),
brand_seed AS (
    INSERT INTO brand (name) VALUES ('Tesco'), ('Aldi'), ('Lidl')
    ON CONFLICT (name) DO NOTHING
),
discount_seed AS (
    INSERT INTO discount_type (name, description)
    VALUES ('Membership discount', 'Applies when a loyalty membership was used.')
    ON CONFLICT (name) DO NOTHING
),
account_seed AS (
    INSERT INTO account (public_handle) VALUES ('demo-user') RETURNING id
),
profile_seed AS (
    INSERT INTO account_profile (account_id, display_name, locale, timezone_name, preferred_currency_code)
    SELECT id, 'Alex Pricewatch', 'en-GB', 'Europe/London', 'GBP' FROM account_seed
),
email_seed AS (
    INSERT INTO account_email (account_id, email, normalized_email, verified_at)
    SELECT id, 'alex@example.com', 'alex@example.com', NOW() FROM account_seed
),
phone_seed AS (
    INSERT INTO account_phone (account_id, phone_number, is_primary_for_account, verified_at)
    SELECT id, '+447700900123', TRUE, NOW() FROM account_seed
),
legal_seed AS (
    INSERT INTO legal_document (document_key, version, title, content_url, is_current)
    VALUES
    ('terms_of_service', '2026-04-01', 'Terms of Service', 'https://example.com/legal/terms', TRUE),
    ('privacy_policy', '2026-04-01', 'Privacy Policy', 'https://example.com/legal/privacy', TRUE)
),
consent_seed AS (
    INSERT INTO consent_record (account_id, document_key, version)
    SELECT account_seed.id, 'terms_of_service', '2026-04-01' FROM account_seed
    UNION ALL
    SELECT account_seed.id, 'privacy_policy', '2026-04-01' FROM account_seed
),
item_seed AS (
    INSERT INTO item (category_id, name, specification)
    SELECT dairy.id, 'Milk', 'Semi-skimmed' FROM dairy
    RETURNING id
),
variant_seed AS (
    INSERT INTO item_variant (item_id, brand_id, unit_id, quantity, website)
    SELECT item_seed.id,
           (SELECT id FROM brand WHERE name = 'Tesco' LIMIT 1),
           litre.id,
           1.0,
           'https://example.com/items/milk-1l'
    FROM item_seed, litre
    RETURNING id
)
INSERT INTO variant_identifier (variant_id, code, code_type, scope, shop_id, label)
SELECT variant_seed.id, '5012345678901', 'gtin', 'global', NULL, 'Manufacturer barcode'
FROM variant_seed;
