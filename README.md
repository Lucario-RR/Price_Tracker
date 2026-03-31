# Price_Tracker

## What it does
- Add records of goods/servies price at different time
- Add / Remove goods/servies
- Visualise in tables, graphs, etc...
- Compare same goods/services across different brands/place of purchase

## Workflow
### Add (a) record(s)

### Manage records

### Manage Goods/Servies

### Visualization
To do...

## Database Struceture
### Items
| VariableName    | Type    | PK/FK | Nullable? | Description                                                                                                                                        |
|-----------------|---------|-------|-----------|----------------------------------------------------------------------------------------------------------------------------------------------------|
| `ID`            | Varchar | PK    | N         | Local unique id                                                                                                                                    |
| `CategoryID`    | Varchar | FK    | Y         | Category of goods/services                                                                                                                         |
| `Name`          | Varchar |       | N         | Name of the goods/services                                                                                                                         |
| `Specification` | Varchar |       | Y         | Spec of the item, e.g. pork belly can be cut in slices or uncut. Quantity does not include in this field, i.e. milk 1L or 2L share same item milk. |
| `Notes`         | Varchar |       | Y         | More notes                                                                                                                                         |
| `CreateAt`      | Varchar |       | N         | Timestamp of first creation                                                                                                                        |
### Categories
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

### ItemBrand
| VariableName | Type    | PK/FK | Nullable? | Description        |
|--------------|---------|-------|-----------|--------------------|
| `ID`         | Varchar | PK    | N         |                    |
| `ItemID`     | Varchar | FK    | N         |                    |
| `BrandID`    | Varchar | FK    | Y         |                    |
| `Qty`        | Number  |       | N         | Quantity of item   |
| `QtyUnitID`  | Varchar | FK    | N         | Unit of quantity   |
| `SKU`        | Varchar |       | Y         | Barcode ish things |




