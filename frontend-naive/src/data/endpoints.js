export const ENDPOINTS = [
  {
    id: "health",
    group: "System",
    label: "Health check",
    method: "GET",
    path: "/health",
    description: "Check backend health plus the applied v2 SQL scripts."
  },
  {
    id: "auth-register",
    group: "Auth",
    label: "Register",
    method: "POST",
    path: "/auth/register",
    description: "Create a new account and accept the current legal documents.",
    bodyTemplate: {
      email: "${email}",
      password: "${password}",
      displayName: "${displayName}",
      primaryPhone: "${primaryPhone}",
      acceptedLegalDocuments: [
        { documentKey: "privacy_policy", version: "2026-04-01" },
        { documentKey: "terms_of_service", version: "2026-04-01" }
      ]
    }
  },
  {
    id: "auth-register-admin",
    group: "Auth",
    label: "Register admin (temporary)",
    method: "POST",
    path: "/auth/register-admin",
    description: "Temporarily create an admin account without an existing login.",
    bodyTemplate: {
      email: "${email}",
      password: "${password}",
      displayName: "${displayName}",
      primaryPhone: "${primaryPhone}",
      acceptedLegalDocuments: [
        { documentKey: "privacy_policy", version: "2026-04-01" },
        { documentKey: "terms_of_service", version: "2026-04-01" }
      ]
    }
  },
  {
    id: "auth-login",
    group: "Auth",
    label: "Login",
    method: "POST",
    path: "/auth/login",
    description: "Authenticate a user and capture the returned session details.",
    bodyTemplate: {
      email: "${email}",
      password: "${password}",
      rememberMe: true
    }
  },
  {
    id: "auth-refresh",
    group: "Auth",
    label: "Refresh session",
    method: "POST",
    path: "/auth/refresh",
    description: "Refresh the current authenticated session and renew the session cookie."
  },
  {
    id: "auth-logout",
    group: "Auth",
    label: "Logout",
    method: "POST",
    path: "/auth/logout",
    description: "Expire the backend session cookie and let the client clear local cached state."
  },
  {
    id: "auth-password-change",
    group: "Auth",
    label: "Change password",
    method: "POST",
    path: "/auth/password/change",
    description: "Change the current account password using the existing password for verification.",
    bodyTemplate: {
      currentPassword: "${password}",
      newPassword: "EvenStrongerPassword!456"
    }
  },
  {
    id: "auth-password-forgot",
    group: "Auth",
    label: "Forgot password",
    method: "POST",
    path: "/auth/password/forgot",
    description: "Stubbed password-reset initiation route.",
    bodyTemplate: {
      email: "${email}"
    }
  },
  {
    id: "auth-password-reset",
    group: "Auth",
    label: "Reset password",
    method: "POST",
    path: "/auth/password/reset",
    description: "Stubbed password-reset completion route.",
    bodyTemplate: {
      token: "demo-reset-token",
      newPassword: "EvenStrongerPassword!456"
    }
  },
  {
    id: "auth-mfa-verify",
    group: "Auth",
    label: "Verify MFA",
    method: "POST",
    path: "/auth/mfa/verify",
    description: "Inspect the MFA challenge stub response.",
    bodyTemplate: {
      challengeId: "00000000-0000-0000-0000-000000000000",
      method: "totp",
      code: "123456"
    }
  },
  {
    id: "auth-passkey-options",
    group: "Auth",
    label: "Passkey auth options",
    method: "POST",
    path: "/auth/passkeys/authentication/options",
    description: "Fetch browser passkey authentication options."
  },
  {
    id: "auth-passkey-verify",
    group: "Auth",
    label: "Passkey auth verify",
    method: "POST",
    path: "/auth/passkeys/authentication/verify",
    description: "Stubbed passkey verification route.",
    bodyTemplate: {
      credentialId: "demo-passkey-id",
      response: {}
    }
  },
  {
    id: "me-get",
    group: "Profile",
    label: "Get profile",
    method: "GET",
    path: "/me",
    description: "Fetch the current profile resolved from the authenticated session."
  },
  {
    id: "me-update",
    group: "Profile",
    label: "Update profile",
    method: "PATCH",
    path: "/me",
    description: "Update the current account profile, locale, bio, timezone, and preferred currency.",
    bodyTemplate: {
      displayName: "${displayName}",
      defaultCurrency: "GBP",
      locale: "en-GB",
      timezoneName: "Europe/London",
      profileBio: "Track everyday prices and save evidence quickly."
    }
  },
  {
    id: "me-avatar-update",
    group: "Profile",
    label: "Set avatar",
    method: "POST",
    path: "/me/avatar",
    description: "Attach an uploaded file owned by the current account as the active avatar.",
    bodyTemplate: {
      fileId: "${fileId}"
    }
  },
  {
    id: "me-avatar-delete",
    group: "Profile",
    label: "Remove avatar",
    method: "DELETE",
    path: "/me/avatar",
    description: "Remove the current account avatar attachment."
  },
  {
    id: "security-overview",
    group: "Profile",
    label: "Security overview",
    method: "GET",
    path: "/me/security",
    description: "Inspect the current password, MFA, and passkey summary."
  },
  {
    id: "categories-list",
    group: "Catalog",
    label: "List categories",
    method: "GET",
    path: "/categories",
    description: "Load the catalog category tree."
  },
  {
    id: "brands-list",
    group: "Catalog",
    label: "List brands",
    method: "GET",
    path: "/brands",
    description: "Load all brands."
  },
  {
    id: "units-list",
    group: "Catalog",
    label: "List units",
    method: "GET",
    path: "/units",
    description: "Load all measurement units."
  },
  {
    id: "discount-types-list",
    group: "Catalog",
    label: "List discount types",
    method: "GET",
    path: "/discount-types",
    description: "Load the available discount types for price submissions."
  },
  {
    id: "items-list",
    group: "Catalog",
    label: "List items",
    method: "GET",
    path: "/items",
    description: "Search or load item summaries.",
    queryParams: ["q"],
    queryDefaults: {
      q: ""
    }
  },
  {
    id: "item-detail",
    group: "Catalog",
    label: "Get item",
    method: "GET",
    path: "/items/{itemId}",
    description: "Inspect a single item plus its variants.",
    pathParams: ["itemId"],
    pathDefaults: {
      itemId: "${itemId}"
    }
  },
  {
    id: "item-variants",
    group: "Catalog",
    label: "List item variants",
    method: "GET",
    path: "/items/{itemId}/variants",
    description: "Load variants for the selected item.",
    pathParams: ["itemId"],
    pathDefaults: {
      itemId: "${itemId}"
    }
  },
  {
    id: "variant-detail",
    group: "Catalog",
    label: "Get item variant",
    method: "GET",
    path: "/item-variants/{variantId}",
    description: "Inspect one variant and its product codes.",
    pathParams: ["variantId"],
    pathDefaults: {
      variantId: "${variantId}"
    }
  },
  {
    id: "variant-prices",
    group: "Catalog",
    label: "List variant prices",
    method: "GET",
    path: "/item-variants/{variantId}/prices",
    description: "Load verified public prices for a variant.",
    pathParams: ["variantId"],
    pathDefaults: {
      variantId: "${variantId}"
    }
  },
  {
    id: "variant-price-history",
    group: "Catalog",
    label: "Get price history",
    method: "GET",
    path: "/item-variants/{variantId}/price-history",
    description: "Load historical price points for the variant.",
    pathParams: ["variantId"],
    pathDefaults: {
      variantId: "${variantId}"
    }
  },
  {
    id: "compare-query",
    group: "Catalog",
    label: "Compare variants (query)",
    method: "GET",
    path: "/compare",
    description: "Compare variants using the query-string route.",
    queryParams: ["variantIds"],
    queryDefaults: {
      variantIds: "${variantId}"
    }
  },
  {
    id: "compare-body",
    group: "Catalog",
    label: "Compare variants (body)",
    method: "POST",
    path: "/comparisons",
    description: "Compare variants using a JSON body.",
    bodyTemplate: {
      variantIds: ["${variantId}"]
    }
  },
  {
    id: "shops-list",
    group: "Shops & Files",
    label: "List shops",
    method: "GET",
    path: "/shops",
    description: "Load the known active shops."
  },
  {
    id: "shop-detail",
    group: "Shops & Files",
    label: "Get shop",
    method: "GET",
    path: "/shops/{shopId}",
    description: "Inspect one shop by ID.",
    pathParams: ["shopId"],
    pathDefaults: {
      shopId: "${shopId}"
    }
  },
  {
    id: "product-code-lookup",
    group: "Shops & Files",
    label: "Lookup product code",
    method: "GET",
    path: "/shops/{shopId}/product-codes/{code}",
    description: "Resolve a barcode or code into a known item variant.",
    pathParams: ["shopId", "code"],
    pathDefaults: {
      shopId: "${shopId}",
      code: "${code}"
    }
  },
  {
    id: "file-upload-intent",
    group: "Shops & Files",
    label: "Create file upload intent",
    method: "POST",
    path: "/files/uploads",
    description: "Create an upload intent for a receipt or evidence file.",
    bodyTemplate: {
      filename: "receipt.jpg",
      contentType: "image/jpeg",
      size: 120000,
      purpose: "PRICE_EVIDENCE",
      checksumSha256: ""
    }
  },
  {
    id: "file-upload-complete",
    group: "Shops & Files",
    label: "Complete file upload",
    method: "POST",
    path: "/files/uploads/{fileId}/complete",
    description: "Mark one of the current account's upload intents as completed.",
    pathParams: ["fileId"],
    pathDefaults: {
      fileId: "${fileId}"
    }
  },
  {
    id: "file-own",
    group: "Shops & Files",
    label: "Get own file",
    method: "GET",
    path: "/me/files/{fileId}",
    description: "Inspect a file owned by the current account.",
    pathParams: ["fileId"],
    pathDefaults: {
      fileId: "${fileId}"
    }
  },
  {
    id: "file-download",
    group: "Shops & Files",
    label: "Get file download",
    method: "GET",
    path: "/me/files/{fileId}/download",
    description: "Request a temporary file download URL.",
    pathParams: ["fileId"],
    pathDefaults: {
      fileId: "${fileId}"
    }
  },
  {
    id: "purchase-create",
    group: "Purchases & Prices",
    label: "Create purchase",
    method: "POST",
    path: "/purchases",
    description: "Create a purchase that can later be linked to price observations.",
    bodyTemplate: {
      shopId: "${shopId}",
      purchaseTime: "${purchaseTimeIso}",
      attachmentFileIds: [],
      notes: "Captured from the Vue workbench"
    }
  },
  {
    id: "purchase-list",
    group: "Purchases & Prices",
    label: "List my purchases",
    method: "GET",
    path: "/me/purchases",
    description: "Load all purchases for the current account."
  },
  {
    id: "purchase-detail",
    group: "Purchases & Prices",
    label: "Get my purchase",
    method: "GET",
    path: "/me/purchases/{purchaseId}",
    description: "Inspect one purchase by ID.",
    pathParams: ["purchaseId"],
    pathDefaults: {
      purchaseId: "${purchaseId}"
    }
  },
  {
    id: "purchase-update",
    group: "Purchases & Prices",
    label: "Update my purchase",
    method: "PATCH",
    path: "/me/purchases/{purchaseId}",
    description: "Patch the notes, timestamp, or attachments for a purchase.",
    pathParams: ["purchaseId"],
    pathDefaults: {
      purchaseId: "${purchaseId}"
    },
    bodyTemplate: {
      purchaseTime: "${purchaseTimeIso}",
      attachmentFileIds: [],
      notes: "Updated from the Vue workbench"
    }
  },
  {
    id: "purchase-delete",
    group: "Purchases & Prices",
    label: "Delete my purchase",
    method: "DELETE",
    path: "/me/purchases/{purchaseId}",
    description: "Delete a purchase owned by the current account.",
    pathParams: ["purchaseId"],
    pathDefaults: {
      purchaseId: "${purchaseId}"
    }
  },
  {
    id: "price-create",
    group: "Purchases & Prices",
    label: "Create price submission",
    method: "POST",
    path: "/prices",
    description: "Submit a price observation linked to a purchase.",
    bodyTemplate: {
      itemVariantId: "${variantId}",
      purchaseId: "${purchaseId}",
      originalAmount: "${originalAmount}",
      originalCurrency: "${originalCurrency}",
      discountAmount: "${discountAmount}",
      discountCurrency: "${discountCurrency}",
      discountTypeId: "${discountTypeId}",
      recordedAt: "${recordedAtIso}",
      notes: "Submitted from the Vue workbench"
    }
  },
  {
    id: "price-list",
    group: "Purchases & Prices",
    label: "List my prices",
    method: "GET",
    path: "/me/prices",
    description: "Load all price submissions for the current account."
  },
  {
    id: "price-detail",
    group: "Purchases & Prices",
    label: "Get my price",
    method: "GET",
    path: "/me/prices/{priceId}",
    description: "Inspect one owned price submission.",
    pathParams: ["priceId"],
    pathDefaults: {
      priceId: "${priceId}"
    }
  },
  {
    id: "price-update",
    group: "Purchases & Prices",
    label: "Update my price",
    method: "PATCH",
    path: "/me/prices/{priceId}",
    description: "Patch an owned price submission.",
    pathParams: ["priceId"],
    pathDefaults: {
      priceId: "${priceId}"
    },
    bodyTemplate: {
      originalAmount: "${originalAmount}",
      originalCurrency: "${originalCurrency}",
      discountAmount: "${discountAmount}",
      discountCurrency: "${discountCurrency}",
      discountTypeId: "${discountTypeId}",
      recordedAt: "${recordedAtIso}",
      notes: "Updated from the Vue workbench"
    }
  },
  {
    id: "price-delete",
    group: "Purchases & Prices",
    label: "Delete my price",
    method: "DELETE",
    path: "/me/prices/{priceId}",
    description: "Delete a price submission owned by the current account.",
    pathParams: ["priceId"],
    pathDefaults: {
      priceId: "${priceId}"
    }
  },
  {
    id: "watchlist-list",
    group: "Watchlist & Alerts",
    label: "List watchlist",
    method: "GET",
    path: "/me/watchlist",
    description: "Load watchlist entries for the current account."
  },
  {
    id: "watchlist-create",
    group: "Watchlist & Alerts",
    label: "Create watchlist item",
    method: "POST",
    path: "/me/watchlist/items",
    description: "Add a variant to the watchlist.",
    bodyTemplate: {
      itemVariantId: "${variantId}"
    }
  },
  {
    id: "watchlist-delete",
    group: "Watchlist & Alerts",
    label: "Delete watchlist item",
    method: "DELETE",
    path: "/me/watchlist/items/{watchId}",
    description: "Remove a watchlist entry by ID.",
    pathParams: ["watchId"],
    pathDefaults: {
      watchId: "${watchId}"
    }
  },
  {
    id: "alerts-list",
    group: "Watchlist & Alerts",
    label: "List alerts",
    method: "GET",
    path: "/me/alerts",
    description: "Load price alerts for the current account."
  },
  {
    id: "alert-create",
    group: "Watchlist & Alerts",
    label: "Create alert",
    method: "POST",
    path: "/me/alerts",
    description: "Create a target-price alert for a variant.",
    bodyTemplate: {
      itemVariantId: "${variantId}",
      targetFinalAmount: "1.50",
      currency: "GBP",
      isEnabled: true
    }
  },
  {
    id: "alert-update",
    group: "Watchlist & Alerts",
    label: "Update alert",
    method: "PATCH",
    path: "/me/alerts/{alertId}",
    description: "Patch an existing price alert.",
    pathParams: ["alertId"],
    pathDefaults: {
      alertId: "${alertId}"
    },
    bodyTemplate: {
      targetFinalAmount: "1.40",
      currency: "GBP",
      isEnabled: true
    }
  },
  {
    id: "alert-delete",
    group: "Watchlist & Alerts",
    label: "Delete alert",
    method: "DELETE",
    path: "/me/alerts/{alertId}",
    description: "Delete an alert by ID.",
    pathParams: ["alertId"],
    pathDefaults: {
      alertId: "${alertId}"
    }
  },
  {
    id: "moderation-prices",
    group: "Admin",
    label: "List moderation prices",
    method: "GET",
    path: "/admin/moderation/prices",
    description: "Load price observations currently awaiting moderation."
  },
  {
    id: "admin-overview",
    group: "Admin",
    label: "Admin overview",
    method: "GET",
    path: "/admin/overview",
    description: "Load admin summary counts plus curated editable table definitions."
  },
  {
    id: "admin-users-list",
    group: "Admin",
    label: "List admin users",
    method: "GET",
    path: "/admin/users",
    description: "Load the user directory exposed to admins."
  },
  {
    id: "admin-users-create",
    group: "Admin",
    label: "Create admin user",
    method: "POST",
    path: "/admin/users",
    description: "Create a new user or admin account from the admin console.",
    bodyTemplate: {
      email: "new.user@example.com",
      password: "StrongPassword!234",
      displayName: "New User",
      primaryPhone: "+447700900123",
      roleCodes: ["user"],
      accountStatus: "active"
    }
  },
  {
    id: "admin-users-update",
    group: "Admin",
    label: "Update admin user",
    method: "PATCH",
    path: "/admin/users/{accountId}",
    description: "Update one user account from the admin portal editor.",
    pathParams: ["accountId"],
    pathDefaults: {
      accountId: "${accountId}"
    },
    bodyTemplate: {
      displayName: "Updated User",
      primaryEmail: "updated.user@example.com",
      primaryPhone: "+447700900123",
      roleCodes: ["user"],
      accountStatus: "active",
      defaultCurrency: "GBP",
      locale: "en-GB",
      timezoneName: "Europe/London",
      profileBio: "Updated by an admin."
    }
  },
  {
    id: "admin-users-bulk",
    group: "Admin",
    label: "Bulk update admin users",
    method: "POST",
    path: "/admin/users/bulk-actions",
    description: "Apply a bulk action such as freeze, activate, delete, restore, or set-status.",
    bodyTemplate: {
      accountIds: ["${accountId}"],
      action: "freeze",
      reason: "Administrative action",
      status: "suspended"
    }
  },
  {
    id: "admin-settings-list",
    group: "Admin",
    label: "List admin settings",
    method: "GET",
    path: "/admin/settings",
    description: "Load the system settings exposed through the admin dashboard."
  },
  {
    id: "admin-settings-update",
    group: "Admin",
    label: "Update admin setting",
    method: "PATCH",
    path: "/admin/settings/{settingKey}",
    description: "Update one system setting value.",
    pathParams: ["settingKey"],
    pathDefaults: {
      settingKey: "system.maintenanceMode"
    },
    bodyTemplate: {
      value: true
    }
  },
  {
    id: "admin-db-tables",
    group: "Admin",
    label: "List admin tables",
    method: "GET",
    path: "/admin/database/tables",
    description: "List the database tables exposed through the admin dashboard."
  },
  {
    id: "admin-db-table",
    group: "Admin",
    label: "Get admin table rows",
    method: "GET",
    path: "/admin/database/tables/{tableId}",
    description: "Load editable rows for one admin-managed table.",
    pathParams: ["tableId"],
    pathDefaults: {
      tableId: "categories"
    }
  },
  {
    id: "admin-db-table-create",
    group: "Admin",
    label: "Create admin row",
    method: "POST",
    path: "/admin/database/tables/{tableId}",
    description: "Create a new row in a curated admin-managed table.",
    pathParams: ["tableId"],
    pathDefaults: {
      tableId: "categories"
    },
    bodyTemplate: {
      values: {}
    }
  },
  {
    id: "admin-db-record-update",
    group: "Admin",
    label: "Update admin row",
    method: "PATCH",
    path: "/admin/database/tables/{tableId}/{recordId}",
    description: "Update one row in a curated admin-managed table.",
    pathParams: ["tableId", "recordId"],
    pathDefaults: {
      tableId: "categories",
      recordId: "00000000-0000-0000-0000-000000000000"
    },
    bodyTemplate: {
      values: {}
    }
  },
  {
    id: "admin-db-record-delete",
    group: "Admin",
    label: "Delete admin row",
    method: "DELETE",
    path: "/admin/database/tables/{tableId}/{recordId}",
    description: "Delete or archive one row in a curated admin-managed table.",
    pathParams: ["tableId", "recordId"],
    pathDefaults: {
      tableId: "categories",
      recordId: "00000000-0000-0000-0000-000000000000"
    }
  },
  {
    id: "admin-db-record-approve",
    group: "Admin",
    label: "Approve admin row",
    method: "POST",
    path: "/admin/database/tables/{tableId}/{recordId}/approve",
    description: "Approve one admin-managed record in a module that supports approval.",
    pathParams: ["tableId", "recordId"],
    pathDefaults: {
      tableId: "items",
      recordId: "00000000-0000-0000-0000-000000000000"
    }
  },
  {
    id: "moderation-verify",
    group: "Admin",
    label: "Verify moderated price",
    method: "POST",
    path: "/admin/moderation/prices/{priceId}/verify",
    description: "Approve a pending price observation.",
    pathParams: ["priceId"],
    pathDefaults: {
      priceId: "${priceId}"
    },
    bodyTemplate: {
      reason: "Looks correct"
    }
  },
  {
    id: "moderation-reject",
    group: "Admin",
    label: "Reject moderated price",
    method: "POST",
    path: "/admin/moderation/prices/{priceId}/reject",
    description: "Reject a pending price observation.",
    pathParams: ["priceId"],
    pathDefaults: {
      priceId: "${priceId}"
    },
    bodyTemplate: {
      reason: "Insufficient evidence"
    }
  },
  {
    id: "emails-list",
    group: "Identity",
    label: "List emails",
    method: "GET",
    path: "/me/emails",
    description: "Load the email addresses on the current account."
  },
  {
    id: "emails-create",
    group: "Identity",
    label: "Create email",
    method: "POST",
    path: "/me/emails",
    description: "Add a secondary email address.",
    bodyTemplate: {
      email: "alt@example.com",
      emailRole: "SECONDARY",
      isLoginEnabled: true
    }
  },
  {
    id: "emails-delete",
    group: "Identity",
    label: "Delete email",
    method: "DELETE",
    path: "/me/emails/{emailId}",
    description: "Soft-delete an email by ID.",
    pathParams: ["emailId"],
    pathDefaults: {
      emailId: "${emailId}"
    }
  },
  {
    id: "emails-verify",
    group: "Identity",
    label: "Verify email",
    method: "POST",
    path: "/me/emails/{emailId}/verify",
    description: "Mark an email as verified using a code payload.",
    pathParams: ["emailId"],
    pathDefaults: {
      emailId: "${emailId}"
    },
    bodyTemplate: {
      code: "123456"
    }
  },
  {
    id: "emails-make-primary",
    group: "Identity",
    label: "Make email primary",
    method: "POST",
    path: "/me/emails/{emailId}/make-primary",
    description: "Promote an email address to primary.",
    pathParams: ["emailId"],
    pathDefaults: {
      emailId: "${emailId}"
    }
  },
  {
    id: "phones-list",
    group: "Identity",
    label: "List phones",
    method: "GET",
    path: "/me/phones",
    description: "Load phone numbers for the current account."
  },
  {
    id: "phones-create",
    group: "Identity",
    label: "Create phone",
    method: "POST",
    path: "/me/phones",
    description: "Add a phone number to the current account.",
    bodyTemplate: {
      phoneNumber: "+447700900123"
    }
  },
  {
    id: "phones-delete",
    group: "Identity",
    label: "Delete phone",
    method: "DELETE",
    path: "/me/phones/{phoneId}",
    description: "Soft-delete a phone number by ID.",
    pathParams: ["phoneId"],
    pathDefaults: {
      phoneId: "${phoneId}"
    }
  },
  {
    id: "phones-verify",
    group: "Identity",
    label: "Verify phone",
    method: "POST",
    path: "/me/phones/{phoneId}/verify",
    description: "Mark a phone number as verified using a code payload.",
    pathParams: ["phoneId"],
    pathDefaults: {
      phoneId: "${phoneId}"
    },
    bodyTemplate: {
      code: "123456"
    }
  },
  {
    id: "phones-make-primary",
    group: "Identity",
    label: "Make phone primary",
    method: "POST",
    path: "/me/phones/{phoneId}/make-primary",
    description: "Promote a phone number to primary.",
    pathParams: ["phoneId"],
    pathDefaults: {
      phoneId: "${phoneId}"
    }
  },
  {
    id: "legal-docs",
    group: "Legal & Privacy",
    label: "List legal documents",
    method: "GET",
    path: "/legal/documents",
    description: "Load the currently published legal documents."
  },
  {
    id: "privacy-consents-list",
    group: "Legal & Privacy",
    label: "List privacy consents",
    method: "GET",
    path: "/me/privacy-consents",
    description: "Load the current account's accepted legal documents."
  },
  {
    id: "privacy-consents-create",
    group: "Legal & Privacy",
    label: "Accept privacy documents",
    method: "POST",
    path: "/me/privacy-consents",
    description: "Create new privacy-consent records.",
    bodyTemplate: {
      acceptedLegalDocuments: [
        { documentKey: "privacy_policy", version: "2026-04-01" },
        { documentKey: "cookie_policy", version: "2026-04-01" }
      ]
    }
  },
  {
    id: "cookie-preferences-get",
    group: "Legal & Privacy",
    label: "Get cookie preferences",
    method: "GET",
    path: "/privacy/cookie-preferences",
    description: "Load the latest cookie-consent values."
  },
  {
    id: "cookie-preferences-update",
    group: "Legal & Privacy",
    label: "Update cookie preferences",
    method: "POST",
    path: "/privacy/cookie-preferences",
    description: "Save updated cookie preferences.",
    bodyTemplate: {
      analytics: false,
      marketing: false,
      preferences: true
    }
  },
  {
    id: "passkeys-list",
    group: "Passkeys & MFA",
    label: "List passkeys",
    method: "GET",
    path: "/me/passkeys",
    description: "Inspect the current passkey list stub."
  },
  {
    id: "passkeys-registration-options",
    group: "Passkeys & MFA",
    label: "Passkey registration options",
    method: "POST",
    path: "/me/passkeys/registration/options",
    description: "Fetch browser passkey registration options."
  },
  {
    id: "passkeys-registration-verify",
    group: "Passkeys & MFA",
    label: "Verify passkey registration",
    method: "POST",
    path: "/me/passkeys/registration/verify",
    description: "Stubbed passkey registration verification route.",
    bodyTemplate: {
      credentialId: "demo-passkey-id",
      response: {}
    }
  },
  {
    id: "passkeys-delete",
    group: "Passkeys & MFA",
    label: "Delete passkey",
    method: "DELETE",
    path: "/me/passkeys/{passkeyId}",
    description: "Delete a passkey by ID.",
    pathParams: ["passkeyId"],
    pathDefaults: {
      passkeyId: "00000000-0000-0000-0000-000000000000"
    }
  },
  {
    id: "mfa-totp-setup",
    group: "Passkeys & MFA",
    label: "TOTP setup",
    method: "POST",
    path: "/me/mfa/totp/setup",
    description: "Stubbed TOTP setup route."
  },
  {
    id: "mfa-totp-enable",
    group: "Passkeys & MFA",
    label: "TOTP enable",
    method: "POST",
    path: "/me/mfa/totp/enable",
    description: "Stubbed TOTP enable route.",
    bodyTemplate: {
      code: "123456"
    }
  },
  {
    id: "mfa-totp-disable",
    group: "Passkeys & MFA",
    label: "TOTP disable",
    method: "POST",
    path: "/me/mfa/totp/disable",
    description: "Stubbed TOTP disable route.",
    bodyTemplate: {
      code: "123456"
    }
  },
  {
    id: "mfa-recovery-rotate",
    group: "Passkeys & MFA",
    label: "Rotate recovery codes",
    method: "POST",
    path: "/me/mfa/recovery-codes/rotate",
    description: "Stubbed recovery-code rotation route."
  },
  {
    id: "custom",
    group: "Custom",
    label: "Custom request",
    method: "GET",
    path: "/health",
    description: "Manually compose a request for any additional route or experiment."
  }
];
