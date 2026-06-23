use crate::types::Language;

mod de;
mod en;
mod es;
mod fr;
mod ja;
mod pt;
mod ru;
mod zh;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Translations {

    pub add_task: &'static str,
    pub edit_task: &'static str,
    pub task_text: &'static str,
    pub task_placeholder: &'static str,
    pub save: &'static str,
    pub cancel: &'static str,
    pub delete: &'static str,
    pub pin_required: &'static str,
    pub enter_pin: &'static str,
    pub invalid_pin: &'static str,
    pub logout_tooltip: &'static str,
    pub theme_toggle_tooltip: &'static str,
    pub toast_task_moved: &'static str,
    pub toast_task_added: &'static str,
    pub toast_task_updated: &'static str,
    pub toast_task_deleted: &'static str,
    pub print_tooltip: &'static str,
    pub confirm_delete: &'static str,

}

pub fn get_translations(lang: Language) -> Translations {
    match lang {
        Language::Chinese => zh::translations(),
        Language::Spanish => es::translations(),
        Language::German => de::translations(),
        Language::Japanese => ja::translations(),
        Language::French => fr::translations(),
        Language::Portuguese => pt::translations(),
        Language::Russian => ru::translations(),
        _ => en::translations(),
    }
}
