use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ReportConfig {
    #[serde(default)]
    pub fields: Vec<String>,
    #[serde(default)]
    pub filters: Vec<ReportFilter>,
    #[serde(default)]
    pub group_by: Vec<String>,
    #[serde(default)]
    pub sort: Vec<ReportSort>,
    #[serde(default)]
    pub aggregates: Vec<ReportAggregate>,
    #[serde(default)]
    pub calculated: Vec<CalculatedColumn>,
    #[serde(default)]
    pub date_range: Option<DateRange>,
    #[serde(default)]
    pub date_field: Option<String>,
    #[serde(default)]
    pub relation_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportFilter {
    pub field: String,
    pub op: String,
    #[serde(default)]
    pub value: Value,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportSort {
    pub field: String,
    #[serde(default)]
    pub direction: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportAggregate {
    pub field: String,
    pub op: String,
    #[serde(default)]
    pub alias: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CalculatedColumn {
    pub name: String,
    pub expression: String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DateRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportResult {
    pub columns: Vec<String>,
    pub rows: Vec<Value>,
    pub total: usize,
}

fn valid_name(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
fn scalar(v: &Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}
fn get_value(row: &Value, field: &str) -> Value {
    if let Some(v) = row.get(field) {
        return v.clone();
    }
    let mut current = vec![row.clone()];
    for key in field.split('.') {
        let mut next = Vec::new();
        for value in current {
            match value {
                Value::Object(o) => {
                    if let Some(v) = o.get(key) {
                        next.push(v.clone());
                    }
                }
                Value::Array(items) => {
                    for item in items {
                        if let Value::Object(o) = item {
                            if let Some(v) = o.get(key) {
                                next.push(v.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        current = next;
        if current.is_empty() {
            return Value::Null;
        }
    }
    if current.len() == 1 {
        current.remove(0)
    } else {
        Value::Array(current)
    }
}
fn cmp(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (scalar(a), scalar(b)) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
        _ => a.to_string().cmp(&b.to_string()),
    }
}
fn matches_filter(row: &Value, f: &ReportFilter) -> bool {
    let a = get_value(row, &f.field);
    let b = &f.value;
    match f.op.as_str() {
        "eq" | "=" => a == *b,
        "neq" | "!=" => a != *b,
        "contains" => a.as_str().unwrap_or("").contains(b.as_str().unwrap_or("")),
        "starts_with" => a
            .as_str()
            .unwrap_or("")
            .starts_with(b.as_str().unwrap_or("")),
        "ends_with" => a.as_str().unwrap_or("").ends_with(b.as_str().unwrap_or("")),
        "gt" | ">" => cmp(&a, b) == std::cmp::Ordering::Greater,
        "gte" | ">=" => cmp(&a, b) != std::cmp::Ordering::Less,
        "lt" | "<" => cmp(&a, b) == std::cmp::Ordering::Less,
        "lte" | "<=" => cmp(&a, b) != std::cmp::Ordering::Greater,
        "in" => b
            .as_array()
            .map(|xs| xs.iter().any(|x| x == &a))
            .unwrap_or(false),
        "is_null" => a.is_null(),
        "not_null" => !a.is_null(),
        _ => false,
    }
}

fn aggregate(rows: &[Value], a: &ReportAggregate) -> Value {
    if a.op == "count" {
        return json!(rows.len());
    }
    let vals: Vec<f64> = rows
        .iter()
        .filter_map(|r| scalar(&get_value(r, &a.field)))
        .collect();
    match a.op.as_str() {
        "sum" => json!(vals.iter().sum::<f64>()),
        "avg" => json!(if vals.is_empty() {
            0.0
        } else {
            vals.iter().sum::<f64>() / vals.len() as f64
        }),
        "min" => vals
            .iter()
            .cloned()
            .reduce(f64::min)
            .map_or(Value::Null, |v| json!(v)),
        "max" => vals
            .iter()
            .cloned()
            .reduce(f64::max)
            .map_or(Value::Null, |v| json!(v)),
        _ => Value::Null,
    }
}

async fn validate_fields(
    pool: &SqlitePool,
    entity_id: &str,
    config: &ReportConfig,
    role: &str,
) -> Result<HashSet<String>> {
    let fields = crate::repository::list_fields(pool, entity_id).await?;
    let names: HashSet<String> = fields.iter().map(|f| f.name.clone()).collect();
    let mut referenced = Vec::new();
    for name in config
        .fields
        .iter()
        .chain(config.group_by.iter())
        .chain(config.sort.iter().map(|s| &s.field))
        .chain(config.filters.iter().map(|f| &f.field))
        .chain(
            config
                .aggregates
                .iter()
                .filter(|a| a.op != "count")
                .map(|a| &a.field),
        )
    {
        let base = name.split('.').next().unwrap_or(name);
        if !valid_name(base) || !names.contains(base) {
            return Err(anyhow!("unknown report field: {name}"));
        }
        if !name.contains('.') {
            referenced.push(base.to_string());
        }
    }
    if let Some(name) = &config.date_field {
        if !valid_name(name) || !names.contains(name) {
            return Err(anyhow!("unknown report date field: {name}"));
        }
        referenced.push(name.clone());
    }
    for name in referenced {
        crate::repository::check_field_permission(pool, entity_id, &name, role, false).await?;
    }
    if let Some(relation_id) = &config.relation_id {
        let ok: Option<String> = sqlx::query_scalar(
            "SELECT id FROM _meta_relation WHERE id = ? AND source_entity_id = ?",
        )
        .bind(relation_id)
        .bind(entity_id)
        .fetch_optional(pool)
        .await?;
        if ok.is_none() {
            return Err(anyhow!("report relation does not belong to entity"));
        }
    }
    for aggregate in &config.aggregates {
        if !matches!(
            aggregate.op.as_str(),
            "count" | "sum" | "avg" | "min" | "max"
        ) {
            return Err(anyhow!("unsupported report aggregate: {}", aggregate.op));
        }
    }
    for sort in &config.sort {
        if !sort.direction.is_empty()
            && !sort.direction.eq_ignore_ascii_case("asc")
            && !sort.direction.eq_ignore_ascii_case("desc")
        {
            return Err(anyhow!("unsupported report sort direction"));
        }
    }
    for calculated in &config.calculated {
        if !valid_name(&calculated.name) {
            return Err(anyhow!("invalid calculated column name"));
        }
        crate::formula::validate_syntax(&calculated.expression)?;
    }
    Ok(names)
}
async fn load_rows(
    pool: &SqlitePool,
    entity_id: &str,
    config: &ReportConfig,
) -> Result<Vec<Value>> {
    let mut rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id,payload FROM _doc WHERE entity_id = ? ORDER BY created_at",
    )
    .bind(entity_id)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(id, p)| {
        let mut v: Value = serde_json::from_str(&p).unwrap_or(Value::Object(Default::default()));
        if let Value::Object(ref mut o) = v {
            o.insert("_id".into(), json!(id));
        }
        v
    })
    .collect::<Vec<_>>();
    if let Some(relation_id) = &config.relation_id {
        let links = sqlx::query_as::<_, (String, String)>(
            "SELECT source_doc_id,target_doc_id FROM _meta_relation_link WHERE relation_id=?",
        )
        .bind(relation_id)
        .fetch_all(pool)
        .await?;
        let targets: HashMap<String,Value> = sqlx::query_as::<_,(String,String)>("SELECT id,payload FROM _doc WHERE entity_id=(SELECT target_entity_id FROM _meta_relation WHERE id=?)")
            .bind(relation_id).fetch_all(pool).await?.into_iter().filter_map(|(id,p)| serde_json::from_str(&p).ok().map(|v|(id,v))).collect();
        let mut by_source: HashMap<String, Vec<Value>> = HashMap::new();
        for (s, t) in links {
            if let Some(v) = targets.get(&t) {
                by_source.entry(s).or_default().push(v.clone());
            }
        }
        for row in &mut rows {
            let id = row.get("_id").and_then(Value::as_str).map(str::to_owned);
            if let Some(id) = id {
                if let Value::Object(o) = row {
                    o.insert(
                        "_relation".into(),
                        Value::Array(by_source.remove(&id).unwrap_or_default()),
                    );
                }
            }
        }
    }
    Ok(rows)
}

pub async fn run(
    pool: &SqlitePool,
    entity_id: &str,
    config: &ReportConfig,
    _role: &str,
) -> Result<ReportResult> {
    validate_fields(pool, entity_id, config, _role).await?;
    if config.filters.iter().any(|f| {
        !matches!(
            f.op.as_str(),
            "eq" | "="
                | "neq"
                | "!="
                | "contains"
                | "starts_with"
                | "ends_with"
                | "gt"
                | ">"
                | "gte"
                | ">="
                | "lt"
                | "<"
                | "lte"
                | "<="
                | "in"
                | "is_null"
                | "not_null"
        )
    }) {
        return Err(anyhow!("unsupported report filter operator"));
    }
    let mut rows = load_rows(pool, entity_id, config).await?;
    rows.retain(|r| config.filters.iter().all(|f| matches_filter(r, f)));
    if let Some(dr) = &config.date_range {
        if let Some(df) = &config.date_field {
            rows.retain(|r| {
                let date_value = get_value(r, df);
                let x = date_value.as_str().unwrap_or("");
                dr.from.as_deref().is_none_or(|v| x >= v) && dr.to.as_deref().is_none_or(|v| x <= v)
            });
        }
    }
    rows.sort_by(|a, b| {
        for s in &config.sort {
            let o = cmp(&get_value(a, &s.field), &get_value(b, &s.field));
            if o != std::cmp::Ordering::Equal {
                return if s.direction.eq_ignore_ascii_case("desc") {
                    o.reverse()
                } else {
                    o
                };
            }
        }
        std::cmp::Ordering::Equal
    });
    let mut out = Vec::new();
    if config.group_by.is_empty() && config.aggregates.is_empty() {
        for r in rows {
            let mut o = serde_json::Map::new();
            for f in &config.fields {
                o.insert(f.clone(), get_value(&r, f));
            }
            for c in &config.calculated {
                let vars = r
                    .as_object()
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect();
                o.insert(
                    c.name.clone(),
                    crate::formula::evaluate(&c.expression, &vars)?,
                );
            }
            out.push(Value::Object(o));
        }
    } else {
        let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
        for r in rows {
            let key = config
                .group_by
                .iter()
                .map(|f| get_value(&r, f).to_string())
                .collect::<Vec<_>>()
                .join("\u{1f}");
            groups.entry(key).or_default().push(r);
        }
        for (_, rs) in groups {
            let mut o = serde_json::Map::new();
            for f in &config.group_by {
                o.insert(f.clone(), get_value(&rs[0], f));
            }
            for a in &config.aggregates {
                o.insert(
                    a.alias
                        .clone()
                        .unwrap_or_else(|| format!("{}_{}", a.op, a.field)),
                    aggregate(&rs, a),
                );
            }
            out.push(Value::Object(o));
        }
    }
    let mut cols = config.fields.clone();
    cols.extend(config.group_by.clone());
    cols.extend(config.aggregates.iter().map(|a| {
        a.alias
            .clone()
            .unwrap_or_else(|| format!("{}_{}", a.op, a.field))
    }));
    cols.extend(config.calculated.iter().map(|c| c.name.clone()));
    cols.sort();
    cols.dedup();
    let total = out.len();
    Ok(ReportResult {
        columns: cols,
        rows: out,
        total,
    })
}

#[cfg(test)]
mod report_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn nested_relation_values_are_resolved() {
        let row = json!({"_relation": [{"name": "A"}, {"name": "B"}]});
        assert_eq!(get_value(&row, "_relation.name"), json!(["A", "B"]));
    }

    #[test]
    fn filters_and_aggregates_are_deterministic() {
        let rows = vec![json!({"amount": 10}), json!({"amount": 20})];
        assert!(matches_filter(
            &rows[0],
            &ReportFilter {
                field: "amount".into(),
                op: "gte".into(),
                value: json!(10)
            }
        ));
        assert_eq!(
            aggregate(
                &rows,
                &ReportAggregate {
                    field: "amount".into(),
                    op: "sum".into(),
                    alias: None
                }
            ),
            json!(30.0)
        );
    }

    #[test]
    fn report_field_names_are_safe() {
        assert!(valid_name("customer_id"));
        assert!(!valid_name("customer.id"));
        assert!(!valid_name("customer-id"));
    }
}
