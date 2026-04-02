# PriceTracker API Dev Sheet

**Version:** v1 draft
**Style:** REST + JSON
**Audience:** frontend, backend, and security reviewers

## 1. Service boundary assumed from the current design

The API should follow the same split implied by the frontend/backend architecture and the newer module list: public catalog/search/comparison on the read side, authenticated contribution flows for purchases and prices, file upload for receipts, and restricted admin/moderation/config endpoints. Unit normalization, duplicate prevention, trust/validation, receipt handling, and abuse protection are first-class concerns. 

Recommended backend service grouping behind one API gateway or BFF:

* **Auth/User service**: login, registration, profile, session, scopes
* **Catalog service**: items, item variants, categories, brands, units
* **Shop service**: shops and public shop display info
* **Price service**: price submissions, history, public price read models
* **Purchase service**: user-owned purchase records linked to receipts
* **Comparison/Analytics service**: compare, history, cheapest shop, basket optimization
* **File service**: signed upload/download, malware scanning, quarantine
* **Moderation/Admin service**: merge duplicates, approve suggestions, validate suspicious prices

## 2. Public API naming and versioning

Use a clean external contract even if the database keeps older names such as `ItemVarient`. The external API should normalize names like this:

| DB / legacy concept | External API resource |
| ------------------- | --------------------- |
| `Item`              | `items`               |
| `ItemVarient`       | `item-variants`       |
| `PriceRecord`       | `prices`              |
| `PurchaseRecord`    | `purchases`           |
| `Category`          | `categories`          |
| `Brand`             | `brands`              |
| `Shop`              | `shops`               |
| `Unit`              | `units`               |
| `DiscountType`      | `discount-types`      |
| `Address`           | `addresses`           |
| `File`              | `files`               |

The uploaded schema and rough guide already group the system around these domains, so this keeps the same structure while modernizing the public contract.  

Base URL:

```text
https://api.example.com/api/v1
```

Conventions:

* JSON request and response bodies
* `camelCase` for JSON fields
* `kebab-case` for path names
* UUID or ULID for public IDs
* ISO 8601 UTC timestamps
* ISO 4217 alpha-3 for currency, such as `GBP`
* cursor pagination for feeds/history
* `PATCH` for partial update
* soft delete for user/admin data where auditability matters

## 3. Access model

The recent generated behavior clearly separates guest browsing from user contribution and admin moderation. 

| Role            | Read                                                       | Write                                                            |
| --------------- | ---------------------------------------------------------- | ---------------------------------------------------------------- |
| Guest           | items, variants, shops, public comparison, public history  | none                                                             |
| User            | guest reads + own purchases, own prices, watchlist, alerts | create/edit own submissions, upload receipts, create suggestions |
| Moderator/Admin | all user data as permitted + moderation views + config     | approve/reject/merge/edit/configure                              |

Recommended scopes:

```text
catalog:read
shop:read
price:read_public
price:write_own
purchase:read_own
purchase:write_own
file:write_own
watchlist:write_own
admin:moderate
admin:config
admin:security
```

## 4. Sensitive data policy

Your schema includes fields that should never be treated as public by default, such as `VATID`, `FileID`, `SourceID`, free-text `Notes`, structured addresses, and product-level identifiers like `SN` and `BatchCode`. The current product flow also includes receipt uploads and trust/moderation, which means public and private DTOs must be separated.  

### Data classes

| Class      | Examples                                                                                                               | Guest-visible? | Notes                                |
| ---------- | ---------------------------------------------------------------------------------------------------------------------- | -------------- | ------------------------------------ |
| Public     | item name, specification, shop display info, normalized price, price history aggregates                                | yes            | cacheable                            |
| Protected  | my purchases, my receipt file refs, my watchlist, my notes                                                             | no             | owner or admin only                  |
| Restricted | auth credentials, refresh tokens, raw receipts, VAT/tax IDs, source identity, IP/device fingerprints, moderation notes | no             | encrypted, audited, redacted in logs |

### Hard rules

* Never expose `SourceID` on public price endpoints.
* Never expose `PurchaseID`, `VATID`, raw receipt URLs, `SN`, or `BatchCode` to guests.
* Do not return raw receipt files from public APIs.
* Do not log passwords, tokens, receipt URLs, or full note bodies.
* Do not place sensitive values in query strings.
* Do not accept client-supplied `SourceID`; derive it from the authenticated session.
* Treat user notes as untrusted text; sanitize and optionally redact PII before any public display.

