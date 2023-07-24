use strum::EnumIter;
use strum::EnumString;
use serde_json::Value;

#[derive(Copy, Clone, EnumString, EnumIter, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum ArtifactComponent {
    Abi,
}

pub fn extract_artifact_component_json(component: ArtifactComponent, data: &[u8]) -> anyhow::Result<Value> {
    match component {
        ArtifactComponent::Abi => {
            Ok(serde_json::from_str::<Value>(std::str::from_utf8(data)?)?["abi"].clone())
        }
    }
}