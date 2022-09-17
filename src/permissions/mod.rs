/// The permissions that an `Account` can have.
#[derive(Debug)]
#[derive(PartialEq)]
pub enum BitcoinPermissions {
    Send,
    Receive,
}