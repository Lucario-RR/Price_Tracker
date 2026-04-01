## Database Structure
### Item
| VariableName    | Type    | PK/FK | Nullable? | Description                                                                                                                                        |
|-----------------|---------|-------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| `ID`            | Varchar | PK    | N         | Local unique id                                                                                                                                    |
| `CategoryID`    | Varchar | FK    | Y         | Category of goods/services                                                                                                                         |
| `Name`          | Varchar |       | N         | Name of the goods/services                                                                                                                         |
| `Specification` | Varchar |       | Y         | Spec of the item, e.g. pork belly can be cut in slices or uncut. Quantity does not include in this field, i.e. milk 1L or 2L share same item milk. |
| `Notes`         | Varchar |       | Y         | More notes                                                                                                                                         |
| `CreateAt`      | Varchar |       | N         | Timestamp of first creation                                                                                                                        |
### Category
| VariableName | Type    | PK/FK | Nullable? | Description                              |
|--------------|---------|-------|-----------|------------------------------------------|
| `ID`         | Varchar | PK    | N         | An uid for category                      |
| `FatherID`   | Varchar | FK    | Y         | Big category of the current sub category |
| `Name`       | Varchar |       | N         | Name of the category                     |
| `Notes`      | Varchar |       | Y         | Additional Notes                         |

### Brand
| VariableName | Type    | PK/FK | Nullable? | Description                                 |
|--------------|---------|-------|-----------|---------------------------------------------|
| `ID`         | Varchar | PK    | N         |                                             |
| `Name`       | Varchar |       | N         |                                             |
| `Region`     | Char(3) |       | Y         | Use ISO 3166 Numeric code                   |
| `LocarionID` | Varchar | FK    | Y         | Link to a detailed address in another table |

### Shop
| VariableName | Type    | PK/FK | Nullable? | Description               |
|--------------|---------|-------|-----------|---------------------------|
| `ID`         | Varchar | PK    | N         | UID of place of purchase  |
| `Name`       | Varchar |       | N         | Name of place of purchase |
| `Location`   | Varchar | FK    | Y         | Address if applicable     |
| `Website`    | Varchar |       | Y         | Website if applicable     |
| `Telephone`  | Varchar |       | Y         | Telephone if applicable   |

### ItemVarient
| VariableName | Type    | PK/FK | Nullable? | Description        |
|--------------|---------|-------|-----------|--------------------|
| `ID`         | Varchar | PK    | N         |                    |
| `ItemID`     | Varchar | FK    | N         |                    |
| `BrandID`    | Varchar | FK    | Y         |                    |
| `Qty`        | Number  |       | N         | Quantity of item   |
| `UnitID`  | Varchar | FK    | N         | Unit of quantity   |
| `SKU`        | Varchar |       | Y         | Barcode ish things |
| `Website` | Text | | Y | Website link if available |

### Unit
| VariableName | Type    | PK/FK | Nullable? | Description                                      |
|--------------|---------|-------|-----------|--------------------------------------------------|
| `ID`         | Varchar | PK    | N         |                                                  |
| `Name`       | Varchar |       | N         |                                                  |
| `BaseUnitID` | Varchar | FK    | Y         | Used to quote base unit, e.g. kg for g, L for ml |

### PurchaseRecord
| VariableName   | Type    | PK/FK | Nullable? | Description      |
|----------------|---------|-------|-----------|------------------|
| `ID`           | Varchar | PK    | N         |                  |
| `RecieptID`    | Varchar |       | Y         |                  |
| `ShopID`       | Varchar | PK    | Y         |                  |
| `PurchaseTime` | Time    |       | N         |                  |
| `VATID`        | Varchar |       | Y         |                  |
| `FileID`       | Varchar | FK    | Y         | Photo of reciept |

### PriceRecord
| VariableName       | Type    | PK/FK | Nullable? | Description                                                       |
|--------------------|---------|-------|-----------|-------------------------------------------------------------------|
| `ID`               | Varchar | PK    | N         |                                                                   |
| `ItemVarientID`    | Varchar | FK    | N         |                                                                   |
| `BatchCode`        | Varchar |       | Y         |                                                                   |
| `SN`               | Varchar |       | Y         |                                                                   |
| `PurchaseID`       | Varchar | FK    | Y         |                                                                   |
| `OriginalAmount`   | Number  |       | N         |                                                                   |
| `OriginalCurrency` | Char(3) |       | N         |                                                                   |
| `DiscountAmount`   | Number  |       | Y         |                                                                   |
| `DiscountCurrency` | Char(3) |       | Y         |                                                                   |
| `DiscountTypeID`   | Varchar | FK    | Y         | Types of discount, reduce to clear, membership, general discount… |
| `RecordAt`         | Time    |       | N         |                                                                   |
| `SourceID`         | Varchar | FK    | N         | Who registe the record                                            |
| `Notes`            | Text    |       | Y         |                                                                   |

### DiscountType

### Address
| VariableName     | Type        | PK/FK | Nullable? | Description |
|------------------|------------|-------|-----------|-------------|
| id               | SERIAL     | PK    | No        | Unique identifier for the address |
| country_code     | CHAR(2)    |       | No        | ISO country code (e.g. GB, US, CN) |
| building_number  | TEXT       |       | Yes       | House/building number (e.g. 221B, 10) |
| street_name      | TEXT       |       | Yes       | Street name (e.g. Baker Street) |
| street_line2     | TEXT       |       | Yes       | Additional address line (area, estate, etc.) |
| unit             | TEXT       |       | Yes       | Flat/Apt/Room/Suite number |
| floor            | TEXT       |       | Yes       | Floor number or level |
| building_name    | TEXT       |       | Yes       | Building name (e.g. Tower A, Sherlock House) |
| district         | TEXT       |       | Yes       | District / borough / locality |
| city             | TEXT       |       | Yes       | City or town |
| state_region     | TEXT       |       | Yes       | State / county / province |
| postal_code      | TEXT       |       | Yes       | ZIP/postcode |
| landmark         | TEXT       |       | Yes       | Nearby landmark for navigation |
| notes            | TEXT       |       | Yes       | Additional notes or delivery info |
| full_text        | TEXT       |       | Yes       | Original unstructured address |
| created_at       | TIMESTAMP  |       | No        | Record creation timestamp |


# Reference
[C4 Diagram](https://c4model.com/diagrams/system-context)
