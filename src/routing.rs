use std::env;
use std::fs::read_to_string;

use crate::handlers;
use crate::suffix_tree::SuffixTree;

use gotham::handler::assets::FileOptions;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;

use std::sync::{Arc, Mutex};

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

pub fn router() -> Router {
    let mut text_path = env::current_dir().unwrap();
    text_path.push("text/text.txt");
    let text = read_to_string(text_path).unwrap();
    let suffix_tree = SuffixTree::new(&text[..]);

    let text_state = TextState {
        text,
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
