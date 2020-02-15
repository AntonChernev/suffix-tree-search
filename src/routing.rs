use crate::handlers;
use crate::suffix_tree::SuffixTree;

use std::env;
use std::sync::{Arc, Mutex};
use gotham::handler::assets::FileOptions;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;

#[derive(Clone, StateData)]
pub struct TextState {
  pub text: String,
  pub suffix_tree: Arc<Mutex<SuffixTree>>
}

unsafe impl Send for TextState {}
unsafe impl Sync for TextState {}

#[derive(Deserialize, StateData, StaticResponseExtender)]
pub struct QueryStringExtractor {
    pub part: String,
}

pub fn router(text: &str) -> Router {
    let suffix_tree = SuffixTree::new(text);

    let text_state = TextState {
        text: String::from(text),
        suffix_tree: Arc::new(Mutex::new(suffix_tree))
    };
    let state_middleware = StateMiddleware::new(text_state);
    let state_pipeline = single_middleware(state_middleware);
    let (chain, pipelines) = single_pipeline(state_pipeline);

    build_router(chain, pipelines, |route| {
        route.get("/api/search")
            .with_query_string_extractor::<QueryStringExtractor>()
            .to(handlers::search);
        route.get("/api/text").to(handlers::get_text);

        let mut assets_path = env::current_dir().unwrap();
        assets_path.push("public");
        route.get("*").to_dir(
            FileOptions::new(&assets_path)
                .with_cache_control("no-cache")
                .with_gzip(true)
                .build(),
        );
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;
    use hyper::{StatusCode};

    #[test]
    fn test_get_text() {
        let text = "abcdeзfзgз";
        let test_server = TestServer::new(router(text)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/api/text")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.read_body().unwrap();
        let expected_body = text;
        assert_eq!(&body[..], expected_body.as_bytes());
    }

    #[test]
    fn test_search() {
        let text = "abcdeзfзgз";
        let test_server = TestServer::new(router(text)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/api/search?part=f")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.read_body().unwrap();
        let expected_body = serde_json::to_string(&vec![String::from("fзgз")]).unwrap();
        assert_eq!(&body[..], expected_body.as_bytes());
    }
}
