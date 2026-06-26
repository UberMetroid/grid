use crate::app::App;
use crate::i18n::get_translations;
use crate::storage::StorageService;
use crate::types::*;
use crate::utils::normalize_board_data;
use gloo_net::http::Request;
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
        let (normalized, needs_save) = normalize_board_data(data);
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
        let next_theme = match self.theme.as_str() {
            "crateria" => "brinstar",
            "brinstar" => "norfair",
            "norfair" => "wrecked_ship",
            "wrecked_ship" => "maridia",
            "maridia" => "tourian",
            _ => "crateria",
        };
        self.theme = next_theme.to_string();
        StorageService::set_item("theme", next_theme);

        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(el) = document.document_element()
        {
            let _ = el.set_attribute("class", next_theme);
            let _ = el.set_attribute("data-theme", next_theme);
        }
        true
    }

    pub fn handle_save_task(&mut self, ctx: &Context<Self>) -> bool {
        if self.task_modal_text.trim().is_empty() {
            return false;
        }
        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
            && let Some(ref col_id) = self.task_modal_column_id
            && let Some(col) = board.columns.get_mut(col_id)
        {
            let tr = get_translations(self.language);
            if let Some(idx) = self.task_modal_index {
                col.tasks[idx] = self.task_modal_text.trim().to_string();
                self.show_toast(tr.toast_task_updated.to_string(), false, ctx);
            } else {
                col.tasks.push(self.task_modal_text.trim().to_string());
                self.show_toast(tr.toast_task_added.to_string(), false, ctx);
            }
            self.save_tasks_backend(ctx);
        }
        self.show_task_modal = false;
        true
    }

    pub fn handle_delete_task(&mut self, ctx: &Context<Self>) -> bool {
        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
            && let Some(ref col_id) = self.task_modal_column_id
            && let Some(idx) = self.task_modal_index
            && let Some(col) = board.columns.get_mut(col_id)
        {
            col.tasks.remove(idx);
            self.save_tasks_backend(ctx);
            let tr = get_translations(self.language);
            self.show_toast(tr.toast_task_deleted.to_string(), false, ctx);
        }
        self.show_task_modal = false;
        true
    }

    pub fn handle_delete_task_direct(
        &mut self,
        ctx: &Context<Self>,
        col_id: String,
        idx: usize,
    ) -> bool {
        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
            && let Some(col) = board.columns.get_mut(&col_id)
            && idx < col.tasks.len()
        {
            let window = web_sys::window().unwrap();
            let tr = get_translations(self.language);
            let message = format!("{}\n\n\"{}\"", tr.confirm_delete, col.tasks[idx]);
            if window.confirm_with_message(&message).unwrap_or(false) {
                col.tasks.remove(idx);
                self.save_tasks_backend(ctx);
                self.show_toast(tr.toast_task_deleted.to_string(), false, ctx);
            }
        }
        true
    }

    pub fn handle_drag_start(&mut self, col_id: String, idx: usize, e: web_sys::DragEvent) -> bool {
        self.dragged_column_id = Some(col_id);
        self.dragged_task_index = Some(idx);
        if let Some(dt) = e.data_transfer() {
            let _ = dt.set_data(
                "text/plain",
                &format!("{}:{}", self.dragged_column_id.as_ref().unwrap(), idx),
            );
            dt.set_effect_allowed("move");
        }
        false
    }

    pub fn handle_drop(
        &mut self,
        ctx: &Context<Self>,
        dest_col_id: String,
        dest_idx: Option<usize>,
        e: web_sys::DragEvent,
    ) -> bool {
        e.prevent_default();
        let source_data = e
            .data_transfer()
            .map(|dt| dt.get_data("text/plain").unwrap_or_default())
            .unwrap_or_default();

        let (src_col_id, src_idx) = if !source_data.is_empty() {
            let parts: Vec<&str> = source_data.split(':').collect();
            if parts.len() == 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    (parts[0].to_string(), idx)
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            if let (Some(c), Some(i)) = (&self.dragged_column_id, self.dragged_task_index) {
                (c.clone(), i)
            } else {
                return false;
            }
        };

        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
        {
            let task_opt = board
                .columns
                .get_mut(&src_col_id)
                .map(|col| col.tasks.remove(src_idx));
            if let (Some(task), Some(dest_col)) = (task_opt, board.columns.get_mut(&dest_col_id)) {
                if let Some(idx) = dest_idx {
                    dest_col.tasks.insert(idx, task);
                } else {
                    dest_col.tasks.push(task);
                }
                self.save_tasks_backend(ctx);
                let tr = get_translations(self.language);
                self.show_toast(tr.toast_task_moved.to_string(), false, ctx);
            }
        }

        self.dragged_column_id = None;
        self.dragged_task_index = None;
        true
    }
}
