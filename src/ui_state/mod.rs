mod add_state;
mod delete_state;
mod list_state;

pub use add_state::AddState;
pub use delete_state::DeleteState;
pub use list_state::ListState;

#[derive(Debug)]
pub enum UiState {
    List(ListState),
    Add(AddState),
    Delete(DeleteState),
    Quit,
}
