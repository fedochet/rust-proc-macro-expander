use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExpansionTask {
    /// Argument of macro call.
    ///
    /// In custom derive that would be a struct or enum; in attribute-like macro - underlying
    /// item; in function-like macro - the macro body.
    pub macro_body: String,

    /// Names of macros to expand.
    ///
    /// In custom derive those are names of derived traits (`Serialize`, `Getters`, etc.). In
    /// attribute-like and functiona-like macros - single name of macro itself (`show_streams`).
    pub macro_name: String,

    /// Possible attributes for the attribute-like macros.
    pub attributes: Option<String>,

    pub libs: Vec<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ExpansionResult {
    #[serde(rename = "success")]
    Success { expansion: String },
    #[serde(rename = "error")]
    Error { reason: String },
}
