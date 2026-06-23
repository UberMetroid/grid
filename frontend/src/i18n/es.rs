use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ Añadir tarea",
            edit_task: "Editar tarea",
            task_text: "Contenido de tarea",
            task_placeholder: "Descripción de la tarea...",
            save: "Guardar",
            cancel: "Cancelar",
            delete: "Eliminar",
            pin_required: "PIN de acceso requerido",
            enter_pin: "Ingrese PIN",
            invalid_pin: "PIN inválido",
            logout_tooltip: "Cerrar sesión",
            theme_toggle_tooltip: "Cambiar tema",
            toast_task_moved: "Tarea movida",
            toast_task_added: "Tarea añadida",
            toast_task_updated: "Tarea actualizada",
            toast_task_deleted: "Tarea eliminada",
            print_tooltip: "Imprimir tablero",
            confirm_delete: "¿Está seguro de que desea eliminar esta tarea?",
    }
}
