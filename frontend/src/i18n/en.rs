use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ Add Task",
            edit_task: "Edit Task",
            task_text: "Task Content",
            task_placeholder: "Task description...",
            save: "Save",
            cancel: "Cancel",
            delete: "Delete",
            pin_required: "Access PIN Required",
            enter_pin: "Enter PIN",
            invalid_pin: "Invalid PIN",
            logout_tooltip: "Log out",
            theme_toggle_tooltip: "Toggle theme",
            toast_task_moved: "Task moved",
            toast_task_added: "Task added",
            toast_task_updated: "Task updated",
            toast_task_deleted: "Task deleted",
            print_tooltip: "Print board",
            confirm_delete: "Are you sure you want to delete this task?",
    }
}
