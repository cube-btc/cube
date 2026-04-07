/// Operating kind type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OperatingKind {
    Node,
    Engine,
}

impl ToString for OperatingKind {
    fn to_string(&self) -> String {
        match self {
            OperatingKind::Node => "node".to_string(),
            OperatingKind::Engine => "engine".to_string(),
        }
    }
}
