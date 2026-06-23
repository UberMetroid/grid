use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ Aufgabe hinzufügen",
            edit_task: "Aufgabe bearbeiten",
            task_text: "Aufgabenbeschreibung",
            task_placeholder: "Aufgabendetails...",
            save: "Speichern",
            cancel: "Abbrechen",
            delete: "Löschen",
            pin_required: "PIN erforderlich",
            enter_pin: "PIN eingeben",
            invalid_pin: "Ungültige PIN",
            logout_tooltip: "Abmelden",
            theme_toggle_tooltip: "Farbschema ändern",
            toast_task_moved: "Aufgabe verschoben",
            toast_task_added: "Aufgabe hinzugefügt",
            toast_task_updated: "Aufgabe aktualisiert",
            toast_task_deleted: "Aufgabe gelöscht",
            print_tooltip: "Board drucken",
            confirm_delete: "Sind Sie sicher, dass Sie diese Aufgabe löschen möchten?",
    }
}
