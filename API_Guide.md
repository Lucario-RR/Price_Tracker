# API_Guide
## `PriceTracker` Dedicated
The following modules are for `PriceTracker` only. 
They are categorised by database tables and business logic.  
The details of each table can check [Dev_doc]()

### `PriceRecord`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/PriceRecord`    | Get value(s) of an existing row         |         |          |         |
| `POST`   | `/PriceTracker/PriceRecord`    | Add a new row in `PriceRecord`          |         |          |         |
| `PATCH`  | `/PriceTracker/PriceRecord`    | Edit an existing row in `PriceRecord`   |         |          |         |
| `DELETE` | `/PriceTracker/PriceRecord`    | Remove an existing row in `PriceRecord` |         |          |         |
|          |                                |                                         |         |          |         |

### `PurchaseRecord`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/PurchaseRecord` |                                         |         |          |         |
| `POST`   | `/PriceTracker/PurchaseRecord` |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/PurchaseRecord` |                                         |         |          |         |
| `DELETE` | `/PriceTracker/PurchaseRecord` |                                         |         |          |         |

### `ItemVarient`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/ItemVarient`    |                                         |         |          |         |
| `POST`   | `/PriceTracker/ItemVarient`    |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/ItemVarient`    |                                         |         |          |         |
| `DELETE` | `/PriceTracker/ItemVarient`    |                                         |         |          |         |

### `Item`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/Item`           |                                         |         |          |         |
| `POST`   | `/PriceTracker/Item`           |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/Item`           |                                         |         |          |         |
| `DELETE` | `/PriceTracker/Item`           |                                         |         |          |         |

### `Category`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/Category`       |                                         |         |          |         |
| `POST`   | `/PriceTracker/Category`       |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/Category`       |                                         |         |          |         |
| `DELETE` | `/PriceTracker/Category`       |                                         |         |          |         |

### `Brand`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/Brand`          |                                         |         |          |         |
| `POST`   | `/PriceTracker/Brand`          |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/Brand`          |                                         |         |          |         |
| `DELETE` | `/PriceTracker/Brand`          |                                         |         |          |         |

### `Shop`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/Shop`           |                                         |         |          |         |
| `POST`   | `/PriceTracker/Shop`           |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/Shop`           |                                         |         |          |         |
| `DELETE` | `/PriceTracker/Shop`           |                                         |         |          |         |
|          |                                |                                         |         |          |         |

### `Address`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/Address`        |                                         |         |          |         |
| `POST`   | `/PriceTracker/Address`        |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/Address`        |                                         |         |          |         |
| `DELETE` | `/PriceTracker/Address`        |                                         |         |          |         |
|          |                                |                                         |         |          |         |

### `Unit`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/Unit`           |                                         |         |          |         |
| `POST`   | `/PriceTracker/Unit`           |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/Unit`           |                                         |         |          |         |
| `DELETE` | `/PriceTracker/Unit`           |                                         |         |          |         |

### `DiscountType`

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
| `GET`    | `/PriceTracker/DiscountType`   |                                         |         |          |         |
| `POST`   | `/PriceTracker/DiscountType`   |                                         |         |          |         |
| `PATCH`  | `/PriceTracker/DiscountType`   |                                         |         |          |         |
| `DELETE` | `/PriceTracker/DiscountType`   |                                         |         |          |         |
|          |                                |                                         |         |          |         |


## General Environment
The following modules are the fundmental to run the system, no matter which project is.  

### `User`
This part handles general user setting:  
- Edit user profile  
- New user registration  
- Delete user  
- ...  

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
|          | `/user/`                       |                                         |         |          |         |
|          |                                |                                         |         |          |         |
|          | `/user/Setting`                |                                         |         |          |         |
|          |                                |                                         |         |          |         |

### `Admin`
This part is similar to `User`, but it is more powerful to config system setting and database. 
Usually won't be accessable to normal users.  

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
|          | `/admin/`                      |                                         |         |          |         |
|          |                                |                                         |         |          |         |


### `Security`
This part handles anyrequest releted to logon:  
- Logon  
- Password setting  
- Password recovering  
- Provide session key  
- Provide cookie  
- ...  

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
|          | `/security`                    |                                         |         |          |         |
|          |                                |                                         |         |          |         |

### `File`
Anything related to file uploading will be processed here.  

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
|          | `/file`                        |                                         |         |          |         |
|          |                                |                                         |         |          |         |

### Unsorted
Planned bur unsorted.  

| Method   | Endpoint                       | Description                             | Version | Response | Request |
|----------|--------------------------------|-----------------------------------------|---------|----------|---------|
|          |                                |                                         |         |          |         |
|          |                                |                                         |         |          |         |
|          |                                |                                         |         |          |         |

# Reference
[REST API Introduction](https://www.geeksforgeeks.org/node-js/rest-api-introduction/)
