/// Operating mode type.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ResourceMode {
    Pruned,
    Archival,
}

impl ToString for ResourceMode {
    fn to_string(&self) -> String {
        match self {
            ResourceMode::Pruned => "pruned".to_string(),
            ResourceMode::Archival => "archival".to_string(),
        }
    }
}
