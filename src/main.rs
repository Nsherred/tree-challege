mod node;
mod tree;
mod tree_store;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use tree_store::TreeStore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // actix will spin up a thread pool.
    // We have to ensure that the Arc is created outside of the lambda.
    let tree_store = web::Data::new(TreeStore::default());

    HttpServer::new(move || App::new().configure(|cfg| setup_app(cfg, tree_store.clone())))
        .bind(("127.0.0.1", 3001))?
        .run()
        .await
}

fn setup_app(cfg: &mut web::ServiceConfig, tree_store: web::Data<TreeStore>) {
    cfg.app_data(tree_store).service(
        web::scope("/api/tree")
            .route("", web::get().to(get_tree))
            .route("", web::post().to(add_node)),
    );
}

async fn get_tree(tree_store: web::Data<TreeStore>) -> impl Responder {
    match tree_store.get_tree() {
        Ok(tree) => HttpResponse::Ok().json(tree),
        Err(error) => HttpResponse::InternalServerError().body(error.to_string()),
    }
}

#[derive(Deserialize, Serialize)]
struct AddNodeRequest {
    label: String,
    parent_id: Option<i32>,
}

async fn add_node(
    payload: web::Json<AddNodeRequest>,
    tree_store: web::Data<TreeStore>,
) -> impl Responder {
    let payload = payload.into_inner();

    if let Err(result) = tree_store.add_node(payload.label, payload.parent_id) {
        println!("error adding node: {:?}", result);
        return HttpResponse::BadRequest().body(result.message);
    }

    match tree_store.get_tree() {
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
        Ok(result) => return HttpResponse::Ok().json(result),
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::{test, web::Bytes};
    use serde_json::json;

    macro_rules! test_app {
        ( ) => {{
            {
                let tree_store = web::Data::new(TreeStore::default());
                let cfg = App::new().configure(|cfg| setup_app(cfg, tree_store.clone()));
                let app = test::init_service(cfg).await;

                (tree_store, app)
            }
        }};
    }

    #[actix_rt::test]
    async fn get_node_returns_200() {
        let (_, app) = test_app!();
        let req = test::TestRequest::get().uri("/api/tree").to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), 200);
    }

    #[actix_rt::test]
    async fn initial_get_node_returns_empty_json_array() {
        let (_, app) = test_app!();

        let req = test::TestRequest::get().uri("/api/tree").to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let json = test::read_body(response).await;
        let expected = Bytes::from(r#"[]"#);
        assert_eq!(json, expected);
    }

    #[actix_rt::test]
    async fn post_node_returns_200() {
        let (_, app) = test_app!();

        let req = test::TestRequest::post()
            .uri("/api/tree")
            .set_json(&json!({"label": "test", "parent_id": null}))
            .to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), 200);
    }

    #[actix_rt::test]
    async fn post_failure_returns_400() {
        let (_, app) = test_app!();

        let req = test::TestRequest::post()
            .uri("/api/tree")
            .set_json(&json!({"label": "test", "parent_id": "not an int"}))
            .to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), 400);
    }

    #[actix_web::test]
    async fn happy_path_get_tree() {
        let (tree_store, app) = test_app!();

        let req = test::TestRequest::get().uri("/api/tree").to_request();
        let response = test::call_service(&app, req).await;

        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let json = test::read_body(response).await;
        assert_eq!(json, Bytes::from_static(b"[]"));

        tree_store
            .clone()
            .add_node(String::from("root"), None)
            .unwrap();

        let req = test::TestRequest::get().uri("/api/tree").to_request();
        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), 200);

        let json = test::read_body(response).await;
        let expected = Bytes::from(r#"[{"id":1,"label":"root","children":[]}]"#);
        assert_eq!(json, expected);
    }

    #[actix_rt::test]
    async fn happy_path_add_node() {
        let (tree_store, app) = test_app!();

        assert!(tree_store.len() == 0);
        let req = test::TestRequest::post()
            .uri("/api/tree")
            .set_json(&AddNodeRequest {
                label: "root".to_string(),
                parent_id: None,
            })
            .to_request();

        let response = test::call_service(&app, req).await;
        assert_eq!(response.status(), 200);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
        let json = test::read_body(response).await;
        assert_eq!(
            json,
            Bytes::from_static(b"[{\"id\":1,\"label\":\"root\",\"children\":[]}]")
        );

        assert!(tree_store.len() == 1);
    }
}
