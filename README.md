# PriceTracker Implementation

This repository now includes:

- `backend/`: an Axum + SQLx backend aligned to the OpenAPI contract in `doc/pricetracker-openapi-3.1.yaml`
- `frontend/`: a small Vue + Vite app for manual API testing

The backend is now wired to the v2 SQL schema files in `backend/migrations/`:

- `pricetracker_schema_v2.sql`
- `pricetracker_seed_v2.sql`

## Backend

1. Create a PostgreSQL database.
2. Copy `backend/.env.example` to `backend/.env` and update `DATABASE_URL`.
3. Run the API:

```bash
cd backend
cargo run
```

The backend applies the v2 SQL scripts on startup and seeds a demo account plus starter catalog data.

Demo login:

- email: `alex@example.com`
- password: `StrongPassword!234`

## Frontend

```bash
cd frontend
npm install
npm run dev
```

Open `http://localhost:5173`.

## Notes

- The core catalog, shop, file, purchase, price, watchlist, alert, moderation, privacy-consent, and profile flows are wired to PostgreSQL using the v2 schema.
- The deeper identity-security endpoints in the YAML are scaffolded with placeholder responses so the route surface exists and can be extended into full password history, passkey, and MFA implementations later.
- Cargo build output is redirected outside OneDrive in `backend/.cargo/config.toml` to avoid the Windows file-lock issues that were breaking local builds.
