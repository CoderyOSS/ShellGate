use crate::types::{GateError, GateRequest, GateResponse};

/// Parse a JSON length-prefixed frame into a GateRequest.
///
/// Spec: gate-server/spec.md > "Unix socket server for command authorization"
/// Tasks: 4.4
/// Pure function — deserializes JSON.
pub fn parse_request(data: &[u8]) -> Result<GateRequest, GateError> {
    todo!("parse_request: read 4-byte length prefix, decode JSON body into GateRequest")
}

/// Serialize a GateResponse into a JSON length-prefixed frame.
///
/// Spec: gate-server/spec.md > "Unix socket server for command authorization"
/// Tasks: 4.4
/// Pure function — serializes JSON with length prefix.
pub fn serialize_response(response: &GateResponse) -> Vec<u8> {
    todo!("serialize_response: serialize GateResponse to JSON, prepend 4-byte length prefix")
}
