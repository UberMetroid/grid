mod header;
mod i18n;
mod storage;
mod types;

use crate::header::Header;
use crate::i18n::get_translations;
use crate::storage::StorageService;
use crate::types::{Board, BoardData, Column, Language};
use gloo_net::http::Request;
use web_sys::{DragEvent, HtmlInputElement};
use yew::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub is_error: bool,
}

pub enum Msg {
    FetchConfigSuccess(serde_json::Value),
    FetchTasksSuccess(BoardData),
    FetchTasksError,
    VerifyPinSuccess,
    VerifyPinFailure(String),
    PinInputChanged(String),
    VerifyPin,
    Logout,
    SwitchLanguage(Language),
    ToggleTheme,
    PrintBoard,

    // Tasks management
    OpenAddTaskModal(String),
    OpenEditTaskModal(String, usize),
    TaskModalInputChanged(String),
    SaveTask,
    DeleteTask,
    DeleteTaskDirect(String, usize),
    CloseTaskModal,

    // Drag & Drop
    DragStart(String, usize, DragEvent),
    DragOver(DragEvent),
    Drop(String, Option<usize>, DragEvent),

    // Toast
    ShowToast(String, bool),
    DismissToast(usize),
    Nothing,
}

pub struct App {
    site_title: String,
    theme: String,
    language: Language,
    is_authenticated: bool,
    pin_required: bool,
    pin_length: usize,
    pin_input: String,
    error_message: Option<String>,
    board_data: Option<BoardData>,

    // UI states
    active_board_id: String,

    task_modal_column_id: Option<String>,
    task_modal_index: Option<usize>,
    task_modal_text: String,
    show_task_modal: bool,

    // Drag data
    dragged_column_id: Option<String>,
    dragged_task_index: Option<usize>,

    // Toast
    toasts: Vec<Toast>,
    next_toast_id: usize,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Load initial theme & language
        let raw_theme = StorageService::get_item("theme", "crateria");
        let theme = match raw_theme.as_str() {
            "light" => "brinstar".to_string(),
            "dark" => "crateria".to_string(),
            "nord" => "maridia".to_string(),
            "dracula" => "wrecked_ship".to_string(),
            "sepia" => "norfair".to_string(),
            t => t.to_string(),
        };
        if theme != raw_theme {
            StorageService::set_item("theme", &theme);
        }
        let language = Language::from_code(&StorageService::get_item("language", "en"));

        // Initialize document classes
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(el) = document.document_element()
        {
            let _ = el.set_attribute("class", &theme);
            let _ = el.set_attribute("data-theme", &theme);
        }

