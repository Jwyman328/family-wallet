use crate::child::Child;

/// A Struct containing all the `children` of the app. 
/// 
/// The Struct's main functionality is the abliity search through it 
/// to find a `Child` by their id.
pub struct Children {
    pub children: Vec<Child>,
}

impl Children { 
    pub fn new() -> Children{
         Children {
            children:vec![]
        }
    }

    /// Adds a new `Child` to the exist children.
    pub fn add_child(&mut self, account_id: i32,account_name: String ) {
        let new_child = Child {
            user_id: account_id,
            account_name: account_name,
        };
       self.children.push(new_child);
    }

    /// This function searches through all the children and returns the 
    /// `Child` matching an id.
    pub fn get_child_by_id(&self, account_id: i32) -> Option<&Child>{
        for child in &self.children {
            if child.user_id == account_id {
                return Some(&child)
            }
        }

        None
    }

    /// This function searches through all the children and returns a 
    /// mutable `Child` matching an id.
    pub fn get_mutable_child_by_id(&mut self, account_id: i32) -> Option<&mut Child>{
        for child in &mut self.children {
            if child.user_id == account_id {
                return Some(child)
            }
        }

        None
    }
}