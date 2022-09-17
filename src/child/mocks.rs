use crate::child::Child;


/// A function which creates a Child struct to use in tests.
pub fn mock_child() -> Child{
    let child_without_permissions = Child {
        user_id: 1,
        account_name: String::from("bob"),
    };
    child_without_permissions
}