use crate::types::{GateError, GateRequest, GateResponse};

pub fn parse_request(data: &[u8]) -> Result<GateRequest, GateError> {
    if data.len() < 4 {
        return Err("frame too short: need 4-byte length prefix".into());
    }
    let len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + len {
        return Err(format!(
            "frame truncated: header says {} bytes, got {}",
            len,
            data.len() - 4
        )
        .into());
    }
    let json = &data[4..4 + len];
    Ok(serde_json::from_slice(json)?)
}

pub fn serialize_response(response: &GateResponse) -> Vec<u8> {
    let json = serde_json::to_vec(response).expect("GateResponse serialization");
    let len = json.len() as u32;
    let mut out = len.to_be_bytes().to_vec();
    out.extend_from_slice(&json);
    out
}
