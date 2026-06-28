use crate::app::App;
use crate::storage::StorageService;
use crate::types::*;
use gloo_net::http::Request;
use shared_frontend::theme::Theme;
use yew::prelude::*;

impl App {
    pub fn handle_fetch_config_success(
        &mut self,
        ctx: &Context<Self>,
        json: serde_json::Value,
    ) -> bool {
        let pin_req = json
            .get("required")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let pin_len = json.get("length").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

        self.pin_required = pin_req;
        self.pin_length = pin_len;
        self.enable_translation = json
            .get("enable_translation")
            .or_else(|| json.get("enableTranslation"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        self.enable_themes = json
            .get("enable_themes")
            .or_else(|| json.get("enableThemes"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        self.enable_print = json
            .get("enable_print")
            .or_else(|| json.get("enablePrint"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        self.show_version = json
            .get("show_version")
            .or_else(|| json.get("showVersion"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        self.show_github = json
            .get("show_github")
            .or_else(|| json.get("showGithub"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if !self.enable_themes {
            self.theme = "tourian".to_string();
            if let Some(window) = web_sys::window()
                && let Some(doc) = window.document()
                && let Some(html) = doc.document_element()
            {
                let _ = html.set_attribute("data-theme", "tourian");
                let _ = html.set_attribute("class", "tourian");
            }
        }

        if pin_req {
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(resp) = Request::get("/api/auth-check").send().await {
                    if resp.status() == 200 {
                        link.send_message(Msg::VerifyPinSuccess);
                    } else {
                        link.send_message(Msg::VerifyPinFailure(String::new()));
                    }
                } else {
                    link.send_message(Msg::VerifyPinFailure(String::new()));
                }
            });
        } else {
            self.is_authenticated = true;
            self.load_tasks(ctx);
        }
        true
    }

    pub fn handle_fetch_tasks_success(&mut self, ctx: &Context<Self>, data: BoardData) -> bool {
        let (normalized, needs_save) = crate::utils::normalize_board_data(data);
        self.board_data = Some(normalized.clone());
        self.active_board_id = normalized.active_board.clone();
        if needs_save {
            self.save_tasks_backend(ctx);
        }
        self.update_document_title();
        true
    }

    pub fn handle_verify_pin(&mut self, ctx: &Context<Self>) -> bool {
        let pin = self.pin_input.clone();
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let body = serde_json::json!({ "pin": pin });
            match Request::post("/api/verify-pin")
                .json(&body)
                .unwrap()
                .send()
                .await
            {
                Ok(resp) if resp.status() == 200 => {
                    link.send_message(Msg::VerifyPinSuccess);
                }
                Ok(resp) => {
                    if let Ok(err_json) = resp.json::<serde_json::Value>().await {
                        let msg = err_json
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Invalid PIN")
                            .to_string();
                        link.send_message(Msg::VerifyPinFailure(msg));
                    } else {
                        link.send_message(Msg::VerifyPinFailure("Invalid PIN".to_string()));
                    }
                }
                Err(_) => {
                    link.send_message(Msg::VerifyPinFailure("Connection error".to_string()));
                }
            }
        });
        self.pin_input.clear();
        true
    }

    pub fn handle_logout(&mut self, ctx: &Context<Self>) -> bool {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            let _ = Request::post("/api/logout").send().await;
            link.send_message(Msg::FetchConfigSuccess(serde_json::json!({
                "required": true,
                "length": 4
            })));
        });
        self.is_authenticated = false;
        self.board_data = None;
        self.update_document_title();
        true
    }

    pub fn handle_toggle_theme(&mut self) -> bool {
        let current = Theme::from_name(&self.theme).unwrap_or_default();
        let next = match current {
            Theme::Brinstar => Theme::Norfair,
            Theme::Norfair => Theme::WreckedShip,
            Theme::WreckedShip => Theme::Maridia,
            Theme::Maridia => Theme::Tourian,
            Theme::Tourian => Theme::Crateria,
            Theme::Crateria => Theme::Brinstar,
        };
        self.theme = next.name().to_string();
        StorageService::set_item("theme", next.name());

        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(el) = document.document_element()
        {
            let _ = el.set_attribute("class", next.name());
            let _ = el.set_attribute("data-theme", next.name());
        }
        true
    }
}
