// Copyright Kamu Data, Inc. and contributors. All rights reserved.
//
// Use of this software is governed by the Business Source License
// included in the LICENSE file.
//
// As of the Change Date specified in that file, in accordance with
// the Business Source License, use of this software will be governed
// by the Apache License, Version 2.0.

// TODO: Knowing the response schema we could make CBOR transcoding utilize
// standard tags for date-times, bignum, and other data types. An even better
// option would be to support CBOR directly in the API server. But do we want
// this complexity in Solidity? Perhaps we should use schema to simplify dates
// to unix timestamps.
//
// See: https://www.rfc-editor.org/rfc/rfc8949.html#name-tagging-of-items
pub(crate) fn json_to_cbor(value: serde_json::Value) -> ciborium::Value {
    match value {
        serde_json::Value::Null => ciborium::Value::Null,
        serde_json::Value::Bool(v) => ciborium::Value::Bool(v),
        serde_json::Value::Number(v) if v.is_u64() => {
            ciborium::Value::Integer(v.as_u64().unwrap().into())
        }
        serde_json::Value::Number(v) if v.is_i64() => {
            ciborium::Value::Integer(v.as_i64().unwrap().into())
        }
        serde_json::Value::Number(v) if v.is_f64() => ciborium::Value::Float(v.as_f64().unwrap()),
        serde_json::Value::Number(_) => unreachable!(),
        serde_json::Value::String(v) => ciborium::Value::Text(v),
        serde_json::Value::Array(v) => {
            ciborium::Value::Array(v.into_iter().map(json_to_cbor).collect())
        }
        serde_json::Value::Object(v) => ciborium::Value::Map(
            v.into_iter()
                .map(|(k, v)| (ciborium::Value::Text(k), json_to_cbor(v)))
                .collect(),
        ),
    }
}
