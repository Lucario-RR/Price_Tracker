# PriceTracker Project Plan v2 (Greenfield)

## Why this plan changed

The project is still greenfield, so there is no migration phase.  
The baseline architecture is now broader than a simple price-tracking backend:

- identity must support **multiple emails**, **multiple phone numbers**, **passkeys**, **2FA**, and **recovery codes**
- password handling must support **salted hashing** and **password history enforcement**
- the file layer must support **multiple files per business entry**, especially multiple receipt images for one purchase
- product code lookup must treat barcodes as **shop-scoped product codes** when needed, not assume one universal international barcode
- privacy/compliance needs first-class APIs for **legal document acceptance** and **cookie preferences**

Because of that, the plan moves security/privacy earlier and treats user/file modules as reusable platform components.

---

## Updated target architecture

### Core backend modules

1. **Identity & Session**
   - registration
   - password login
   - passkey login
   - refresh/session management
   - MFA challenge/verification
   - password reset/change
   - password history enforcement

2. **User Profile & Contact Points**
   - user profile
   - multiple emails
   - multiple phone numbers
   - verification flows
   - primary contact selection

3. **Privacy & Compliance**
   - current legal document versions
   - acceptance records
   - cookie preferences
   - audit trail for consent changes

4. **File Service**
   - signed upload intents
   - malware scanning / quarantine
   - private download
   - reusable attachment model
   - support multiple files linked to one purchase

5. **Catalog & Shop**
   - items / variants / brands / categories
   - shops / addresses
   - shop-scoped product codes
   - unit normalization support

6. **Purchase & Price Contribution**
   - purchase records with multi-file attachments
   - price submissions
   - edit/delete own records
   - suspicious value detection hooks

7. **Read Models / Shopper Features**
   - public search
   - comparison
   - price history
   - cheapest shop
   - watchlist / alerts
   - basket comparison

8. **Admin & Moderation**
   - moderation queue
   - duplicate merge
   - config management
   - abuse/security events

---

## Delivery sequence

### Phase 0 — Contracts and foundation decisions

**Goal:** lock the external contract before coding.

Deliverables:
- OpenAPI 3.1 spec v2
- finalized resource naming
- error code catalogue
- auth/session strategy
- privacy/consent rules
- shop product code rules

Decisions to freeze:
- UUID public IDs
- decimal strings for money
- private file upload via signed URL
- purchase attachments as an array, not a single receipt field
- `primaryEmail` + separate `/me/emails`
- `primaryPhone` + separate `/me/phones`
- passkey and TOTP both supported
- legal version acceptance required during registration

### Phase 1 — Reusable identity/security platform

**Goal:** build the reusable account layer first.

Scope:
- register / login / refresh / logout
- password change / forgot / reset
- password history check
- Argon2id password hashing with unique salt per password version
- multiple email management
- multiple phone management
- email / phone verification
- passkey registration and authentication
- TOTP setup / enable / disable
- recovery code generation and rotation
- account security summary

Why first:
- all protected app features depend on this
- privacy/compliance and audit behavior sit here
- this module can be reused in future projects

### Phase 2 — Privacy, consent, and cookie controls

**Goal:** make compliance part of the platform, not a later patch.

Scope:
- current legal document API
- consent acceptance recording
- consent history lookup
- cookie preference read/write
- policy version checks during registration and sign-in

Outputs:
- legal document storage model
- consent audit log
- frontend registration/settings flows for consent

### Phase 3 — File service and attachment model

**Goal:** build the generic private-file pipeline.

Scope:
- upload intent creation
- direct object-store upload
- upload completion callback
- scan/quarantine flow
- download URL generation
- file purpose classification
- attachment linkage for purchases

Important rule:
- file records are reusable and private by default
- one purchase may link to many files

### Phase 4 — Catalog, shops, and product-code lookup

**Goal:** build searchability and in-store identification.

Scope:
- categories / brands / units
- items / variants
- shops / addresses
- shop product code lookup endpoint
- unit normalization metadata
- deduplication hooks

Important rule:
- product code resolution must allow:
  - global codes
  - shop-scoped merchant barcodes
  - PLU / internal shelf codes

### Phase 5 — Purchase and price contribution

**Goal:** support trusted user contribution.

Scope:
- create/edit/delete purchases
- attach multiple receipt images/PDFs
- create/edit/delete price submissions
- duplicate submission suppression
- suspicious price flagging hooks
- contributor ownership checks

Important rule:
- client never sends internal source identity
- source/user attribution is derived from session

### Phase 6 — Public read APIs and shopper features

**Goal:** deliver the user-visible value.

Scope:
- item search
- item variant detail
- price comparison
- price history
- cheapest shop
- compare endpoint
- watchlist
- alerts

### Phase 7 — Admin, moderation, and operations

**Goal:** keep data quality high and system abuse low.

Scope:
- moderation queue
- approve/reject prices
- verify receipts
- merge duplicate items/variants
- shop verification
- config for units/categories/discount types
- user banning / abuse handling
- security event monitoring

### Phase 8 — Advanced features (after MVP)

Optional later scope:
- trust score
- community voting/reporting
- basket optimization
- spending insights
- price prediction
- regional analytics

---

## MVP recommendation

For the first usable release, include:

- identity/security platform
- privacy/cookie controls
- file upload + multi-attachment purchases
- catalog / shop / product-code lookup
- price submission
- public comparison / history
- basic admin moderation

Do **not** wait for trust score, prediction, or community voting before launch.

---

## Rust backend recommendation

Recommended stack:
- **axum** for HTTP layer
- **utoipa** for OpenAPI generation/validation alignment
- **sqlx** for DB access
- **serde** for DTO serialization
- **validator** for request validation
- **argon2** for password hashing
- **webauthn-rs** for passkeys
- **totp-rs** for TOTP MFA
- **tower-cookies** or session middleware for cookie handling
- **tracing** for structured audit/security logs

Suggested crate split:

```text
/apps/api
/crates/contracts
/crates/domain
/crates/application
/crates/infra-db
/crates/infra-auth
/crates/infra-files
/crates/infra-notify
```

Why this split:
- contracts stay aligned with OpenAPI
- auth/files stay reusable across projects
- domain stays clean from transport details

---

## Priority changes versus the older plan

1. **Security moved earlier**
   - passkeys, MFA, recovery codes, and password lifecycle are now phase 1, not later polish

2. **Privacy moved earlier**
   - cookie and consent APIs are part of MVP, not an afterthought

3. **File model generalized**
   - no single `receiptFileId` assumption
   - attachment arrays supported from day one

4. **Barcode model corrected**
   - product codes can be shop-specific

5. **General user/file template emphasized**
   - user/contact/security/file modules should be designed for reuse beyond PriceTracker

---

## Immediate next implementation documents

After this plan, the best next documents are:

1. DB schema v2 aligned with the new API
2. auth/security decision record
3. file storage and scanning flow
4. Rust workspace/module layout
5. frontend flow notes for:
   - register + legal consent
   - add phone/email + verify
   - passkey enrollment
   - TOTP setup
   - multi-photo receipt upload
   - shop barcode lookup
