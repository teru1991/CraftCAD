#[derive(Debug, Clone)]
pub enum DeepLink {
    Entity { entity_id: String },
    Part { part_id: String },
    NestJob { job_id: String },
    ExportResult { export_id: String },
}

#[derive(Debug, Clone)]
pub enum DeepLinkEffect {
    NavigateTo(DeepLink),
    None,
}

pub fn parse_jump_to_entity(entity_id: &str) -> DeepLinkEffect {
    if entity_id.trim().is_empty() {
        return DeepLinkEffect::None;
    }
    DeepLinkEffect::NavigateTo(DeepLink::Entity {
        entity_id: entity_id.to_string(),
    })
}
