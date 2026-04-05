BEGIN;

INSERT INTO "role" (id, code, name, created_at) VALUES
('f1000000-0000-4000-8000-000000000001', 'user', 'User', NOW()),
('f1000000-0000-4000-8000-000000000002', 'admin', 'Administrator', NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO permission (id, code, name, description, created_at) VALUES
('f2000000-0000-4000-8000-000000000001', 'catalog:read', 'Read catalog', 'View public catalog data.', NOW()),
('f2000000-0000-4000-8000-000000000002', 'price:read_public', 'Read public prices', 'View published price data.', NOW()),
('f2000000-0000-4000-8000-000000000003', 'price:write_own', 'Write own prices', 'Create and manage your own price submissions.', NOW()),
('f2000000-0000-4000-8000-000000000004', 'purchase:write_own', 'Write own purchases', 'Create and manage your own purchases.', NOW()),
('f2000000-0000-4000-8000-000000000005', 'admin:dashboard', 'Use admin dashboard', 'Access the admin dashboard.', NOW()),
('f2000000-0000-4000-8000-000000000006', 'catalog:write', 'Manage catalog', 'Create and edit curated catalog data.', NOW()),
('f2000000-0000-4000-8000-000000000007', 'moderation:write', 'Moderate prices', 'Review and moderate submitted prices.', NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO role_permission (role_id, permission_id, granted_at) VALUES
('f1000000-0000-4000-8000-000000000001', 'f2000000-0000-4000-8000-000000000001', NOW()),
('f1000000-0000-4000-8000-000000000001', 'f2000000-0000-4000-8000-000000000002', NOW()),
('f1000000-0000-4000-8000-000000000001', 'f2000000-0000-4000-8000-000000000003', NOW()),
('f1000000-0000-4000-8000-000000000001', 'f2000000-0000-4000-8000-000000000004', NOW()),
('f1000000-0000-4000-8000-000000000002', 'f2000000-0000-4000-8000-000000000001', NOW()),
('f1000000-0000-4000-8000-000000000002', 'f2000000-0000-4000-8000-000000000002', NOW()),
('f1000000-0000-4000-8000-000000000002', 'f2000000-0000-4000-8000-000000000003', NOW()),
('f1000000-0000-4000-8000-000000000002', 'f2000000-0000-4000-8000-000000000004', NOW()),
('f1000000-0000-4000-8000-000000000002', 'f2000000-0000-4000-8000-000000000005', NOW()),
('f1000000-0000-4000-8000-000000000002', 'f2000000-0000-4000-8000-000000000006', NOW()),
('f1000000-0000-4000-8000-000000000002', 'f2000000-0000-4000-8000-000000000007', NOW())
ON CONFLICT (role_id, permission_id) DO NOTHING;

INSERT INTO account (id, public_handle, account_status, created_at, updated_at, deleted_at, last_active_at) VALUES
('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'admin-user', 'active', NOW(), NOW(), NULL, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO account_profile (account_id, display_name, locale, timezone_name, preferred_currency_code, profile_bio, created_at, updated_at) VALUES
('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'PriceTracker Admin', 'en-GB', 'Europe/London', 'GBP', 'Admin dashboard test account', NOW(), NOW())
ON CONFLICT (account_id) DO NOTHING;

INSERT INTO account_email (
    id, account_id, email, normalized_email, email_role, is_login_enabled,
    is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at
) VALUES (
    'f3000000-0000-4000-8000-000000000001', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    'admin@pricetracker.local', 'admin@pricetracker.local', 'PRIMARY', TRUE,
    TRUE, NOW(), 'seed', NOW(), NOW(), NULL
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO account_phone (
    id, account_id, e164_phone_number, extension, phone_role, is_sms_enabled, is_voice_enabled,
    is_primary_for_account, verified_at, verification_method, created_at, updated_at, deleted_at
) VALUES (
    'f3000000-0000-4000-8000-000000000002', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    '+447700900999', NULL, 'PRIMARY', TRUE, TRUE,
    TRUE, NOW(), 'seed', NOW(), NOW(), NULL
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO data_source (id, source_type, account_id, source_name, trust_score, is_verified, created_at) VALUES
('f3000000-0000-4000-8000-000000000003', 'USER_SUBMISSION', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'PriceTracker Admin', 100.00, TRUE, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO account_role (account_id, role_id, granted_by_account_id, granted_at) VALUES
('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', 'f1000000-0000-4000-8000-000000000001', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', NOW()),
('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'f1000000-0000-4000-8000-000000000001', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', NOW()),
('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', 'f1000000-0000-4000-8000-000000000002', 'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb', NOW())
ON CONFLICT (account_id, role_id) DO NOTHING;

COMMIT;
