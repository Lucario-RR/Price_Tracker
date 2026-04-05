# PriceTracker Implementation

This repository now includes:

- `backend/`: an Axum + SQLx backend aligned to the OpenAPI contract in `doc/pricetracker-openapi-3.1.yaml`
- `frontend/`: a Vue + Vite publish-style dashboard frontend with hidden admin debug modules

The backend is now wired to the v2 SQL schema files in `backend/migrations/`:

- `pricetracker_schema_v2.sql`
- `pricetracker_seed_v2.sql`
- `pricetracker_zz_admin_seed_v2.sql`
- `pricetracker_zzz_system_settings_v2.sql`

## Backend

1. Create a PostgreSQL database.
2. Copy `backend/.env.example` to `backend/.env` and update `DATABASE_URL`.
3. Run the API:

```bash
cd backend
cargo run
```

The backend applies the v2 SQL scripts on startup and seeds a demo account, an admin account, and starter catalog data.

Session behavior in the current dev build:

- login, register, admin bootstrap, and refresh now issue an `HttpOnly` session cookie for the demo session
- `POST /api/v1/auth/logout` expires that cookie immediately
- authenticated routes now require a valid session token or session cookie, and reject mismatched `x-account-id` headers instead of falling back to another account
- active accounts now store per-account password credentials, and `POST /api/v1/auth/password/change` rotates that password for the signed-in user
- suspended or deleted accounts are blocked from authenticated routes and from new sign-ins

Temporary admin bootstrap:

- `POST /api/v1/auth/register-admin` allows creating an admin account with email/password and no prior login
- this is controlled by `ALLOW_PUBLIC_ADMIN_BOOTSTRAP`
- the current default is `true` so local setup works immediately
- set it to `false` or remove the route before a stricter production release

Demo login:

- email: `alex@example.com`
- password: `StrongPassword!234`

Admin login:

- email: `admin@pricetracker.local`
- password: `StrongPassword!234`

## Frontend

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173`.

The Vue frontend now includes:

- a real home page that opens into a dashboard-style application shell
- a left-side menu that can stay pinned open to keep navigation visible while shrinking the main content area
- a dedicated login/register page reached from the top-right auth section
- user-only dashboard areas for capture, queue recovery, watchlists, alerts, recent activity, and self-service account management
- self-service profile controls for display name, locale, timezone, bio, password, email, phone, cookie preferences, and avatar attachment
- profile editors now constrain preferred currency choices to the seeded supported values so invalid codes do not bounce back as backend failures
- a separate lazy-loaded admin portal so guest and user sessions do not load admin UI code
- admin-only modules for dashboard stats, user management, approvals, catalog tables, settings, and a separate debug area
- searchable relation selectors inside admin catalog forms, so foreign-key fields can be chosen by name instead of typing raw UUIDs
- approval-aware catalog modules now expose per-record approve actions plus an `Everything / Awaiting approval` switch at the top of the table
- admin table modules now surface backend load failures separately from true empty-table states
- auto-generated primary IDs in admin create flows, with base units able to create or reuse a unit family automatically
- separate status lights for browser connectivity, backend reachability, and database connectivity
- live barcode scanning with camera access
- fallback image-based barcode recognition
- manual code entry
- persistent local draft state
- an offline retry queue for safe resubmission when connectivity returns
- a hidden admin API explorer for the full backend route surface
- request-state feedback banners so sign-in, registration, profile saves, alerts, and admin setting updates show sending/success/failure states
- a full sign-out flow that clears app-owned local storage, retry queue data, response history, and in-memory user state after asking the backend to expire its cookie

Debug tooling is no longer part of the main published flow. It now lives in its own dedicated section inside the separate admin portal.

For builds:

- `npm run build` creates the publish-focused build
- `npm run build:debug` keeps the older explicit debug mode available if you want to ship a more openly diagnostic build

## Notes

- The core catalog, shop, file, purchase, price, watchlist, alert, moderation, privacy-consent, and profile flows are wired to PostgreSQL using the v2 schema.
- Admin settings are now backed by `setting_definition` + `system_setting`, so the frontend can edit real persisted system controls instead of mock values.
- Admin user management now uses `account`, `account_profile`, `account_role`, `account_suspension`, and avatar `file_attachment` records to create, freeze, restore, delete, and review accounts from the frontend.
- Sign-out is intentionally destructive for client-owned cached data so the next guest or user session cannot inherit queued writes or stale private responses from the previous account.
- Password change is now implemented for the current dev flow, while the deeper passkey and MFA endpoints remain scaffolded for later extension.
- Cargo build output is redirected outside OneDrive in `backend/.cargo/config.toml` to avoid the Windows file-lock issues that were breaking local builds.
