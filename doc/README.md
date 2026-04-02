# Price_Tracker

## What it does (By ChatGPT)

### 🧭 1. Core Feature Categories (System Modules)

#### 📦 Item & Catalog Module

* Browse items, variants, brands, categories
* Normalize comparison (unit conversion via `Unit`)
* Handle SKU / product matching

#### 💰 Price Tracking Module

* Record prices (`PriceRecord`)
* Link to purchase (`PurchaseRecord`)
* Track discounts

#### 📊 Analytics & Comparison Module

* Compare prices across shops
* Historical trends
* Unit price normalization (very important)

#### 🏪 Shop Module

* Manage shops and locations
* Link prices to shops

#### 👤 User & Source Module

* Track who submitted data (`SourceID`)
* Trust / validation system


### 👀 2. Guest Functions (No Login)

Goal: **explore + compare**, minimal friction

#### 🔍 Search & Browse

* Search item by name / category
* Filter by:

  * Category (`CategoryID`)
  * Brand (`BrandID`)
  * Shop
* View item details:

  * Specifications
  * Variants (size, unit)

#### 📊 Price Comparison

* Compare same item across shops
  👉 Example:

  * “Milk 1L across Tesco, Aldi, Lidl”

* Compare variants using unit normalization
  👉 £1.50 / 1L vs £2.50 / 2L → show £/L

#### 📈 Price History (READ ONLY)

* View price trend of an item variant
* Compare **two items price history** (your example)

👉 Example function:

```
ComparePriceHistory(itemVariantA, itemVariantB)
→ returns time-series graph
```

#### 🏪 Shop View

* View shop details
* Show:

  * Latest prices
  * Cheapest items in that shop

#### ⭐ Insights (basic)

* Cheapest shop for an item
* Average price
* Price volatility

---

### 👤 3. Registered User Functions

Goal: **contribute + personalize**

#### 🧾 Data Contribution

* Add purchase record (`PurchaseRecord`)
* Upload receipt (link to `FileID`)
* Add price record (`PriceRecord`)
* Add new item / variant if missing

👉 Important logic:

* Auto-suggest existing item to avoid duplicates

#### ✏️ Edit & Manage Own Data

* Edit submitted prices
* Delete incorrect entries
* Add notes (e.g. “clearance sale”)

#### 🔔 Tracking & Alerts

* Track item price:

  * “Notify me when milk < £1”

* Watchlist:

  * Items
  * Shops

#### 📊 Advanced Comparison

* Multi-item comparison table
  👉 Example:

  ```
  CompareItems([milk_1L, milk_2L, milk_500ml])
  ```

* Basket comparison:

  * Cheapest shop for a list of items

#### 📈 Personal Analytics

* Spending history
* Price trends of frequently bought items

#### 🏷️ Smart Features

* Auto unit conversion:

  * g ↔ kg, ml ↔ L
* Detect abnormal price:

  * Flag outliers

#### 🤝 Community Features (optional but powerful)

* Upvote/downvote price records
* Trust score for contributors
* Report incorrect data


### 🛠️ 4. Admin Functions

Goal: **data integrity + system control**

#### 🧹 Data Moderation

* Approve/reject new:

  * Items
  * Variants
  * Brands
* Detect duplicates:

  * Same item created twice

👉 Example:

```
MergeItem(itemA, itemB)
```

#### 🔍 Data Validation

* Flag suspicious prices:

  * Too low/high vs average
* Validate receipt uploads

#### 🏪 Shop Management

* Add/edit shops
* Link to `Address`
* Verify shop legitimacy

#### 🧾 Price Record Management

* Bulk edit/delete records
* Fix incorrect unit mappings

#### 📊 System Analytics

* Most tracked items
* Most active users
* Price trends across regions

#### ⚙️ Configuration

* Manage:

  * Discount types
  * Units & conversions
  * Categories hierarchy

#### 🔐 Security

* Monitor:

  * SQL injection attempts
  * Abuse of API
* Rate limiting
* User banning

---

### 🧠 5. Advanced / Smart Features (High Value)

These are what make your system stand out:

#### 🧮 Unit Price Engine (VERY IMPORTANT)

Core function:

```
NormalizePrice(price, qty, unit) → price per base unit
```

Uses:

* `Unit.BaseUnitID`

#### 🧠 Item Matching / Deduplication

* Detect:

  * “Coca Cola 500ml” vs “Coke 0.5L”
* Use:

  * Name similarity
  * SKU

#### 🛒 Basket Optimization

