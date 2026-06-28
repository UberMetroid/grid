use crate::app::App;
use crate::i18n::get_translations;
use yew::prelude::*;

impl App {
    pub fn handle_save_task(&mut self, ctx: &Context<Self>) -> bool {
        if self.task_modal_text.trim().is_empty() {
            return false;
        }
        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
            && let Some(ref col_id) = self.task_modal_column_id
            && let Some(col) = board.columns.get_mut(col_id)
        {
            let tr = get_translations(self.language);
            if let Some(idx) = self.task_modal_index {
                col.tasks[idx] = self.task_modal_text.trim().to_string();
                self.show_toast(tr.toast_task_updated.to_string(), false, ctx);
            } else {
                col.tasks.push(self.task_modal_text.trim().to_string());
                self.show_toast(tr.toast_task_added.to_string(), false, ctx);
            }
            self.save_tasks_backend(ctx);
        }
        self.show_task_modal = false;
        true
    }

    pub fn handle_delete_task(&mut self, ctx: &Context<Self>) -> bool {
        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
            && let Some(ref col_id) = self.task_modal_column_id
            && let Some(idx) = self.task_modal_index
            && let Some(col) = board.columns.get_mut(col_id)
        {
            col.tasks.remove(idx);
            self.save_tasks_backend(ctx);
            let tr = get_translations(self.language);
            self.show_toast(tr.toast_task_deleted.to_string(), false, ctx);
        }
        self.show_task_modal = false;
        true
    }

    pub fn handle_delete_task_direct(
        &mut self,
        ctx: &Context<Self>,
        col_id: String,
        idx: usize,
    ) -> bool {
        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
            && let Some(col) = board.columns.get_mut(&col_id)
            && idx < col.tasks.len()
        {
            let window = web_sys::window().unwrap();
            let tr = get_translations(self.language);
            let message = format!("{}\n\n\"{}\"", tr.confirm_delete, col.tasks[idx]);
            if window.confirm_with_message(&message).unwrap_or(false) {
                col.tasks.remove(idx);
                self.save_tasks_backend(ctx);
                self.show_toast(tr.toast_task_deleted.to_string(), false, ctx);
            }
        }
        true
    }

    pub fn handle_drag_start(&mut self, col_id: String, idx: usize, e: web_sys::DragEvent) -> bool {
        self.dragged_column_id = Some(col_id);
        self.dragged_task_index = Some(idx);
        if let Some(dt) = e.data_transfer() {
            let _ = dt.set_data(
                "text/plain",
                &format!("{}:{}", self.dragged_column_id.as_ref().unwrap(), idx),
            );
            dt.set_effect_allowed("move");
        }
        false
    }

    pub fn handle_drop(
        &mut self,
        ctx: &Context<Self>,
        dest_col_id: String,
        dest_idx: Option<usize>,
        e: web_sys::DragEvent,
    ) -> bool {
        e.prevent_default();
        let source_data = e
            .data_transfer()
            .map(|dt| dt.get_data("text/plain").unwrap_or_default())
            .unwrap_or_default();

        let (src_col_id, src_idx) = if !source_data.is_empty() {
            let parts: Vec<&str> = source_data.split(':').collect();
            if parts.len() == 2 {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    (parts[0].to_string(), idx)
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            if let (Some(c), Some(i)) = (&self.dragged_column_id, self.dragged_task_index) {
                (c.clone(), i)
            } else {
                return false;
            }
        };

        if let Some(ref mut data) = self.board_data
            && let Some(board) = data.boards.get_mut(&self.active_board_id)
        {
            let task_opt = board
                .columns
                .get_mut(&src_col_id)
                .map(|col| col.tasks.remove(src_idx));
            if let (Some(task), Some(dest_col)) = (task_opt, board.columns.get_mut(&dest_col_id)) {
                if let Some(idx) = dest_idx {
                    dest_col.tasks.insert(idx, task);
                } else {
                    dest_col.tasks.push(task);
                }
                self.save_tasks_backend(ctx);
                let tr = get_translations(self.language);
                self.show_toast(tr.toast_task_moved.to_string(), false, ctx);
            }
        }

        self.dragged_column_id = None;
        self.dragged_task_index = None;
        true
    }
}
