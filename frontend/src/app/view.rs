use crate::app::App;
use crate::header::Header;
use crate::i18n::get_translations;
use crate::types::*;
use web_sys::HtmlInputElement;
use yew::prelude::*;

impl App {
    pub fn view_app(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        let show_version = self.show_version;
        let show_github = self.show_github;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let version_url = format!("https://github.com/UberMetroid/grid/releases/tag/v{}", version);

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
                    on_print={ctx.link().callback(|_| Msg::PrintBoard)}
                    print_tooltip={tr.print_tooltip.to_string()}
                    disable_print={disable_print}
                    theme_toggle_tooltip={tr.theme_toggle_tooltip.to_string()}
                    enable_translation={self.enable_translation}
                    enable_themes={self.enable_themes}
                    enable_print={self.enable_print}
                />

                /* Main Body Wrapper */
                <div class="app-body">
                    {board_html}
                </div>

                /* Footer */
                <crate::footer::Footer {show_version} {version} {show_github} {version_url}>
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
                </crate::footer::Footer>

                /* Task Modal Dialog */
                if self.show_task_modal {
                    {self.render_task_modal(ctx)}
                }
            </>
        }
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
