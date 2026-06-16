mod add_state;
mod delete_state;
mod edit_state;
mod list_state;

pub use add_state::AddState;
pub use delete_state::DeleteState;
pub use edit_state::EditState;
pub use list_state::ListState;

#[derive(Debug)]
pub enum UiState {
    List(ListState),
    Add(AddState),
    Edit(EditState),
    Delete(DeleteState),
    ConfirmOverwrite(usize),
    Quit,
}
