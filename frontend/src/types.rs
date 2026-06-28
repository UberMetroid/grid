use serde::{Deserialize, Serialize};

pub use shared_core::i18n::Language;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Column {
    pub name: String,
    pub tasks: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Board {
    pub name: String,
    pub columns: indexmap::IndexMap<String, Column>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BoardData {
    /// Monotonic version. Server returns this; the client sends it back on
    /// save. On mismatch (someone else wrote first) the server returns 409
    /// and the client must refetch and retry. `#[serde(default)]` so files
    /// written before versioning was added still load (they start at 0).
    #[serde(default)]
    pub version: u64,
    pub boards: indexmap::IndexMap<String, Board>,
    #[serde(rename = "activeBoard", default)]
    pub active_board: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: usize,
    pub message: String,
    pub is_error: bool,
}

pub enum Msg {
    FetchConfigSuccess(serde_json::Value),
    FetchTasksSuccess(BoardData),
    FetchTasksError,
    VerifyPinSuccess,
    VerifyPinFailure(String),
    PinInputChanged(String),
    VerifyPin,
    Logout,
    SwitchLanguage(Language),
    ToggleTheme,
    PrintBoard,

    // Tasks management
    OpenAddTaskModal(String),
    OpenEditTaskModal(String, usize),
    TaskModalInputChanged(String),
    SaveTask,
    DeleteTask,
    DeleteTaskDirect(String, usize),
    CloseTaskModal,

    // Drag & Drop
    DragStart(String, usize, web_sys::DragEvent),
    DragOver(web_sys::DragEvent),
    Drop(String, Option<usize>, web_sys::DragEvent),

    // Toast
    DismissToast(usize),
}
