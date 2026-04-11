#[derive(Clone, PartialEq, Eq)]
pub enum IpSource {
    Single(String),
    Multi(Vec<String>),
}