        // Fetch config from backend
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get("/api/pin-required").send().await {
                Ok(resp) if resp.status() == 200 => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        link.send_message(Msg::FetchConfigSuccess(json));
                    }
                }
                _ => {}
            }
        });

        let app = Self {
            site_title: "RustKan".to_string(),
            theme,
            language,
            is_authenticated: false,
            pin_required: false,
            pin_length: 0,
            pin_input: String::new(),
            error_message: None,
            board_data: None,
            active_board_id: "work".to_string(),

            task_modal_column_id: None,
            task_modal_index: None,
            task_modal_text: String::new(),
            show_task_modal: false,
            dragged_column_id: None,
            dragged_task_index: None,
            toasts: Vec::new(),
            next_toast_id: 0,
        };
        app.update_document_title();
        app
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchConfigSuccess(json) => {
                let pin_req = json
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let pin_len = json.get("length").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                self.pin_required = pin_req;
                self.pin_length = pin_len;

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
            Msg::FetchTasksSuccess(mut data) => {
                let mut needs_save = false;
                if data.boards.is_empty() {
                    let new_board = Board {
                        name: "Work".to_string(),
                        columns: {
                            let mut cols = indexmap::IndexMap::new();
                            cols.insert(
                                "todo".to_string(),
                                Column {
                                    name: "To Do".to_string(),
                                    tasks: vec![],
                                },
                            );
                            cols.insert(
                                "doing".to_string(),
                                Column {
                                    name: "Doing".to_string(),
                                    tasks: vec![],
                                },
                            );
                            cols.insert(
                                "done".to_string(),
                                Column {
                                    name: "Done".to_string(),
                                    tasks: vec![],
                                },
                            );
                            cols
                        },
                    };
                    data.boards.insert("work".to_string(), new_board);
                    data.active_board = "work".to_string();
                    needs_save = true;
                }

                // Normalise all columns to exactly: To Do, Doing, Done
                for board in data.boards.values_mut() {
                    let mut new_cols = indexmap::IndexMap::new();
                    let mut todo_tasks = vec![];
                    let mut doing_tasks = vec![];
                    let mut done_tasks = vec![];

                    let old_cols = std::mem::take(&mut board.columns);
                    for (id, col) in old_cols {
                        match id.as_str() {
                            "todo" => todo_tasks.extend(col.tasks),
                            "doing" => doing_tasks.extend(col.tasks),
                            "done" => done_tasks.extend(col.tasks),
                            _ => {
                                // Put any other columns' tasks in "To Do"
                                todo_tasks.extend(col.tasks);
                                needs_save = true;
                            }
                        }
                    }

                    new_cols.insert(
                        "todo".to_string(),
                        Column {
                            name: "To Do".to_string(),
                            tasks: todo_tasks,
                        },
                    );
                    new_cols.insert(
                        "doing".to_string(),
                        Column {
                            name: "Doing".to_string(),
                            tasks: doing_tasks,
                        },
                    );
                    new_cols.insert(
                        "done".to_string(),
                        Column {
                            name: "Done".to_string(),
                            tasks: done_tasks,
                        },
                    );

                    board.columns = new_cols;
                }

                self.board_data = Some(data.clone());
                self.active_board_id = data.active_board.clone();
                if needs_save {
                    self.save_tasks_backend(ctx);
                }
                self.update_document_title();
                true
            }
            Msg::FetchTasksError => {
                self.show_toast("Failed to load tasks".to_string(), true, ctx);
                true
            }
            Msg::VerifyPinSuccess => {
                self.is_authenticated = true;
                self.error_message = None;
                self.load_tasks(ctx);
                true
            }
            Msg::VerifyPinFailure(err) => {
                self.is_authenticated = false;
                if !err.is_empty() {
                    let tr = get_translations(self.language);
                    if err == "Invalid PIN" {
                        self.error_message = Some(tr.invalid_pin.to_string());
                    } else {
                        self.error_message = Some(err);
                    }
                }
                true
            }
            Msg::PinInputChanged(val) => {
                self.pin_input = val;
                self.error_message = None;
                true
            }
            Msg::VerifyPin => {
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
                            link.send_message(Msg::VerifyPinFailure(
                                "Connection error".to_string(),
                            ));
                        }
                    }
                });
                self.pin_input.clear();
                true
            }
            Msg::Logout => {
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
            Msg::PrintBoard => {
                if let Some(window) = web_sys::window() {
                    let _ = window.print();
                }
                false
            }
            Msg::SwitchLanguage(lang) => {
                self.language = lang;
                StorageService::set_item("language", lang.code());
                true
            }
            Msg::ToggleTheme => {
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

            Msg::OpenAddTaskModal(col_id) => {
                self.task_modal_column_id = Some(col_id);
                self.task_modal_index = None;
                self.task_modal_text.clear();
                self.show_task_modal = true;
                true
            }
            Msg::OpenEditTaskModal(col_id, idx) => {
                if let Some(ref data) = self.board_data
                    && let Some(board) = data.boards.get(&self.active_board_id)
                    && let Some(col) = board.columns.get(&col_id)
                    && let Some(task) = col.tasks.get(idx)
                {
                    self.task_modal_column_id = Some(col_id);
                    self.task_modal_index = Some(idx);
                    self.task_modal_text = task.clone();
                    self.show_task_modal = true;
                }
                true
            }
            Msg::TaskModalInputChanged(val) => {
                self.task_modal_text = val;
                true
            }
            Msg::SaveTask => {
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
            Msg::DeleteTask => {
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
            Msg::DeleteTaskDirect(col_id, idx) => {
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
            Msg::CloseTaskModal => {
                self.show_task_modal = false;
                true
            }
            Msg::DragStart(col_id, idx, e) => {
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
            Msg::DragOver(e) => {
                e.prevent_default();
                false
            }
            Msg::Drop(dest_col_id, dest_idx, e) => {
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
                    // Extract task from source column
                    let task_opt = board
                        .columns
                        .get_mut(&src_col_id)
                        .map(|col| col.tasks.remove(src_idx));
                    if let Some(task) = task_opt {
                        // Insert task into destination column
                        if let Some(dest_col) = board.columns.get_mut(&dest_col_id) {
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
                }

                self.dragged_column_id = None;
                self.dragged_task_index = None;
                true
            }
            Msg::ShowToast(message, is_error) => {
                let id = self.next_toast_id;
                self.next_toast_id += 1;
                self.toasts.push(Toast {
                    id,
                    message,
                    is_error,
                });
                let link = ctx.link().clone();
                gloo_timers::callback::Timeout::new(3000, move || {
                    link.send_message(Msg::DismissToast(id));
                })
                .forget();
                true
            }
            Msg::DismissToast(id) => {
                self.toasts.retain(|t| t.id != id);
                true
            }
            Msg::Nothing => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);

        let disable_print = if let Some(ref data) = self.board_data {
            if let Some(board) = data.boards.get(&self.active_board_id) {
                board.columns.values().all(|col| col.tasks.is_empty())
            } else {
                true
            }
        } else {
            true
        };

        let board_html = if !self.is_authenticated {
            self.render_pin_entry(ctx)
        } else if let Some(ref data) = self.board_data {
            if let Some(board) = data.boards.get(&self.active_board_id) {
                self.render_board(board, ctx)
            } else if !data.boards.is_empty() {
                let fallback_board = data.boards.values().next().unwrap();
                self.render_board(fallback_board, ctx)
            } else {
                html! { <div class="loading-spinner">{"Initializing board..."}</div> }
            }
        } else {
            html! { <div class="loading-spinner">{"Loading Kanban Board..."}</div> }
        };

        html! {
            <>
                /* Header Navigation */
                <Header
                    site_title={self.site_title.clone()}
                    theme={self.theme.clone()}
                    language={self.language}
                    toggle_theme={ctx.link().callback(|_| Msg::ToggleTheme)}
                    on_language_change={ctx.link().callback(Msg::SwitchLanguage)}
                    is_authenticated={self.is_authenticated}
                    pin_required={self.pin_required}
                    on_logout={ctx.link().callback(|_| Msg::Logout)}
                    logout_tooltip={tr.logout_tooltip.to_string()}
                    theme_toggle_tooltip={tr.theme_toggle_tooltip.to_string()}
                    on_print={ctx.link().callback(|_| Msg::PrintBoard)}
                    print_tooltip={tr.print_tooltip.to_string()}
                    disable_print={disable_print}
                />

                /* Main Body Wrapper */
                <div class="app-body">
                    {board_html}
                </div>

                /* Footer */
                <footer class="layout-footer">
                    {
                        if let Some(t) = self.toasts.last() {
                            let cls = if t.is_error { "error" } else { "success" };
                            html! {
                                <div class={classes!("footer-status-text", cls)}>
                                    {&t.message}
                                </div>
                            }
                        } else {
                            html! {
                                <div class="footer-status-text success">
                                    {"Ready"}
                                </div>
                            }
                        }
                    }
                </footer>

                /* Task Modal Dialog */
                if self.show_task_modal {
                    {self.render_task_modal(ctx)}
                }
            </>
        }
    }
}

impl App {
    fn update_document_title(&self) {
        if let Some(window) = web_sys::window()
            && let Some(document) = window.document()
        {
            if let Some(ref data) = self.board_data
                && let Some(board) = data.boards.get(&self.active_board_id)
            {
                document.set_title(&format!("{} - {}", board.name, self.site_title));
                return;
            }
            document.set_title(&self.site_title);
        }
    }

    fn load_tasks(&self, ctx: &Context<Self>) {
        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get("/data/tasks.json").send().await {
                Ok(resp) if resp.status() == 200 => {
                    if let Ok(data) = resp.json::<BoardData>().await {
                        link.send_message(Msg::FetchTasksSuccess(data));
                    } else {
                        link.send_message(Msg::FetchTasksError);
                    }
                }
                _ => {
                    link.send_message(Msg::FetchTasksError);
                }
            }
        });
    }

    fn save_tasks_backend(&self, _ctx: &Context<Self>) {
        if let Some(ref data) = self.board_data {
            let payload = data.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = Request::post("/data/tasks.json")
                    .json(&payload)
                    .unwrap()
                    .send()
                    .await;
            });
        }
    }

    fn show_toast(&mut self, message: String, is_error: bool, ctx: &Context<Self>) {
        let id = self.next_toast_id;
        self.next_toast_id += 1;
        self.toasts.push(Toast {
            id,
            message,
            is_error,
        });
        let link = ctx.link().clone();
        gloo_timers::callback::Timeout::new(3000, move || {
            link.send_message(Msg::DismissToast(id));
        })
        .forget();
    }

    fn render_pin_entry(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        let on_submit = {
            let link = ctx.link().clone();
            Callback::from(move |e: SubmitEvent| {
                e.prevent_default();
                link.send_message(Msg::VerifyPin);
            })
        };
        let on_input = {
            let link = ctx.link().clone();
            let pin_length = self.pin_length;
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                let val = input.value();
                let filtered: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                input.set_value(&filtered);

                link.send_message(Msg::PinInputChanged(filtered.clone()));
                if filtered.len() == pin_length {
                    link.send_message(Msg::VerifyPin);
                }
            })
        };

        html! {
            <div class="login-container">
                <div class="login-box">
                    <div class="pin-header">
                        <h2>{tr.enter_pin}</h2>
                    </div>
                    <form id="pin-form" onsubmit={on_submit}>
                        <div class="pin-wrapper">
                            <input
                                id="pin-input"
                                type="password"
                                class="pin-input-field"
                                value={self.pin_input.clone()}
                                oninput={on_input}
                                autocomplete="current-password"
                                autofocus=true
                                maxlength={self.pin_length.to_string()}
                                placeholder={"•".repeat(self.pin_length)}
                            />
                        </div>
                    </form>
                    <div class="pin-status">
                        if let Some(ref err) = self.error_message {
                            <p id="pin-error" class="pin-error" style="display: block;">{err}</p>
                        }
                    </div>
                </div>
            </div>
        }
    }

    fn render_board(&self, board: &Board, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        html! {
            <main>
                <div class="board">
                    {for board.columns.iter().map(|(col_id, col)| {
                        let col_id = col_id.clone();
                        let col_id_drop = col_id.clone();
                        let col_id_add = col_id.clone();
                        let col_id_tasks = col_id.clone();
                        html! {
                            <div class="column">
                                <div class="column-header-box">
                                    <h2 class="column-name">
                                        {&col.name}
                                    </h2>
                                </div>
                                <div
                                    class="column-tasks-box"
                                    ondragover={ctx.link().callback(Msg::DragOver)}
                                    ondrop={ctx.link().callback(move |e| Msg::Drop(col_id_drop.clone(), None, e))}
                                >
                                    <div class="tasks">
                                        {for col.tasks.iter().enumerate().map({
                                            let col_id_tasks = col_id_tasks.clone();
                                            move |(idx, task)| {
                                                let col_id_drag = col_id_tasks.clone();
                                                let col_id_drop = col_id_tasks.clone();
                                                let col_id_edit = col_id_tasks.clone();
                                                let col_id_delete = col_id_tasks.clone();

                                                let on_delete_click = ctx.link().callback(move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    Msg::DeleteTaskDirect(col_id_delete.clone(), idx)
                                                });

                                                html! {
                                                    <div
                                                        class="task"
                                                        draggable="true"
                                                        ondragstart={ctx.link().callback(move |e| Msg::DragStart(col_id_drag.clone(), idx, e))}
                                                        ondragover={ctx.link().callback(Msg::DragOver)}
                                                        ondrop={ctx.link().callback(move |e| Msg::Drop(col_id_drop.clone(), Some(idx), e))}
                                                        ondblclick={ctx.link().callback(move |_| Msg::OpenEditTaskModal(col_id_edit.clone(), idx))}
                                                    >
                                                        <div class="move-indicator">{"⋮⋮"}</div>
                                                        <div class="task-content">
                                                            <span class="task-text">{task}</span>
                                                        </div>
                                                        <button
                                                            class="task-delete-btn"
                                                            onclick={on_delete_click}
                                                            title={tr.delete.to_string()}
                                                        >
                                                            {"×"}
                                                        </button>
                                                    </div>
                                                }
                                            }
                                        })}
                                    </div>
                                </div>
                                if col_id == "todo" {
                                    <button
                                        class="add-task"
                                        onclick={ctx.link().callback(move |_| Msg::OpenAddTaskModal(col_id_add.clone()))}
                                    >
                                        {tr.add_task}
                                    </button>
                                }
                            </div>
                        }
                    })}
                </div>
            </main>
        }
    }

    fn render_task_modal(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        let on_input = {
            let link = ctx.link().clone();
            Callback::from(move |e: InputEvent| {
                let input: HtmlInputElement = e.target_unchecked_into();
                link.send_message(Msg::TaskModalInputChanged(input.value()));
            })
        };

        html! {
            <div class="modal task-modal-overlay">
                <div class="modal-content card">
                    <h2>{tr.edit_task}</h2>
                    <div class="modal-body">
                        <label>{tr.task_text}</label>
                        <input
                            type="text"
                            placeholder={tr.task_placeholder}
                            value={self.task_modal_text.clone()}
                            oninput={on_input}
                            autofocus=true
                        />
                    </div>
                    <div class="modal-actions">
                        <button class="btn btn-primary" onclick={ctx.link().callback(|_| Msg::SaveTask)}>
                            {tr.save}
                        </button>
                        if self.task_modal_index.is_some() {
                            <button class="btn btn-danger" onclick={ctx.link().callback(|_| Msg::DeleteTask)}>
                                {tr.delete}
                            </button>
                        }
                        <button class="btn btn-secondary" onclick={ctx.link().callback(|_| Msg::CloseTaskModal)}>
                            {tr.cancel}
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
