use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SpinRequest {
    client_seed: String,
    bet: f64,
    lines: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SpinResponse {
    server_seed_hash: String,
    nonce: u64,
    reels: Vec<Vec<u8>>,
    payout: f64,
}

#[function_component(App)]
fn app() -> Html {
    let client_seed = use_state(|| "demo-seed".to_string());
    let result = use_state(|| None as Option<SpinResponse>);

    let do_spin = {
        let client_seed = client_seed.clone();
        let result = result.clone();
        Callback::from(move |_| {
            let client_seed = (*client_seed).clone();
            wasm_bindgen_futures::spawn_local({
                let result = result.clone();
                async move {
                    let req = SpinRequest {
                        client_seed,
                        bet: 1.0,
                        lines: 1,
                    };
                    let resp = match reqwest::Client::new()
                        .post(&format!(
                            "{}/spin",
                            option_env!("BACKEND_URL").unwrap_or("http://127.0.0.1:8080")
                        ))
                        .json(&req)
                        .send()
                        .await
                    {
                        Ok(r) => r.json::<SpinResponse>().await.ok(),
                        Err(_) => None,
                    };
                    result.set(resp);
                }
            });
        })
    };

    html! {
        <div>
            <h1>{"Tigrinho (Demo)"}</h1>
            <input value={(*client_seed).clone()} oninput={{ let client_seed = client_seed.clone(); Callback::from(move |e: InputEvent| { let input: web_sys::HtmlInputElement = e.target_unchecked_into(); client_seed.set(input.value()); }) }} />
            <button onclick={do_spin}>{"Spin"}</button>
            <Canvas result={(*result).clone()} />
            if let Some(res) = &*result { <pre>{format!("hash={} nonce={} payout={}", res.server_seed_hash, res.nonce, res.payout)}</pre> }
            <p>{"Note: Front-end is for demo only. Verify RNG by recomputing HMAC on the server-revealed seed (not implemented here)."}</p>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct CanvasProps {
    result: Option<SpinResponse>,
}

#[function_component(Canvas)]
fn canvas(props: &CanvasProps) -> Html {
    let node_ref = use_node_ref();
    {
        let node_ref = node_ref.clone();
        let res = props.result.clone();
        use_effect_with(res, move |res| {
            if let Some(canvas) = node_ref.cast::<HtmlCanvasElement>() {
                let ctx: CanvasRenderingContext2d = canvas
                    .get_context("2d")
                    .unwrap()
                    .unwrap()
                    .dyn_into()
                    .unwrap();
                ctx.set_fill_style(&JsValue::from_str("#111"));
                ctx.fill_rect(0.0, 0.0, 300.0, 150.0);
                if let Some(r) = res {
                    for (row_idx, row) in r.reels.iter().enumerate() {
                        for (col_idx, sym) in row.iter().enumerate() {
                            let x = (col_idx as f64) * 90.0 + 10.0;
                            let y = (row_idx as f64) * 40.0 + 20.0;
                            ctx.set_fill_style(&JsValue::from_str(match sym {
                                0 => "#e74c3c",
                                1 => "#3498db",
                                2 => "#2ecc71",
                                3 => "#f1c40f",
                                _ => "#9b59b6",
                            }));
                            ctx.fill_rect(x, y, 80.0, 30.0);
                        }
                    }
                }
            }
        });
    }

    html! { <canvas ref={node_ref} width="300" height="150"></canvas> }
}

#[wasm_bindgen(start)]
pub fn run() {
    yew::Renderer::<App>::new().render();
}
