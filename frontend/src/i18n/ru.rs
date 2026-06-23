use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ Добавить задачу",
            edit_task: "Редактировать задачу",
            task_text: "Текст задачи",
            task_placeholder: "Описание задачи...",
            save: "Сохранить",
            cancel: "Отмена",
            delete: "Удалить",
            pin_required: "Требуется PIN код",
            enter_pin: "Введите PIN",
            invalid_pin: "Неверный PIN код",
            logout_tooltip: "Выйти",
            theme_toggle_tooltip: "Переключить тему",
            toast_task_moved: "Задача перемещена",
            toast_task_added: "Задача добавлена",
            toast_task_updated: "Задача обновлена",
            toast_task_deleted: "Задача удалена",
            print_tooltip: "Печать доски",
            confirm_delete: "Вы уверены, что хотите удалить эту задачу?",
    }
}