```
FindCheapestCombination(itemList, shops)
```

→ “Buy milk from Aldi, bread from Tesco”

#### 📉 Price Prediction (future)

* Predict price trends
* Detect sales cycles


### 🧩 6. API-Level Function Examples

These align directly with your DB:

#### Item APIs

* `GET /items?name=milk`
* `GET /item/{id}/variants`

#### Price APIs

* `GET /price-history?itemVariantID=...`
* `POST /price-record`

#### Comparison APIs

* `GET /compare?items=[...]`
* `GET /cheapest?itemID=...`

#### Analytics APIs

* `GET /price-trend`
* `GET /unit-price`


### ⚠️ 7. Key Design Considerations (Important)

#### 1. Unit Normalization

Without this, comparisons are meaningless.

#### 2. Duplicate Prevention

* Same item created multiple times = biggest risk

#### 3. Trust System

* Not all user-submitted prices are reliable

#### 4. SQL Injection Risk

Since you mentioned it:

* Always use prepared statements
* Never trust:

  * item name
  * notes
  * search filters

#### 5. Performance

* Price history queries can get heavy
  → Use indexing on:

  * `ItemVarientID`
  * `RecordAt`


## Modules
### Frontend Navication Structure
```
Home
 ├── Search Results (1)
 │     └── Item Detail (2)
 │            ├── Price Comparison (3)
 │            ├── Price History (4)
 │            └── Add Price (5)
 │
 ├── Compare (multi-item)
 ├── Shops
 │     ├── Shop Detail
 │     └── Search Shop
 │
 ├── Dashboard (user only)
 │     ├── Watchlist
 │     ├── My Records
 │     └── Insights
 │
 ├── Admin Panel (admin only)
 │
 └── Auth (Login/Register)
```
1. Search Results
- Step 1-1: Search `Item` with `user_input`
- Step 1-2: Search `ItemVarient` with `user_input`
- Step 2: Jump to Item Details with `ItemID` and `ItemVarientID`(2)

2. Item Details (2)
- Step 0: Call with list of `ItemID` and `ItemVarientID`
- Step 1: If empty, prompt not found
- Step 2: If `ItemID` found, request details of `ItemID` and sons in `ItemVarient`
- Step 3: If `ItemVarientID` found (from Step 0 and 2), request details of `ItemVarientID`, 
### 🏠 2. Home Page (Entry Point)

#### 🎯 Goal:

Fast search + discovery

#### 🧩 Components:

* 🔍 Search bar (primary focus)
* 🏷 Category shortcuts
* 📊 Trending items
* 💸 “Recently cheapest” items
* 🏪 Popular shops

#### 🔁 User Actions:

* Search → go to **Search Results**
* Click item → go to **Item Detail**
* Click category → filtered search


### 🔍 3. Search Results Page

#### 🎯 Goal:

Help user find the **correct item / variant**

#### 🧩 Components:

* Filters:

  * Category
  * Brand
  * Unit (e.g. L, kg)
* Result list:

  * Item name
  * Variant (e.g. 1L, 500g)
  * Lowest price preview
  * Shop preview

#### 🔁 User Actions:

* Click item → **Item Detail**
* Refine filters → reload results

#### ⚡ Logic:

* Backend aggregates:

  ```
  MIN(price) GROUP BY ItemVariant
  ```


### 📦 4. Item Detail Page (Core Page)

This is your **most important page**.

#### 🧩 Layout (tab-based)

##### 🧾 Header

* Item name + specification
* Category breadcrumb
* Variant selector (VERY IMPORTANT)

#### 📊 Tab 1: Price Comparison (Default)

##### Components:

* Table:
  | Shop | Price | Unit Price | Last Updated | Discount |
* Sort:

  * Cheapest
  * Nearest (future)
* Highlight:

  * 🟢 Cheapest
  * 🔴 Expensive

##### ⚡ Logic:

* Normalize price using `Unit`
* Show:

  ```
  price / base_unit
  ```


#### 📈 Tab 2: Price History

##### Components:

* Line chart (time vs price)
* Filters:

  * Shop
  * Time range

👉 Example:

* Tesco vs Aldi milk price over time


#### ⚖️ Tab 3: Compare Variants

##### Components:

* Table:
  | Variant | Qty | Price | Unit Price |
* Helps answer:
  → “Is 2L cheaper than 1L?”


#### ➕ Tab 4: Add Price (User only)

##### Components:

* Form:

  * Shop
  * Price
  * Discount
  * Date
  * Upload receipt

##### ⚡ UX:

