mod app;
mod app_update;
mod app_update_handlers;
mod app_view;
mod header;
mod footer;
mod i18n;
mod storage;
mod types;
mod utils;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
