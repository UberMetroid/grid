mod app;
mod header;
mod footer;
mod i18n;
mod storage;
mod types;
mod utils;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
