BEGIN;

INSERT INTO setting_definition (
    id, setting_key, scope_type, value_type, default_value_json, is_sensitive, description, created_at
) VALUES
(
    'f4000000-0000-4000-8000-000000000001',
    'system.maintenanceMode',
    'SYSTEM',
    'boolean',
    'false'::jsonb,
    FALSE,
    'Temporarily disable contribution actions from the publish UI.',
    NOW()
),
(
    'f4000000-0000-4000-8000-000000000002',
    'catalog.allowUserRegistration',
    'SYSTEM',
    'boolean',
    'true'::jsonb,
    FALSE,
    'Control whether the standard user registration form is available.',
    NOW()
),
(
    'f4000000-0000-4000-8000-000000000003',
    'debug.hiddenModulesEnabled',
    'SYSTEM',
    'boolean',
    'true'::jsonb,
    FALSE,
    'Keep the hidden admin-only debug modules available behind the published UI.',
    NOW()
),
(
    'f4000000-0000-4000-8000-000000000004',
    'ui.publishBannerText',
    'SYSTEM',
    'string',
    '"Track prices, compare offers, and save receipts from one workspace."'::jsonb,
    FALSE,
    'Small publish-facing supporting copy used by the new frontend shell.',
    NOW()
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO system_setting (
    setting_definition_id, setting_value_json, updated_by_account_id, updated_at
) VALUES
(
    'f4000000-0000-4000-8000-000000000001',
    'false'::jsonb,
    'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    NOW()
),
(
    'f4000000-0000-4000-8000-000000000002',
    'true'::jsonb,
    'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    NOW()
),
(
    'f4000000-0000-4000-8000-000000000003',
    'true'::jsonb,
    'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    NOW()
),
(
    'f4000000-0000-4000-8000-000000000004',
    '"Track prices, compare offers, and save receipts from one workspace."'::jsonb,
    'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
    NOW()
)
ON CONFLICT (setting_definition_id) DO NOTHING;

COMMIT;
