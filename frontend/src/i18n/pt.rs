use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ Adicionar tarefa",
            edit_task: "Editar tarefa",
            task_text: "Conteúdo da tarefa",
            task_placeholder: "Descrição da tarefa...",
            save: "Salvar",
            cancel: "Cancelar",
            delete: "Excluir",
            pin_required: "Código PIN necessário",
            enter_pin: "Digite o PIN",
            invalid_pin: "PIN inválido",
            logout_tooltip: "Sair",
            theme_toggle_tooltip: "Alternar tema",
            toast_task_moved: "Tarefa movida",
            toast_task_added: "Tarefa adicionada",
            toast_task_updated: "Tarefa atualizada",
            toast_task_deleted: "Tarefa excluída",
            print_tooltip: "Imprimir quadro",
            confirm_delete: "Tem certeza de que deseja excluir esta tarefa?",
    }
}
