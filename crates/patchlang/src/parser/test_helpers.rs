//! Shared test helpers for parser tests.

use crate::ast::KvValue;

pub fn kv_str(kv: &crate::ast::KeyValue) -> &str {
    match &kv.value {
        KvValue::Str { value } => value.as_str(),
        other => panic!("expected Str, got {other:?}"),
    }
}

pub fn kv_num(kv: &crate::ast::KeyValue) -> u32 {
    match &kv.value {
        KvValue::Num { value } => *value,
        other => panic!("expected Num, got {other:?}"),
    }
}
