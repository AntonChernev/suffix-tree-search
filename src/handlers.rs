use hyper::{Body, Response, StatusCode};
use gotham::state::{FromState, State};
use gotham::helpers::http::response::create_response;
use crate::routing::{TextState, QueryStringExtractor};

pub fn search(mut state: State) -> (State, Response<Body>) {
    let part = QueryStringExtractor::take_from(&mut state).part;
    let search_result = {
        let text_state = TextState::borrow_from(&state);
        text_state.suffix_tree.lock().unwrap().search(&part[..], 30)
    };

    let res = create_response(
        &state,
        StatusCode::OK,
        mime::APPLICATION_JSON,
        serde_json::to_vec(&search_result).unwrap()
    );
    (state, res)
}

pub fn get_text(state: State) -> (State, Response<Body>) {
    let text_state = TextState::borrow_from(&state);
    let text = text_state.text.clone();

    let res = create_response(
        &state,
        StatusCode::OK,
        mime::TEXT_PLAIN,
        text.into_bytes()
    );
    (state, res)
}