## 5. Authentication and session design

For a frontend/backend split, the safest practical pattern is:

* OIDC / OAuth 2.1 style login
* short-lived access token
* refresh token in `HttpOnly`, `Secure`, `SameSite` cookie
* access token kept in memory, not persistent browser storage
* CSRF token required if the API relies on cookies

Minimum auth endpoints:

| Method | Path                    | Auth   | Purpose                                      |
| ------ | ----------------------- | ------ | -------------------------------------------- |
| POST   | `/auth/register`        | public | create account                               |
| POST   | `/auth/login`           | public | login with email/password or federated token |
| POST   | `/auth/refresh`         | cookie | refresh access token                         |
| POST   | `/auth/logout`          | user   | invalidate session/refresh token             |
| POST   | `/auth/password/forgot` | public | reset flow                                   |
| POST   | `/auth/password/reset`  | public | complete reset                               |
| GET    | `/me`                   | user   | current profile and scopes                   |
| PATCH  | `/me`                   | user   | update profile/preferences                   |

Security requirements:

* Password hashing with Argon2id or bcrypt with modern cost
* lockout/backoff on repeated failures
* MFA-ready design for admin accounts
* HSTS + TLS only
* no credentials in URL parameters

## 6. Common request and response contract

### Headers

```http
Authorization: Bearer <access-token>
Content-Type: application/json
Accept: application/json
X-Request-Id: <uuid>
Idempotency-Key: <uuid>   // required for critical POST writes
If-Match: "<etag>"        // for optimistic concurrency on PATCH
X-CSRF-Token: <token>     // when cookie session mode is used
```

### Success shape

```json
{
  "data": {},
  "meta": {
    "requestId": "req_01H..."
  }
}
```

### Error shape

```json
{
  "error": {
    "code": "PRICE_OUT_OF_RANGE",
    "message": "Submitted price is outside allowed bounds.",
    "details": {
      "field": "originalAmount"
    },
    "requestId": "req_01H..."
  }
}
```

Recommended error codes:

`VALIDATION_ERROR`, `UNAUTHORIZED`, `FORBIDDEN`, `NOT_FOUND`, `CONFLICT`, `RATE_LIMITED`, `FILE_SCAN_PENDING`, `FILE_REJECTED`, `DUPLICATE_SUBMISSION`, `MODERATION_REQUIRED`

## 7. Resource models

The current schema centers on item/catalog, item variants with qty and unit, purchase records linked to shops and receipt files, and price records with original amount, discounts, currency, timestamp, and source metadata. 

### Item DTO

```json
{
  "id": "itm_...",
  "categoryId": "cat_...",
  "name": "Milk",
  "specification": "Semi-skimmed",
  "notes": null,
  "createdAt": "2026-04-02T15:00:00Z"
}
```

### ItemVariant DTO

```json
{
  "id": "iv_...",
  "itemId": "itm_...",
  "brandId": "br_...",
  "qty": 1.0,
  "unitId": "unit_l",
  "sku": "5012345678901",
  "website": "https://..."
}
```

### PublicPrice DTO

```json
{
  "itemVariantId": "iv_...",
  "shop": {
    "id": "shop_...",
    "name": "Tesco"
  },
  "price": {
    "originalAmount": 1.8,
    "currency": "GBP",
    "discountAmount": 0.2,
    "finalAmount": 1.6,
    "unitPrice": 1.6,
    "unitLabel": "GBP/L"
  },
  "recordedAt": "2026-04-01T18:20:00Z",
  "verification": "community"
}
```

### PrivatePurchase DTO

```json
{
  "id": "pur_...",
  "shopId": "shop_...",
  "purchaseTime": "2026-04-01T18:15:00Z",
  "receiptFileId": "file_...",
  "vatId": "private",
  "status": "active"
}
```

### Submission status

Use lifecycle state on user contributions:

```text
draft -> submitted -> flagged|verified|rejected -> published
```

This is better than publishing raw submitted records immediately, especially for price integrity and receipt-linked submissions.

## 8. Endpoint catalogue

### 8.1 Catalog

The latest product behavior depends on browse/search, variant selection, unit normalization, and duplicate prevention. 