* Auto-fill item variant
* Suggest recent shops


### ⚖️ 5. Compare Page (Multi-Item)

#### 🎯 Goal:

Compare **different items together**

#### 🧩 Components:

* Add items (search + select)
* Comparison table:

| Item | Shop | Price | Unit Price |
| ---- | ---- | ----- | ---------- |

#### 🔁 User Actions:

* Add/remove items dynamically
* Sort by cheapest


### 🏪 6. Shop Page

#### 🧩 Shop List

* List of shops
* Filter by location (future)

#### 🏪 Shop Detail Page

##### Components:

* Shop info
* Cheapest items in this shop
* Recent price updates

##### 🔁 Actions:

* Click item → Item Detail


### 👤 7. User Dashboard

#### 🧭 Sections


#### ⭐ Watchlist

* Tracked items
* Show:

  * Current lowest price
  * Price change

#### 🧾 My Records

* List of submitted prices
* Edit / delete


#### 📊 Insights

* Spending summary
* Price trends of tracked items


### 🔐 8. Auth Pages

#### Login / Register

* Simple forms
* Optional:

  * Social login


### 🛠️ 9. Admin Panel

#### 🧩 Sections:

##### 📦 Item Management

* Approve / merge items

##### 💰 Price Records

* View suspicious entries
* Bulk delete/edit

##### 🏪 Shops

* Add/edit shops

##### ⚙️ Config

* Units
* Categories
* Discount types


### 🔁 10. Key User Journeys

#### 🧭 Journey 1: Guest Comparing Prices

```
Home
 → Search "milk"
 → Search Results
 → Item Detail
 → View cheapest shop
 → View price history
```


#### 🧭 Journey 2: User Adding Price

```
Login
 → Search item
 → Item Detail
 → Add Price
 → Submit
 → Redirect to updated comparison
```

#### 🧭 Journey 3: Smart Shopper

```
Compare Page
 → Add multiple items
 → View cheapest shops
 → Decide where to buy
```


#### 🧭 Journey 4: Contributor Loop

```
Dashboard
 → My Records
 → Add new price
 → Gain trust score (future)
```


### 🧠 11. UX Design Principles (Important)

#### 1. Minimize Friction

* Search-first design
* Don’t force login for browsing

#### 2. Make Comparison Obvious

* Always show:

  * Cheapest
  * Unit price

#### 3. Reduce Data Errors

* Autocomplete:

  * Item
  * Shop

#### 4. Handle Complexity Invisibly

* Users shouldn’t think about:

  * Units
  * Variants
    → system handles it

#### 5. Mobile-Friendly

Most users will check prices in-store.

### ⚡ 12. Suggested Frontend Tech Structure

#### Pages (React-style example)

```
/pages
  Home
  SearchResults
  ItemDetail
  Compare
  Shop
  Dashboard
  Admin
```

#### Components

```
SearchBar
ItemCard
PriceTable
PriceChart
VariantSelector
```



### Backend

API → Validation → Sanitisation → Business Logic → Database → Response


### Frontend Edit Common Process
F0. Local parameter pass in  
F1. Fetch missing parameters from server as required  
F2. Terminate if server returns error  
F3. Request user input  
  F3.1 Fetch as request  
  F3.2 If not found, prompt new  
  F3.3 Save current progress  
  F3.4 Jump to add new  
F4. Send data to backend as json and files  
F5. Jump to F3 if backend suggest errors, or F8 to exit  
F6. Show save success message if backend replied  
F7. For Add, suggest new record  
F8. Exit  

### Add / Edit a purchase
`PurchaseID`

### Add / Edit a record
- `ID`: Obtain when uploaded to server  
- `ItemVarientID`: Scan / Type in -> Fetch -> Select, if not exist add new  
- `BatchCode`: Scan / Type in
- `SN`: Scan / Type in  
- `PurchaseID`: Parameter  
- `OriginalAmount` and `OriginalCurrency`: Type in Amount and Fetch - > Select currency  
- `DiscountAmount` and `DiscountCurrency`: Type in Amount and Fetch - > Select currency  
- `DiscountTypeID`: Fetch -> Select / Type in  
- `RecordAt`: Suggest Purchase Time / Type in  
- `SourceID`: UserID  
- `Notes`: Type in


### Manage records

### Manage Goods/Servies

### File(photo) management

### Address management

### Visualization


## API
The API can be separate to two parts: `PriceTrack` dedicated and general user and account control.  
Please check the [API_Guide.md](API_Guide.md) for more details on how to use them.


# Reference
