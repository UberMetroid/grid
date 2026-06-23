use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ タスクを追加",
            edit_task: "タスクを編集",
            task_text: "タスク内容",
            task_placeholder: "タスクの説明...",
            save: "保存",
            cancel: "キャンセル",
            delete: "削除",
            pin_required: "PIN認証が必要です",
            enter_pin: "PINを入力",
            invalid_pin: "無効なPINです",
            logout_tooltip: "ログアウト",
            theme_toggle_tooltip: "テーマ切り替え",
            toast_task_moved: "タスクが移動されました",
            toast_task_added: "タスクが追加されました",
            toast_task_updated: "タスクが更新されました",
            toast_task_deleted: "タスクが削除されました",
            print_tooltip: "ボードを印刷",
            confirm_delete: "このタスクを削除してもよろしいですか？",
    }
}
