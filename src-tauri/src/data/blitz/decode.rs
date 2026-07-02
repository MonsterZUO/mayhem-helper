//! Databricks 列式 payload 解码。
//!
//! payload 形如：
//! `{ manifest: { schema: { columns: [{name,position,typeName}] } }, result: { dataArray: [[...]] } }`
//! 每行是按列位置排列的值数组；按 typeName 转换，STRUCT/MAP/ARRAY 递归解析内嵌 JSON。

use serde_json::Value;
use std::collections::HashMap;

/// 解码 Databricks payload 为按列名索引的行。
/// 缺 schema 时 fail-loud 报错（视为损坏），无数据行时返回空 Vec（正常空结果）。
pub fn decode_databricks_rows(payload: &Value) -> Result<Vec<HashMap<String, Value>>, String> {
    let columns = payload
        .pointer("/manifest/schema/columns")
        .and_then(|c| c.as_array())
        .ok_or("Blitz payload 缺 manifest.schema.columns（疑似损坏）")?;

    let cols: Vec<(usize, String, String)> = columns
        .iter()
        .filter_map(|c| {
            let name = c.get("name")?.as_str()?.to_string();
            let pos = c.get("position")?.as_u64()? as usize;
            let ty = c
                .get("typeName")
                .and_then(|t| t.as_str())
                .unwrap_or("STRING")
                .to_string();
            Some((pos, name, ty))
        })
        .collect();

    let rows = match payload
        .pointer("/result/dataArray")
        .and_then(|d| d.as_array())
    {
        Some(rows) => rows,
        None => return Ok(Vec::new()),
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let arr = row.as_array().ok_or("Blitz dataArray 行不是数组")?;
        let mut map = HashMap::with_capacity(cols.len());
        for (pos, name, ty) in &cols {
            let raw = arr.get(*pos).cloned().unwrap_or(Value::Null);
            map.insert(name.clone(), cast_value(ty, raw));
        }
        out.push(map);
    }
    Ok(out)
}

/// 按 Databricks typeName 转换值。STRUCT/MAP/ARRAY 递归解析内嵌 JSON 字符串。
fn cast_value(type_name: &str, value: Value) -> Value {
    if value.is_null() {
        return value;
    }
    match type_name {
        "BOOLEAN" => {
            let b = value.as_bool().unwrap_or_else(|| value.as_str() == Some("true"));
            Value::Bool(b)
        }
        "DOUBLE" | "BIGINT" | "LONG" | "INT" => match &value {
            Value::Number(_) => value,
            Value::String(s) => s
                .parse::<f64>()
                .ok()
                .and_then(serde_json::Number::from_f64)
                .map(Value::Number)
                .unwrap_or(value),
            _ => value,
        },
        "MAP" | "ARRAY" | "STRUCT" => recursive_json_parse(value),
        _ => value,
    }
}

/// 递归解析内嵌的 JSON 字符串。Blitz 的 `data`(STRUCT) 列值是一段 JSON 字符串套 JSON。
fn recursive_json_parse(value: Value) -> Value {
    match value {
        Value::String(s) => match serde_json::from_str::<Value>(&s) {
            Ok(parsed @ Value::Object(_)) | Ok(parsed @ Value::Array(_)) => {
                recursive_json_parse(parsed)
            }
            Ok(parsed) => parsed,
            Err(_) => Value::String(s),
        },
        Value::Array(a) => Value::Array(a.into_iter().map(recursive_json_parse).collect()),
        Value::Object(o) => {
            Value::Object(o.into_iter().map(|(k, v)| (k, recursive_json_parse(v))).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture_payload() -> Value {
        let raw: Value =
            serde_json::from_str(include_str!("fixtures/mayhem_champion_5.json")).unwrap();
        raw.pointer("/data/executeDatabricksQuery/payload")
            .expect("fixture 应含 payload")
            .clone()
    }

    #[test]
    fn decodes_real_fixture() {
        let rows = decode_databricks_rows(&fixture_payload()).unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.get("champion_id").and_then(|v| v.as_str()), Some("5"));

        let data = row.get("data").expect("有 data 列");
        assert!(
            data.get("augments").map(Value::is_object).unwrap_or(false),
            "augments 应解析为对象"
        );
        assert!(data.get("items").is_some(), "应有 items");
        assert!(data.get("augment_trios").is_some(), "应有 augment_trios");
        assert!(
            row.get("patch")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false),
            "patch 应非空"
        );
    }

    #[test]
    fn empty_data_array_yields_empty() {
        let payload = json!({
            "manifest": {"schema": {"columns": [
                {"name": "champion_id", "position": 0, "typeName": "STRING"}
            ]}},
            "result": {"dataArray": []}
        });
        assert!(decode_databricks_rows(&payload).unwrap().is_empty());
    }

    #[test]
    fn missing_manifest_is_error() {
        let payload = json!({ "result": { "dataArray": [["5"]] } });
        assert!(decode_databricks_rows(&payload).is_err());
    }

    #[test]
    fn parses_nested_struct_string() {
        let payload = json!({
            "manifest": {"schema": {"columns": [
                {"name": "data", "position": 0, "typeName": "STRUCT"}
            ]}},
            "result": {"dataArray": [
                ["{\"augments\":{\"1\":{\"win_rate\":\"0.5\"}}}"]
            ]}
        });
        let rows = decode_databricks_rows(&payload).unwrap();
        assert!(rows[0]
            .get("data")
            .and_then(|d| d.get("augments"))
            .is_some());
    }

    #[test]
    fn casts_numeric_strings() {
        let payload = json!({
            "manifest": {"schema": {"columns": [
                {"name": "n", "position": 0, "typeName": "DOUBLE"}
            ]}},
            "result": {"dataArray": [["0.516605"]]}
        });
        let rows = decode_databricks_rows(&payload).unwrap();
        assert_eq!(rows[0].get("n").and_then(Value::as_f64), Some(0.516605));
    }
}
