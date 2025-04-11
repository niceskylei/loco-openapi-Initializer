use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::{controller::routes::LocoMethodRouter, prelude::*};
use utoipa::openapi::OpenApi;
use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouterExt};
#[cfg(feature = "redoc")]
use utoipa_redoc::{Redoc, Servable};
#[cfg(feature = "scalar")]
use utoipa_scalar::{Scalar, Servable as ScalarServable};
#[cfg(feature = "swagger")]
use utoipa_swagger_ui::SwaggerUi;

use config::{get_openapi_config, set_openapi_config, OpenAPIType};

pub mod config;
pub mod openapi;

/// Loco initializer for OpenAPI with custom initial spec setup.
pub struct OpenapiInitializerWithSetup<F: Fn(&AppContext) -> OpenApi + Send + Sync + 'static> {
    /// Custom setup for the initial OpenAPI spec, if any.
    initial_spec: Option<F>,
}

/// Loco initializer for OpenAPI without custom setup.
impl<F: Fn(&AppContext) -> OpenApi + Send + Sync + 'static> OpenapiInitializerWithSetup<F> {
    #[inline(always)]
    #[must_use]
    pub fn new(initial_spec: F) -> Self {
        Self {
            initial_spec: Some(initial_spec),
        }
    }
}

impl Default for OpenapiInitializerWithSetup<fn(&AppContext) -> OpenApi> {
    fn default() -> Self {
        Self { initial_spec: None }
    }
}

#[async_trait]
impl<F: Fn(&AppContext) -> OpenApi + Send + Sync + 'static> Initializer
    for OpenapiInitializerWithSetup<F>
{
    fn name(&self) -> String {
        "openapi".to_string()
    }

    async fn after_routes(&self, mut router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        set_openapi_config(ctx)?;
        let list_routes = match ctx.app_routes.as_ref() {
            Some(routes) => routes.collect(),
            _ => return Ok(router),
        };

        let mut api_router: OpenApiRouter<AppContext> =
            if let Some(ref custom_spec_fn) = self.initial_spec {
                OpenApiRouter::with_openapi(custom_spec_fn(ctx))
            } else {
                OpenApiRouter::new()
            };

        for route in list_routes {
            match route.method {
                LocoMethodRouter::Axum(_) => continue,
                LocoMethodRouter::Utoipa(method) => {
                    api_router = api_router.routes(method.with_state(ctx.clone()))
                }
            }
        }

        // Collect the OpenAPI spec
        let (_, open_api_spec) = api_router.split_for_parts();
        openapi::set_openapi_spec(open_api_spec);

        // Serve the OpenAPI spec using the enabled OpenAPI visualizers
        #[cfg(feature = "redoc")]
        {
            if let Some(OpenAPIType::Redoc {
                url,
                spec_json_url,
                spec_yaml_url,
            }) = get_openapi_config()
            {
                router = router.merge(Redoc::with_url(url, openapi::get_openapi_spec().clone()));
                router = openapi::add_openapi_endpoints(router, spec_json_url, spec_yaml_url);
            }
        }

        #[cfg(feature = "scalar")]
        {
            if let Some(OpenAPIType::Scalar {
                url,
                spec_json_url,
                spec_yaml_url,
            }) = get_openapi_config()
            {
                router = router.merge(Scalar::with_url(url, openapi::get_openapi_spec().clone()));
                router = openapi::add_openapi_endpoints(router, spec_json_url, spec_yaml_url);
            }
        }

        #[cfg(feature = "swagger")]
        {
            if let Some(OpenAPIType::Swagger {
                url,
                spec_json_url,
                spec_yaml_url,
            }) = get_openapi_config()
            {
                router = router.merge(
                    SwaggerUi::new(url)
                        .url(spec_json_url.clone(), openapi::get_openapi_spec().clone()),
                );
                router = openapi::add_openapi_endpoints(router, &None, spec_yaml_url);
            }
        }

        Ok(router)
    }
}
