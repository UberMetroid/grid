use crate::i18n::Translations;

pub fn translations() -> Translations {
    Translations {
add_task: "+ 添加任务",
            edit_task: "编辑任务",
            task_text: "任务内容",
            task_placeholder: "任务描述...",
            save: "保存",
            cancel: "取消",
            delete: "删除",
            pin_required: "需要访问密码",
            enter_pin: "输入 PIN",
            invalid_pin: "PIN 码错误",
            logout_tooltip: "退出登录",
            theme_toggle_tooltip: "切换主题",
            toast_task_moved: "任务已移动",
            toast_task_added: "任务已添加",
            toast_task_updated: "任务已更新",
            toast_task_deleted: "任务已删除",
            print_tooltip: "打印看板",
            confirm_delete: "您确定要删除此任务吗？",
    }
}