| Method | Path                         | Auth  | Purpose                                  |
| ------ | ---------------------------- | ----- | ---------------------------------------- |
| GET    | `/items`                     | guest | search items by query/category/brand     |
| GET    | `/items/{itemId}`            | guest | item detail                              |
| GET    | `/items/{itemId}/variants`   | guest | list variants                            |
| GET    | `/item-variants/{variantId}` | guest | variant detail                           |
| GET    | `/categories`                | guest | category tree                            |
| GET    | `/brands`                    | guest | brand list/search                        |
| GET    | `/units`                     | guest | allowed units and normalization metadata |
| GET    | `/discount-types`            | guest | public display labels for discount types |
| POST   | `/item-suggestions`          | user  | suggest new item                         |
| POST   | `/item-variant-suggestions`  | user  | suggest new variant                      |
| POST   | `/brand-suggestions`         | user  | suggest new brand                        |

Recommended query parameters for `GET /items`:

```text
query
categoryId
brandId
unitId
cursor
limit
sort=name|relevance|latestPrice
```

Important: allowlist `sort` and `fields`; do not pass them directly into SQL.

### 8.2 Shops

The rough guide already exposes shop lookups and name-based search. Keep shops public, but do not expose raw internal address rows unless needed.  

| Method | Path                             | Auth  | Purpose                          |
| ------ | -------------------------------- | ----- | -------------------------------- |
| GET    | `/shops`                         | guest | list/search shops                |
| GET    | `/shops/{shopId}`                | guest | shop detail                      |
| GET    | `/shops/{shopId}/latest-prices`  | guest | recent public prices in the shop |
| GET    | `/shops/{shopId}/cheapest-items` | guest | cheapest current items           |
| POST   | `/shop-suggestions`              | user  | suggest a missing shop           |

Public shop response should expose a `displayAddress` string, not necessarily the full raw address object.

### 8.3 Public price read APIs

The product behavior emphasizes public comparison, price history, cheapest shop, and variant comparison. This should be the main guest-facing surface, instead of a raw open `GET /PriceRecord` style list. 

| Method | Path                                       | Auth  | Purpose                                   |
| ------ | ------------------------------------------ | ----- | ----------------------------------------- |
| GET    | `/item-variants/{variantId}/prices`        | guest | latest public prices for one variant      |
| GET    | `/item-variants/{variantId}/price-history` | guest | time series history                       |
| GET    | `/items/{itemId}/compare-variants`         | guest | compare 500ml vs 1L vs 2L                 |
| GET    | `/compare`                                 | guest | compare selected variants/items           |
| GET    | `/cheapest`                                | guest | cheapest current shop for item or variant |

Recommended `GET /compare` for simple public compare:

```text
/items?query=milk
/compare?variantIds=iv_1,iv_2,iv_3
```

For larger or user-specific compares, use a POST body:

| Method | Path                  | Auth       | Purpose                 |
| ------ | --------------------- | ---------- | ----------------------- |
| POST   | `/comparisons`        | guest/user | complex compare payload |
| POST   | `/basket-comparisons` | user       | basket optimization     |

Use POST when the request would otherwise leak too much detail into logs or exceed safe URL length.

### 8.4 Purchase and price submission APIs

The current product flow explicitly includes adding purchase records, linking receipts, adding price records, and editing/deleting own data. 

| Method | Path                         | Auth | Purpose                               |
| ------ | ---------------------------- | ---- | ------------------------------------- |
| POST   | `/purchases`                 | user | create purchase record                |
| GET    | `/me/purchases`              | user | list my purchases                     |
| GET    | `/me/purchases/{purchaseId}` | user | get one of my purchases               |
| PATCH  | `/me/purchases/{purchaseId}` | user | edit my purchase                      |
| DELETE | `/me/purchases/{purchaseId}` | user | soft delete my purchase               |
| POST   | `/prices`                    | user | submit a price record                 |
| GET    | `/me/prices`                 | user | list my price submissions             |
| GET    | `/me/prices/{priceId}`       | user | get one of my submissions             |
| PATCH  | `/me/prices/{priceId}`       | user | edit own price before lock/moderation |
| DELETE | `/me/prices/{priceId}`       | user | soft delete own price                 |

Important write-side rules:

* client must not send `SourceId`; server injects it from session
* `purchaseId` must belong to the same user
* `discountCurrency` should match `originalCurrency` unless cross-currency support is explicitly implemented
* `notes` length-limited and sanitized
* duplicate-submission detection by user + variant + shop + time window + amount

### 8.5 Files / receipts

Receipt upload is sensitive and should not use direct raw file passthrough via the main application server unless unavoidable. The safer pattern is signed upload to object storage, then async scan/quarantine. The generated product notes explicitly include receipt upload and validation. 

| Method | Path                               | Auth | Purpose                                 |
| ------ | ---------------------------------- | ---- | --------------------------------------- |
| POST   | `/files/uploads`                   | user | create upload intent and get signed URL |
| POST   | `/files/uploads/{fileId}/complete` | user | finalize upload after object-store PUT  |
| GET    | `/me/files/{fileId}`               | user | get metadata/status for own file        |
| GET    | `/me/files/{fileId}/download`      | user | get short-lived signed download URL     |
| DELETE | `/me/files/{fileId}`               | user | mark own unused file for deletion       |

Required file controls:

* allow only image/PDF MIME types
* max file size
* malware scan
* image re-encode to strip risky metadata
* EXIF removal by default
* quarantine until scan passes
* signed URLs expire quickly
* public endpoints never expose raw receipt URLs

### 8.6 Watchlist and alerts

The recent generated scope includes watchlists and price alerts. 

| Method | Path                            | Auth | Purpose                     |
| ------ | ------------------------------- | ---- | --------------------------- |
| GET    | `/me/watchlist`                 | user | list tracked items/variants |
| POST   | `/me/watchlist/items`           | user | add tracked item/variant    |
| DELETE | `/me/watchlist/items/{watchId}` | user | remove tracked item/variant |
| GET    | `/me/alerts`                    | user | list alerts                 |
| POST   | `/me/alerts`                    | user | create alert                |
| PATCH  | `/me/alerts/{alertId}`          | user | edit alert                  |
| DELETE | `/me/alerts/{alertId}`          | user | delete alert                |

### 8.7 Admin and moderation

The generated admin scope includes approval/rejection, duplicate merge, suspicious price review, shop verification, unit/category/discount configuration, and abuse/security controls. 

| Method | Path                                        | Auth  | Purpose                                |
| ------ | ------------------------------------------- | ----- | -------------------------------------- |
| GET    | `/admin/moderation/prices`                  | admin | queue of flagged/submitted prices      |
| POST   | `/admin/moderation/prices/{priceId}/verify` | admin | verify and publish                     |
| POST   | `/admin/moderation/prices/{priceId}/reject` | admin | reject submission                      |
| GET    | `/admin/moderation/items`                   | admin | pending item/variant/brand suggestions |
| POST   | `/admin/items/{itemId}/merge`               | admin | merge duplicates                       |
| POST   | `/admin/shops/{shopId}/verify`              | admin | verify shop legitimacy                 |
| GET    | `/admin/config/units`                       | admin | config view                            |
| PATCH  | `/admin/config/units/{unitId}`              | admin | edit conversion/unit config            |
| GET    | `/admin/config/categories`                  | admin | category config                        |
| GET    | `/admin/config/discount-types`              | admin | discount type config                   |
| GET    | `/admin/security/events`                    | admin | abuse/security events                  |
| POST   | `/admin/users/{userId}/ban`                 | admin | ban or restrict abusive user           |

Admin writes must always create audit entries.

## 9. Example critical flows

### 9.1 Public item search

```http
GET /api/v1/items?query=milk&categoryId=dairy&limit=20
```

```json
{
  "data": [
    {
      "id": "itm_milk_001",
      "name": "Milk",
      "specification": "Semi-skimmed",
      "variantSummary": {
        "count": 3,
        "lowestKnownPrice": {
          "amount": 1.6,
          "currency": "GBP"
        }
      }
    }
  ],
  "meta": {
    "nextCursor": "cur_..."
  }
}
```

### 9.2 Receipt upload + purchase + price submit

**1. Create upload intent**

```http
POST /api/v1/files/uploads
```

```json
{
  "filename": "receipt.jpg",
  "contentType": "image/jpeg",
  "size": 231231
}
```

Response:

```json
{
  "data": {
    "fileId": "file_123",
    "uploadUrl": "https://object-store/...signed...",
    "expiresAt": "2026-04-02T17:30:00Z"
  }
}
```

