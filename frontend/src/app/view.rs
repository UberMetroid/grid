use crate::app::App;
use crate::components::footer::Footer;
use crate::components::header::Header;
use crate::i18n::get_translations;
use crate::types::*;
use yew::prelude::*;

impl App {
    pub fn view_app(&self, ctx: &Context<Self>) -> Html {
        let tr = get_translations(self.language);
        let show_version = self.show_version;
        let show_github = self.show_github;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let version_url = format!(
            "https://github.com/UberMetroid/grid/releases/tag/v{}",
            version
        );

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
                    on_print={if self.enable_print { Some(ctx.link().callback(|_| Msg::PrintBoard)) } else { None }}
                    print_tooltip={tr.print_tooltip.to_string()}
                    print_disabled={disable_print || !self.enable_print}
                    theme_toggle_tooltip={tr.theme_toggle_tooltip.to_string()}
                    enable_translation={self.enable_translation}
                    enable_themes={self.enable_themes}
                />

                /* Main Body Wrapper */
                <div class="app-body">
                    {board_html}
                </div>

                /* Footer */
                <Footer {show_version} {version} {show_github} {version_url}>
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
                </Footer>

                /* Task Modal Dialog */
                if self.show_task_modal {
                    {self.render_task_modal(ctx)}
                }
            </>
        }
    }
}
