use yew::prelude::*;
use serde::Deserialize;
use gloo::events::EventListener;
use web_sys::window;

#[derive(Debug, Clone, Deserialize)]
struct UsageResponse {
    // simplified for this prototype – expand as needed
}

#[function_component(App)]
fn app() -> Html {
    let usage = use_state(|| Vec::<UsageResponse>::new());

    {
        let usage = usage.clone();
        let cb = Callback::from(move |payload: String| {
            if let Ok(data) = serde_json::from_str::<UsageResponse>(&payload) {
                usage.set(vec![data]);
            }
        });

        let window = window().unwrap();
        let listener = EventListener::new(&window, "usage_update", move |e| {
            let ev = e.dyn_ref::<web_sys::CustomEvent>().unwrap();
            let payload = ev.detail().as_string().unwrap();
            cb.emit(payload);
        });
        drop(listener);
    }

    html! {
        <div class="p-4">
            <h1 class="text-xl font-bold">{ "Hermes Usage Dashboard" }</h1>
            <div>{ /* chart placeholder */ }</div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
