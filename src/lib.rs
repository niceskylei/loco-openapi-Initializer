use async_trait::async_trait;
use axum::Router as AxumRouter;
use loco_rs::prelude::*;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
#[cfg(feature = "redoc")]
use utoipa_redoc::{Redoc, Servable};
#[cfg(feature = "scalar")]
use utoipa_scalar::{Scalar, Servable as ScalarServable};
#[cfg(feature = "swagger")]
use utoipa_swagger_ui::SwaggerUi;

use config::{get_openapi_config, set_openapi_config, OpenAPIType};

pub mod auth;
pub mod config;
pub mod openapi;
pub mod prelude;

type RouterList = Vec<OpenApiRouter<AppContext>>;
type InitialSpec = dyn Fn(&AppContext) -> OpenApi + Send + Sync + 'static;

/// Loco initializer for OpenAPI with custom initial spec setup
pub struct OpenapiInitializerWithSetup {
    /// Custom setup for the initial OpenAPI spec, if any
    initial_spec: Option<Box<InitialSpec>>,
    /// Routes to add to the OpenAPI spec
    routes_setup: Option<RouterList>,
}

impl OpenapiInitializerWithSetup {
    #[inline(always)]
    #[must_use]
    pub fn new<F>(initial_spec: F, routes_setup: RouterList) -> Self
    where
        F: Fn(&AppContext) -> OpenApi + Send + Sync + 'static,
    {
        Self {
            initial_spec: Some(Box::new(initial_spec)),
            routes_setup: Some(routes_setup),
        }
    }
}

impl Default for OpenapiInitializerWithSetup {
    fn default() -> Self {
        Self {
            initial_spec: None,
            routes_setup: None,
        }
    }
}

#[async_trait]
impl Initializer for OpenapiInitializerWithSetup {
    fn name(&self) -> String {
        "openapi".to_string()
    }

    async fn after_routes(&self, mut router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        set_openapi_config(ctx)?;

        let mut api_router: OpenApiRouter<AppContext> =
            if let Some(ref custom_spec_fn) = self.initial_spec {
                OpenApiRouter::with_openapi(custom_spec_fn(ctx))
            } else {
                OpenApiRouter::new()
            };

        // Merge all routers to be added to the OpenAPI spec
        if let Some(ref routes_setup) = self.routes_setup {
            for route in routes_setup {
                api_router = api_router.merge(route.clone());
            }
        }

        // Collect the OpenAPI spec
        let (_, open_api_spec) = api_router.split_for_parts();
        openapi::set_openapi_spec(open_api_spec);

        let open_api_config = if let Some(open_api_config) = get_openapi_config() {
            open_api_config
        } else {
            return Ok(router);
        };

        // Serve the OpenAPI spec using the enabled OpenAPI visualizers
        #[cfg(feature = "redoc")]
        if let Some(OpenAPIType::Redoc {
            url,
            spec_json_url,
            spec_yaml_url,
        }) = &open_api_config.redoc
        {
            router = router.merge(Redoc::with_url(url, openapi::get_openapi_spec().clone()));
            router = openapi::add_openapi_endpoints(router, spec_json_url, spec_yaml_url);
        }

        #[cfg(feature = "scalar")]
        if let Some(OpenAPIType::Scalar {
            url,
            spec_json_url,
            spec_yaml_url,
        }) = &open_api_config.scalar
        {
            router = router.merge(Scalar::with_url(url, openapi::get_openapi_spec().clone()));
            router = openapi::add_openapi_endpoints(router, spec_json_url, spec_yaml_url);
        }

        #[cfg(feature = "swagger")]
        if let Some(OpenAPIType::Swagger {
            url,
            spec_json_url,
            spec_yaml_url,
        }) = &open_api_config.swagger
        {
            router = router.merge(
                SwaggerUi::new(url).url(spec_json_url.clone(), openapi::get_openapi_spec().clone()),
            );
            router = openapi::add_openapi_endpoints(router, &None, spec_yaml_url);
        }

        Ok(router)
    }
}
