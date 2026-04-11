use crate::batch::ip::source::range::IpSourceRange;

pub mod range;

#[derive(Clone, PartialEq, Eq)]
pub enum IpSource {
    Single([u8; 4]),
    Range(Vec<IpSourceRange>),
}