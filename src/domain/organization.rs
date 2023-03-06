use serde::Deserialize;
use validator::Validate;

const MIN_NAME_LENGTH: u8 = 2;
const MAX_NAME_LENGTH: u8 = 90;

#[derive(Debug, Validate, Deserialize)]
pub struct OrgUpdateForm {
    #[validate(length(min = "MIN_NAME_LENGTH", max = "MAX_NAME_LENGTH"))]
    pub name: String,
}
