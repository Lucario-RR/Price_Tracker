# PriceTracker Implementation

This repository now includes:

- `backend/`: an Axum + SQLx backend aligned to the OpenAPI contract in `doc/pricetracker-openapi-3.1.yaml`
- `frontend/`: a small Vue + Vite app for manual API testing

## Backend

1. Create a PostgreSQL database.
2. Copy `backend/.env.example` to `backend/.env` and update `DATABASE_URL`.
3. Run the API:

```bash
cd backend
cargo run
```

The backend applies the SQL migration on startup and seeds a demo account plus starter catalog data.

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

- The core catalog, shop, file, purchase, price, watchlist, alert, moderation, privacy-consent, and profile flows are wired to PostgreSQL.
- The deeper identity-security endpoints in the YAML are scaffolded with placeholder responses so the route surface exists and can be extended into full password history, passkey, and MFA implementations later.
