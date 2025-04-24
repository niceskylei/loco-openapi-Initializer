use async_trait::async_trait;
use loco_openapi::prelude::routes;
use loco_openapi::{
    auth::{set_jwt_location, SecurityAddon},
    prelude::openapi, // Make sure openapi macro is imported
};
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    prelude::*,
    task::Tasks,
};
use serde::Serialize; // Added import for Album
use serde_json::{json, Value};
use std::collections::BTreeMap;
use utoipa::{OpenApi, ToSchema}; // Added ToSchema
                                 // Define a minimal TestApp
use insta::assert_snapshot;
struct TestApp;

// --- Start: Embedded Album Controller ---
mod album {
    use super::*; // Allow using imports from parent module
    use axum::debug_handler;
    use axum::routing::get;

    #[derive(Serialize, Debug, ToSchema)]
    pub struct Album {
        title: String,
        rating: u32,
    }

    /// Get album
    ///
    /// Returns a title and rating
    #[utoipa::path(
        get,
        path = "/api/album/get_album",
        tags = ["album"],
        responses(
            (status = 200, description = "Album found", body = Album),
        ),
    )]
    #[debug_handler]
    pub async fn get_album(State(_ctx): State<AppContext>) -> Result<Response> {
        format::json(Album {
            title: "VH II".to_string(),
            rating: 10,
        })
    }

    pub fn routes() -> Routes {
        Routes::new()
            .prefix("api/album")
            .add("/get_album", openapi(get(get_album), routes!(get_album)))
    }
}
// --- End: Embedded Album Controller ---

// Helper to create test configuration
fn config_test() -> Config {
    let mut config = loco_rs::tests_cfg::config::test_config();
    let mut initializers = BTreeMap::new();
    let mut openapi_conf = serde_json::Map::new();

    // Configure endpoints to match test requests
    openapi_conf.insert(
        "redoc".to_string(),
        json!({
            "redoc": {
                "url": "/redoc"
            }
        }),
    );
    openapi_conf.insert(
        "scalar".to_string(),
        json!({
            "scalar": {
                "url": "/scalar"
            }
        }),
    );
    openapi_conf.insert(
        "swagger".to_string(),
        json!({
            "swagger": {
                "url": "/swagger", // Ensure this matches the test URL
                "spec_json_url": "/api-docs/openapi.json" // Required for swagger
            }
        }),
    );

    initializers.insert("openapi".to_string(), Value::Object(openapi_conf));
    config.initializers = Some(initializers);
    config
}

// Implement Hooks for TestApp
#[async_trait]
impl Hooks for TestApp {
    fn app_name() -> &'static str {
        "loco-openapi-test"
    }
    fn app_version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes().add_route(album::routes()) // Add album routes
    }

    async fn load_config(_environment: &Environment) -> Result<Config> {
        Ok(config_test())
    }

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![Box::new(
            loco_openapi::OpenapiInitializerWithSetup::new(
                |ctx| {
                    #[derive(OpenApi)]
                    #[openapi(
                        modifiers(&SecurityAddon),
                        paths(album::get_album), // Add album path to OpenAPI spec
                        components(schemas(album::Album)), // Add album schema
                        info(
                            title = "Loco Demo Test",
                            description = "Test OpenAPI spec for loco-openapi"
                        )
                    )]
                    struct ApiDoc;
                    set_jwt_location(ctx.into());

                    ApiDoc::openapi()
                },
                None,
            ),
        )])
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        // Assuming Migrator is not needed as per previous iteration
        create_app::<Self>(mode, environment, config).await
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    // Removed truncate and seed as they are not part of the Hooks trait
}

// Test for OpenAPI UI Endpoints
#[tokio::test]
async fn test_openapi_ui_endpoints() {
    loco_rs::testing::request::request::<TestApp, _, _>(|rq, _ctx| async move {
        // Test Redoc endpoint
        let res_redoc = rq.get("/redoc").await;
        assert_eq!(
            res_redoc.status_code(),
            200,
            "Expected /redoc to return 200 OK: {}",
            res_redoc.text()
        );

        assert_snapshot!("redoc", res_redoc.text());

        // Test Scalar endpoint
        let res_scalar = rq.get("/scalar").await;
        assert_eq!(
            res_scalar.status_code(),
            200,
            "Expected /scalar to return 200 OK: {}",
            res_scalar.text()
        );

        assert_snapshot!("scalar", res_scalar.text());

        let res_swagger = rq.get("/swagger/").await;
        assert_eq!(
            res_swagger.status_code(),
            200,
            "Expected /swagger to return 200 OK: {}",
            res_swagger.text()
        );

        assert_snapshot!("swagger", res_swagger.text());
    })
    .await;
}
