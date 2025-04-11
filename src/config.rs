use std::sync::OnceLock;

use loco_rs::{app::AppContext, Error};
use serde::{Deserialize, Serialize};

static OPENAPI_CONFIG: OnceLock<Option<OpenAPIType>> = OnceLock::new();

pub fn set_openapi_config(ctx: &AppContext) -> Result<Option<&OpenAPIType>, Error> {
    let json = ctx
        .config
        .initializers
        .as_ref()
        .and_then(|m| m.get("openapi"))
        .cloned()
        .unwrap_or_default();
    let config: Option<OpenAPIType> = serde_json::from_value(json)?;

    Ok(OPENAPI_CONFIG.get_or_init(|| config).as_ref())
}

pub fn get_openapi_config() -> Option<&'static OpenAPIType> {
    OPENAPI_CONFIG.get().unwrap_or(&None).as_ref()
}

/// `OpenAPI` configuration
#[cfg(any(feature = "swagger", feature = "redoc", feature = "scalar"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAPI {
    /// Redoc configuration
    /// Example:
    /// ```yaml
    /// redoc:
    ///   !Redoc
    ///     url: /redoc
    /// ```
    pub redoc: Option<OpenAPIType>,
    /// Scalar configuration
    /// Example:
    /// ```yaml
    /// scalar:
    ///   !Scalar
    ///     url: /scalar
    /// ```
    pub scalar: Option<OpenAPIType>,
    /// Swagger configuration
    /// Example:
    /// ```yaml
    /// swagger:
    ///  !Swagger
    ///    url: /swagger
    ///    spec_json_url: /openapi.json
    /// ```
    pub swagger: Option<OpenAPIType>,
}

#[cfg(any(feature = "swagger", feature = "redoc", feature = "scalar"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OpenAPIType {
    Redoc {
        /// URL for where to host the redoc `OpenAPI` spec, example: /redoc
        url: String,
        /// URL for openapi.json, for example: /openapi.json
        spec_json_url: Option<String>,
        /// URL for openapi.yaml, for example: /openapi.yaml
        spec_yaml_url: Option<String>,
    },
    Scalar {
        /// URL for where to host the swagger `OpenAPI` spec, example: /scalar
        url: String,
        /// URL for openapi.json, for example: /openapi.json
        spec_json_url: Option<String>,
        /// URL for openapi.yaml, for example: /openapi.yaml
        spec_yaml_url: Option<String>,
    },
    Swagger {
        /// URL for where to host the swagger `OpenAPI` spec, example:
        /// /swagger-ui
        url: String,
        /// URL for openapi.json, for example: /api-docs/openapi.json
        spec_json_url: String,
        /// URL for openapi.yaml, for example: /openapi.yaml
        spec_yaml_url: Option<String>,
    },
}
