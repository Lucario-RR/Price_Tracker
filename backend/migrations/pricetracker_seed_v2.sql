BEGIN;

INSERT INTO currency (code, name, symbol, minor_unit, is_active) VALUES
('GBP', 'British Pound', '£', 2, TRUE),
('EUR', 'Euro', '€', 2, TRUE)
ON CONFLICT (code) DO NOTHING;

INSERT INTO category (id, parent_category_id, name, normalized_name, description, is_active, created_at, updated_at) VALUES
('c1111111-1111-4111-8111-111111111111', NULL, 'Dairy', 'dairy', 'Milk, cheese, yoghurt and related products.', TRUE, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO address (
    id, country_code, building_number, street_name, street_line2, unit, floor, building_name,
    district, city, state_region, postal_code, latitude, longitude, full_text, created_at, updated_at
) VALUES (
    'a1111111-1111-4111-8111-111111111111', 'GB', '12', 'High Street', NULL, NULL, NULL, NULL,
    NULL, 'Bristol', 'England', 'BS1 1AA', NULL, NULL, '12 High Street, Bristol BS1 1AA, UK', NOW(), NOW()
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO brand (id, name, normalized_name, country_code, website_url, headquarters_address_id, is_active, created_at, updated_at) VALUES
('b1111111-1111-4111-8111-111111111111', 'Tesco', 'tesco', 'GB', 'https://www.tesco.com', NULL, TRUE, NOW(), NOW()),
('b2222222-2222-4222-8222-222222222222', 'Aldi', 'aldi', 'GB', 'https://www.aldi.co.uk', NULL, TRUE, NOW(), NOW()),
('b3333333-3333-4333-8333-333333333333', 'Lidl', 'lidl', 'GB', 'https://www.lidl.co.uk', NULL, TRUE, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO unit_family (id, code, name, created_at) VALUES
('aa111111-1111-4111-8111-111111111111', 'volume', 'Volume', NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO unit (id, unit_family_id, code, name, symbol, factor_to_base, is_base_unit, created_at) VALUES
('aa222222-2222-4222-8222-222222222222', 'aa111111-1111-4111-8111-111111111111', 'l', 'Litre', 'L', 1.00000000, TRUE, NOW()),
('aa333333-3333-4333-8333-333333333333', 'aa111111-1111-4111-8111-111111111111', 'ml', 'Millilitre', 'mL', 0.00100000, FALSE, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO retailer (id, name, normalized_name, retailer_type, website_url, is_active, created_at, updated_at) VALUES
('aa444444-4444-4444-8444-444444444444', 'Tesco', 'tesco', 'SUPERMARKET', 'https://www.tesco.com', TRUE, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO shop (id, retailer_id, name, address_id, phone_number, is_online, latitude, longitude, timezone_name, is_active, created_at, updated_at) VALUES
('33333333-3333-4333-8333-333333333333', 'aa444444-4444-4444-8444-444444444444', 'Tesco Extra', 'a1111111-1111-4111-8111-111111111111', NULL, FALSE, NULL, NULL, 'Europe/London', TRUE, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO discount_type (id, code, name, description, is_active, created_at) VALUES
('d1111111-1111-4111-8111-111111111111', 'membership_discount', 'Membership discount', 'Applies when a loyalty membership was used.', TRUE, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO privacy_notice_version (id, notice_kind, version_label, locale, content_hash, published_at, retired_at) VALUES
('aa555555-5555-4555-8555-555555555555', 'terms_of_service', '2026-04-01', 'en-GB', '1111111111111111111111111111111111111111111111111111111111111111', NOW(), NULL),
('aa666666-6666-4666-8666-666666666666', 'privacy_policy', '2026-04-01', 'en-GB', '2222222222222222222222222222222222222222222222222222222222222222', NOW(), NULL),
('aa777777-7777-4777-8777-777777777777', 'cookie_policy', '2026-04-01', 'en-GB', '3333333333333333333333333333333333333333333333333333333333333333', NOW(), NULL)
ON CONFLICT (id) DO NOTHING;

INSERT INTO processing_purpose (id, code, name, lawful_basis, consent_required, retention_policy_id, description, created_at) VALUES
('aa888888-8888-4888-8888-888888888888', 'terms_of_service', 'Terms of Service Acceptance', 'CONTRACT', TRUE, NULL, 'Tracks acceptance of the terms of service notice.', NOW()),
('aa999999-9999-4999-8999-999999999999', 'privacy_policy', 'Privacy Policy Acceptance', 'LEGAL_OBLIGATION', TRUE, NULL, 'Tracks acceptance of the privacy policy notice.', NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO account (id, public_handle, account_status, created_at, updated_at, deleted_at, last_active_at) VALUES
('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'demo-user', 'active', NOW(), NOW(), NULL, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO account_profile (account_id, display_name, locale, timezone_name, preferred_currency_code, profile_bio, created_at, updated_at) VALUES
('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'Alex Pricewatch', 'en-GB', 'Europe/London', 'GBP', NULL, NOW(), NOW())
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO account_email (
    id, account_id, email, normalized_email, email_role, is_login_enabled,
    is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at
) VALUES (
    'e1111111-1111-4111-8111-111111111111', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
    'alex@example.com', 'alex@example.com', 'PRIMARY', TRUE,
    TRUE, NOW(), 'seed', NOW(), NOW(), NULL
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO account_phone (
    id, account_id, e164_phone_number, extension, phone_role, is_sms_enabled, is_voice_enabled,
    is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at
) VALUES (
    'ee111111-1111-4111-8111-111111111111', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
    '+447700900123', NULL, 'PRIMARY', TRUE, TRUE,
    TRUE, NOW(), 'seed', NOW(), NOW(), NULL
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO data_source (id, source_type, account_id, source_name, trust_score, is_verified, created_at) VALUES
('aa121212-1212-4212-8212-121212121212', 'USER_SUBMISSION', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'Alex Pricewatch', 75.00, FALSE, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO item (
    id, category_id, canonical_name, normalized_name, specification_text, description, status,
    created_by_account_id, approved_by_account_id, created_at, updated_at
) VALUES (
    '11111111-1111-4111-8111-111111111111', 'c1111111-1111-4111-8111-111111111111',
    'Milk', 'milk', 'Semi-skimmed', 'Standard semi-skimmed milk.', 'approved',
    'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', NOW(), NOW()
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO item_variant (
    id, item_id, brand_id, variant_name, package_quantity, package_unit_id, pack_count,
    normalized_content_quantity, normalized_content_unit_id, status, created_by_account_id,
    approved_by_account_id, created_at, updated_at
) VALUES (
    '22222222-2222-4222-8222-222222222222', '11111111-1111-4111-8111-111111111111',
    'b1111111-1111-4111-8111-111111111111', '1L Carton', 1.000000, 'aa222222-2222-4222-8222-222222222222', 1,
    1.000000, 'aa222222-2222-4222-8222-222222222222', 'approved', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
    'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', NOW(), NOW()
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO variant_identifier (
    id, item_variant_id, identifier_type, scope_type, retailer_id, shop_id,
    identifier_value, is_primary, valid_from, valid_to, created_at
) VALUES (
    'aa131313-1313-4313-8313-131313131313', '22222222-2222-4222-8222-222222222222',
    'gtin', 'GLOBAL', NULL, NULL, '5012345678901', TRUE, NULL, NULL, NOW()
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO shop_listing (
    id, shop_id, item_variant_id, external_sku, listing_url, first_seen_at, last_seen_at, is_active, created_at
) VALUES (
    'aa141414-1414-4414-8414-141414141414', '33333333-3333-4333-8333-333333333333',
    '22222222-2222-4222-8222-222222222222', 'TESCO-MILK-1L', 'https://example.com/items/milk-1l',
    NOW(), NOW(), TRUE, NOW()
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO consent_record (
    id, account_id, anonymous_subject_token_hash, processing_purpose_id, notice_version_id,
    consent_status, captured_via, evidence_json, captured_at, withdrawn_at
) VALUES
(
    'aa151515-1515-4515-8515-151515151515', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', NULL,
    'aa888888-8888-4888-8888-888888888888', 'aa555555-5555-4555-8555-555555555555',
    'accepted', 'seed', '{"source":"seed"}'::jsonb, NOW(), NULL
),
(
    'aa161616-1616-4616-8616-161616161616', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', NULL,
    'aa999999-9999-4999-8999-999999999999', 'aa666666-6666-4666-8666-666666666666',
    'accepted', 'seed', '{"source":"seed"}'::jsonb, NOW(), NULL
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO cookie_consent (
    id, account_id, anonymous_subject_token_hash, notice_version_id,
    preferences_allowed, analytics_allowed, marketing_allowed, captured_at, updated_at, withdrawn_at
) VALUES (
    'aa171717-1717-4717-8717-171717171717', 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', NULL,
    'aa777777-7777-4777-8777-777777777777',
    TRUE, TRUE, FALSE, NOW(), NOW(), NULL
)
ON CONFLICT (id) DO NOTHING;

COMMIT;
