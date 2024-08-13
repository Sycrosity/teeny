use crate::prelude::*;

#[derive(Debug)]
pub struct AuthParams {
    pub response_type: &'static str,
    pub client_id: &'static str,
    pub scope: &'static str,
    pub code_challenge: String<64>,
    pub code_challenge_method: &'static str,
    pub redirect_uri: &'static str,
}

impl AuthParams {
    pub fn to_string(&self) -> String<256> {
        let mut query_string = String::new();
        query_string.push_str("?").unwrap();
        query_string.push_str("response_type=").unwrap();
        query_string.push_str(self.response_type).unwrap();
        query_string.push_str("&client_id=").unwrap();
        query_string.push_str(self.client_id).unwrap();
        query_string.push_str("&scope=").unwrap();
        query_string.push_str(self.scope).unwrap();
        query_string.push_str("&code_challenge=").unwrap();
        query_string.push_str(self.code_challenge.as_str()).unwrap();
        query_string.push_str("&code_challenge_method=").unwrap();
        query_string.push_str(self.code_challenge_method).unwrap();
        query_string.push_str("&redirect_uri=").unwrap();
        query_string.push_str(self.redirect_uri).unwrap();
        query_string
    }
}
