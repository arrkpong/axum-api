# Chrono

**Date & Time Library** - ไลบรารีจัดการวันที่และเวลา

## 📦 ข้อมูลพื้นฐาน

| รายการ     | ข้อมูล                                                               |
| ---------- | -------------------------------------------------------------------- |
| Repository | [github.com/chronotope/chrono](https://github.com/chronotope/chrono) |
| เอกสาร     | [docs.rs/chrono](https://docs.rs/chrono)                             |

## 🎯 Chrono คืออะไร?

Chrono เป็น Date/Time library ที่:

- **Timezone aware** - รองรับ UTC, Local, และ timezone อื่นๆ
- **Type-safe** - แยก types สำหรับ date, time, datetime
- **Comprehensive** - Formatting, parsing, arithmetic

## 🔧 Features ที่มี

| Feature    | คำอธิบาย                      |
| ---------- | ----------------------------- |
| `clock`    | เข้าถึงเวลาระบบ (default)     |
| `std`      | Standard library (default)    |
| `serde`    | Serialize/Deserialize support |
| `wasmbind` | WebAssembly support           |

## 📝 การใช้งานพื้นฐาน

### 1. เพิ่มใน Cargo.toml

```toml
[dependencies]
chrono = { version = "0.4", features = ["serde"] }
```

### 2. Get Current Time

```rust
use chrono::{Utc, Local};

// UTC time (แนะนำสำหรับ server)
let now_utc = Utc::now();
println!("{}", now_utc);  // 2026-01-07T10:00:00Z

// Local time
let now_local = Local::now();
println!("{}", now_local);  // 2026-01-07T17:00:00+07:00
```

### 3. Create DateTime

```rust
use chrono::{Utc, TimeZone, NaiveDate};

// จาก components
let dt = Utc.with_ymd_and_hms(2026, 1, 7, 10, 0, 0).unwrap();

// จาก NaiveDate
let naive = NaiveDate::from_ymd_opt(2026, 1, 7).unwrap();
let dt = naive.and_hms_opt(10, 0, 0).unwrap();
```

### 4. Duration (ช่วงเวลา)

```rust
use chrono::Duration;

let one_hour = Duration::hours(1);
let fifteen_minutes = Duration::minutes(15);
let one_week = Duration::days(7);
let half_second = Duration::milliseconds(500);
```

### 5. DateTime Arithmetic

```rust
use chrono::{Utc, Duration};

let now = Utc::now();

// บวก
let tomorrow = now + Duration::days(1);
let in_one_hour = now + Duration::hours(1);

// ลบ
let yesterday = now - Duration::days(1);

// หาความแตกต่าง
let diff = tomorrow - now;
println!("{} seconds", diff.num_seconds());
```

### 6. Formatting

```rust
use chrono::Utc;

let now = Utc::now();

// ISO 8601
println!("{}", now.to_rfc3339());
// 2026-01-07T10:00:00+00:00

// RFC 2822 (email format)
println!("{}", now.to_rfc2822());
// Tue, 07 Jan 2026 10:00:00 +0000

// Custom format
println!("{}", now.format("%Y-%m-%d %H:%M:%S"));
// 2026-01-07 10:00:00

println!("{}", now.format("%d/%m/%Y"));
// 07/01/2026

println!("{}", now.format("%A, %B %e, %Y"));
// Tuesday, January  7, 2026
```

### 7. Parsing

```rust
use chrono::{DateTime, Utc, NaiveDateTime};

// Parse ISO 8601
let dt = "2026-01-07T10:00:00Z".parse::<DateTime<Utc>>()?;

// Parse custom format
let naive = NaiveDateTime::parse_from_str(
    "2026-01-07 10:00:00",
    "%Y-%m-%d %H:%M:%S"
)?;
```

### 8. Access Components

```rust
use chrono::{Utc, Datelike, Timelike};

let now = Utc::now();

// Date parts
println!("Year: {}", now.year());
println!("Month: {}", now.month());
println!("Day: {}", now.day());
println!("Weekday: {:?}", now.weekday());

// Time parts
println!("Hour: {}", now.hour());
println!("Minute: {}", now.minute());
println!("Second: {}", now.second());
```

### 9. Compare DateTime

```rust
use chrono::Utc;

let dt1 = Utc::now();
let dt2 = dt1 + chrono::Duration::hours(1);

if dt2 > dt1 {
    println!("dt2 is later");
}

if dt1 < dt2 {
    println!("dt1 is earlier");
}
```

### 10. Timestamp

```rust
use chrono::Utc;

let now = Utc::now();

// Unix timestamp (seconds)
let timestamp = now.timestamp();

// Unix timestamp (milliseconds)
let timestamp_millis = now.timestamp_millis();

// From timestamp
let dt = DateTime::from_timestamp(1704628800, 0).unwrap();
```

## 📚 Types

| Type                    | คำอธิบาย                  |
| ----------------------- | ------------------------- |
| `DateTime<Utc>`         | วันเวลา UTC               |
| `DateTime<Local>`       | วันเวลาท้องถิ่น           |
| `DateTime<FixedOffset>` | วันเวลากับ fixed timezone |
| `NaiveDateTime`         | วันเวลาไม่มี timezone     |
| `NaiveDate`             | วันที่ไม่มี timezone      |
| `NaiveTime`             | เวลาไม่มี timezone        |
| `Duration`              | ช่วงเวลา                  |

## 📝 Format Specifiers

| Specifier | ความหมาย         | ตัวอย่าง |
| --------- | ---------------- | -------- |
| `%Y`      | Year (4 digits)  | 2026     |
| `%m`      | Month (01-12)    | 01       |
| `%d`      | Day (01-31)      | 07       |
| `%H`      | Hour 24h (00-23) | 10       |
| `%M`      | Minute (00-59)   | 30       |
| `%S`      | Second (00-59)   | 00       |
| `%A`      | Weekday name     | Tuesday  |
| `%B`      | Month name       | January  |
| `%Z`      | Timezone name    | UTC      |
| `%z`      | Timezone offset  | +0700    |

## 🔗 Ecosystem

- **serde** - Serialize/Deserialize
- **sqlx** - Database TIMESTAMP
- **jsonwebtoken** - Token expiration