**2. Client uploads directly to object storage**

```http
PUT <signed upload url>
```

**3. Finalize**

```http
POST /api/v1/files/uploads/file_123/complete
```

**4. Create purchase**

```http
POST /api/v1/purchases
```

```json
{
  "shopId": "shop_456",
  "purchaseTime": "2026-04-02T16:55:00Z",
  "receiptFileId": "file_123"
}
```

**5. Submit price**

```http
POST /api/v1/prices
```

```json
{
  "itemVariantId": "iv_789",
  "purchaseId": "pur_321",
  "originalAmount": 1.99,
  "originalCurrency": "GBP",
  "discountAmount": 0.3,
  "discountCurrency": "GBP",
  "discountTypeId": "disc_member",
  "recordedAt": "2026-04-02T16:55:00Z",
  "notes": "yellow sticker"
}
```

Response:

```json
{
  "data": {
    "id": "price_111",
    "submissionStatus": "submitted",
    "visibility": "private",
    "published": false
  }
}
```

### 9.3 Price history

```http
GET /api/v1/item-variants/iv_789/price-history?range=90d&shopId=shop_456
```

```json
{
  "data": {
    "itemVariantId": "iv_789",
    "currency": "GBP",
    "unitLabel": "GBP/L",
    "points": [
      { "recordedAt": "2026-01-05T00:00:00Z", "finalAmount": 1.85, "unitPrice": 1.85 },
      { "recordedAt": "2026-02-05T00:00:00Z", "finalAmount": 1.95, "unitPrice": 1.95 }
    ]
  }
}
```

## 10. Security controls

The generated project notes already call out SQL injection risk, rate limiting, trust/validation, suspicious price detection, duplicate prevention, and security monitoring. Those need to be explicit API requirements, not just backend hopes. 

### Input handling

* validate every request with schema validation
* reject unknown fields on write endpoints
* use parameterized queries or ORM bind variables only
* allowlist sortable/filterable fields
* full-text search input length limits
* sanitize free text to plain text or escaped safe text
* no raw SQL fragments from `fields`, `sort`, or filter params

### Authorization

* scope-based access checks at controller and service layer
* object ownership checks for `/me/*`
* admin endpoints isolated under `/admin/*`
* service-to-service auth with mTLS or signed internal tokens
* default deny on new routes

### Anti-abuse

* per-IP and per-user rate limits
* stricter limits on auth, search spam, and write flows
* CAPTCHA or challenge step for new or low-trust users on repeated writes
* anomaly detection on mass submissions
* duplicate submission suppression
* reputation/trust score can affect moderation thresholds

### File security

* signed upload/download URLs
* AV scan and MIME verification
* do not trust client-supplied file extension
* strip metadata where possible
* image/PDF only unless future policy expands
* private bucket by default
* retention policy for unlinked/orphaned files

### Logging and audit

* always attach `requestId`
* audit all admin actions and all data-changing operations
* redact tokens, passwords, VAT IDs, and raw notes in logs
* do not log signed receipt URLs
* keep moderation decision audit trail

### Privacy and data minimization

* return only public display fields on guest endpoints
* use separate public/private serializers
* store exact raw receipt access only when necessary
* permit user deletion or anonymization flows where legally required
* define retention windows for rejected uploads and soft-deleted submissions

## 11. Changes I would make versus the old API guide

The uploaded `API_Guide.md` is useful as a domain inventory, but I would not keep it as the final public contract. 

Recommended changes:

* do **not** expose raw public `GET /PriceTracker/PriceRecord`
* do **not** expose `PurchaseRecord` to guests at all
* standardize `ItemVarient` to `item-variants` externally
* split public read models from raw transactional records
* move new item/brand/variant creation for normal users into **suggestion** endpoints
* use signed file upload flow instead of generic file POST
* keep auth/user/admin/file as separate tagged API groups, but version everything under `/api/v1`
* prefer curated public endpoints like `/compare`, `/cheapest`, `/price-history` over table-style CRUD for everything

## 12. Suggested starting OpenAPI tags

```text
Auth
Users
Catalog
ItemVariants
Shops
Prices
Purchases
Files
Comparisons
Watchlists
Alerts
AdminModeration
AdminConfig
AdminSecurity
```
