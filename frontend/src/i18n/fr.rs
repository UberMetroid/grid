use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ Ajouter tâche",
            edit_task: "Modifier la tâche",
            task_text: "Contenu de tâche",
            task_placeholder: "Description de la tâche...",
            save: "Enregistrer",
            cancel: "Annuler",
            delete: "Supprimer",
            pin_required: "Code PIN requis",
            enter_pin: "Saisir le PIN",
            invalid_pin: "Code PIN invalide",
            logout_tooltip: "Se déconnecter",
            theme_toggle_tooltip: "Changer le thème",
            toast_task_moved: "Tâche déplacée",
            toast_task_added: "Tâche ajoutée",
            toast_task_updated: "Tâche mise à jour",
            toast_task_deleted: "Tâche supprimée",
            print_tooltip: "Imprimer le tableau",
            confirm_delete: "Voulez-vous vraiment supprimer cette tâche ?",
    }
}
