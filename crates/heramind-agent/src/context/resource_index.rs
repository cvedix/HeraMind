//! Dynamic resource index system.
//!
//! This module provides a dynamic, searchable index of system resources
//! (devices, device types, alert channels, etc.) that evolves as the system changes.
//!
//! ## Key Features
//!
//! - **Dynamic Registration**: Resources register themselves as they come online
//! - **Fuzzy Search**: Find resources by partial names, aliases, or keywords
//! - **Vector Embedding**: Semantic search for resource discovery
//! - **No Hardcoding**: Tools are generated dynamically from available resources

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};

/// Unique identifier for any resource in the system.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId {
    /// Resource type (device, channel, workflow, etc.)
    pub resource_type: String,
    /// Unique identifier within the type
    pub id: String,
}

impl ResourceId {
    /// Create a new resource ID.
    pub fn new(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Create a device resource ID.
    pub fn device(id: impl Into<String>) -> Self {
        Self::new("device", id)
    }

    /// Create a channel resource ID.
    pub fn channel(id: impl Into<String>) -> Self {
        Self::new("channel", id)
    }

    /// Create a device type resource ID.
    pub fn device_type(id: impl Into<String>) -> Self {
        Self::new("device_type", id)
    }

    /// String representation "type:id".
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.resource_type, self.id)
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.resource_type, self.id)
    }
}

/// A searchable resource in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource ID
    pub id: ResourceId,
    /// Display name
    pub name: String,
    /// Aliases for fuzzy matching
    pub aliases: Vec<String>,
    /// Keywords for semantic matching
    pub keywords: Vec<String>,
    /// Resource type specific data
    pub data: ResourceData,
    /// When this resource was registered
    pub registered_at: i64,
    /// Last updated timestamp
    pub updated_at: i64,
    /// Whether the resource is active
    pub active: bool,
}

/// Resource type specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceData {
    /// Device resource
    Device(DeviceResourceData),
    /// Device type resource
    DeviceType(DeviceTypeResourceData),
    /// Alert channel resource
    AlertChannel(AlertChannelResourceData),
    /// Generic resource
    Generic(HashMap<String, serde_json::Value>),
}

/// Device resource data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceResourceData {
    /// Device type ID
    pub device_type: String,
    /// Location
    pub location: Option<String>,
    /// Capabilities (read metrics, write commands)
    pub capabilities: Vec<Capability>,
    /// Current state values
    pub state: HashMap<String, ResourceValue>,
    /// Connection status
    pub online: bool,
}

/// Device type resource data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceTypeResourceData {
    /// Device category (sensor, actuator, controller, etc.)
    pub category: String,
    /// Manufacturer
    pub manufacturer: Option<String>,
    /// Model
    pub model: Option<String>,
    /// Default capabilities for this type
    pub default_capabilities: Vec<Capability>,
}

/// Alert channel resource data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannelResourceData {
    /// Channel type (email, webhook, mqtt, etc.)
    pub channel_type: String,
    /// Supported severity levels
    pub supported_severities: Vec<String>,
    /// Whether the channel is enabled
    pub enabled: bool,
}

/// Device capability (metric or command).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name
    pub name: String,
    /// Capability type
    pub cap_type: CapabilityType,
    /// Data type
    pub data_type: String,
    /// Valid values (for enum type)
    pub valid_values: Option<Vec<String>>,
    /// Unit (for metrics)
    pub unit: Option<String>,
    /// Read/Write access
    pub access: AccessType,
}

/// Capability type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Readable metric
    Metric,
    /// Writable command
    Command,
    /// Readable and writable property
    Property,
}

/// Access type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessType {
    Read,
    Write,
    ReadWrite,
}

/// A value from a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceValue {
    /// Value
    pub value: serde_json::Value,
    /// Timestamp
    pub timestamp: i64,
    /// Quality flags
    pub quality: ValueQuality,
}

/// Value quality flags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueQuality {
    /// Whether the value is valid
    pub valid: bool,
    /// Whether the value is stale
    pub stale: bool,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Search result with relevance score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Matching resource
    pub resource: Resource,
    /// Relevance score (0-1)
    pub score: f32,
    /// Matched fields
    pub matched_fields: Vec<String>,
    /// Match highlights
    pub highlights: Vec<String>,
}

/// Search query for resources.
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// Search terms
    pub terms: Vec<String>,
    /// Resource type filter
    pub resource_type: Option<String>,
    /// Location filter
    pub location: Option<String>,
    /// Capability filter
    pub capability: Option<String>,
    /// Minimum relevance score (0-1)
    pub min_score: f32,
    /// Maximum results
    pub limit: usize,
}

/// Dynamic resource index.
pub struct ResourceIndex {
    /// All indexed resources
    resources: Arc<RwLock<HashMap<ResourceId, Resource>>>,

    /// Name index: lowercase name -> resource IDs
    name_index: Arc<RwLock<HashMap<String, Vec<ResourceId>>>>,

    /// Alias index: lowercase alias -> resource IDs
    alias_index: Arc<RwLock<HashMap<String, Vec<ResourceId>>>>,

    /// Keyword index: keyword -> resource IDs
    keyword_index: Arc<RwLock<HashMap<String, Vec<ResourceId>>>>,

    /// Location index: location -> resource IDs
    location_index: Arc<RwLock<HashMap<String, Vec<ResourceId>>>>,

    /// Capability index: capability -> resource IDs
    capability_index: Arc<RwLock<HashMap<String, Vec<ResourceId>>>>,

    /// Type index: resource type -> resource IDs
    type_index: Arc<RwLock<HashMap<String, Vec<ResourceId>>>>,
}

impl Default for ResourceIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceIndex {
    /// Create a new resource index.
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            name_index: Arc::new(RwLock::new(HashMap::new())),
            alias_index: Arc::new(RwLock::new(HashMap::new())),
            keyword_index: Arc::new(RwLock::new(HashMap::new())),
            location_index: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
            type_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a resource in the index.
    pub async fn register(&self, resource: Resource) -> Result<(), String> {
        let id = resource.id.clone();
        let resource_type = id.resource_type.clone();

        // Extract data for indexing
        let name_lower = resource.name.to_lowercase();
        let location = resource.data.location();
        let capabilities = resource.data.capabilities();

        // Remove old version if exists
        self.deregister(&id).await.ok();

        // Store resource
        {
            let mut resources = self.resources.write().await;
            resources.insert(id.clone(), resource.clone());
        }

        // Index by name
        {
            let mut name_idx = self.name_index.write().await;
            name_idx
                .entry(name_lower)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Index by aliases
        {
            let mut alias_idx = self.alias_index.write().await;
            for alias in &resource.aliases {
                alias_idx
                    .entry(alias.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }

        // Index by keywords
        {
            let mut kw_idx = self.keyword_index.write().await;
            for kw in &resource.keywords {
                kw_idx
                    .entry(kw.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }

        // Index by location
        if let Some(loc) = &location {
            let mut loc_idx = self.location_index.write().await;
            loc_idx
                .entry(loc.to_lowercase())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        // Index by capabilities
        {
            let mut cap_idx = self.capability_index.write().await;
            for cap in &capabilities {
                cap_idx
                    .entry(cap.to_lowercase())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }

        // Index by type
        {
            let mut type_idx = self.type_index.write().await;
            type_idx
                .entry(resource_type)
                .or_insert_with(Vec::new)
                .push(id.clone());
        }

        Ok(())
    }

    /// Deregister a resource.
    pub async fn deregister(&self, id: &ResourceId) -> Result<(), String> {
        // Get the resource first for cleanup
        let resource = {
            let resources = self.resources.read().await;
            resources.get(id).cloned()
        };

        let Some(resource) = resource else {
            return Err("Resource not found".to_string());
        };

        // Remove from main store
        {
            let mut resources = self.resources.write().await;
            resources.remove(id);
        }

        // Clean up indexes
        let name_lower = resource.name.to_lowercase();
        {
            let mut name_idx = self.name_index.write().await;
            if let Some(ids) = name_idx.get_mut(&name_lower) {
                ids.retain(|x| x != id);
                if ids.is_empty() {
                    name_idx.remove(&name_lower);
                }
            }
        }

        for alias in &resource.aliases {
            let alias_lower = alias.to_lowercase();
            let mut alias_idx = self.alias_index.write().await;
            if let Some(ids) = alias_idx.get_mut(&alias_lower) {
                ids.retain(|x| x != id);
                if ids.is_empty() {
                    alias_idx.remove(&alias_lower);
                }
            }
        }

        Ok(())
    }

    /// Search resources by query.
    pub async fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let mut candidates: Vec<ResourceId> = Vec::new();

        // Gather candidates from various indexes
        for term in &query.terms {
            let term_lower = term.to_lowercase();

            // Name index
            {
                let name_idx = self.name_index.read().await;
                for (name, ids) in name_idx.iter() {
                    if name.contains(&term_lower) || term_lower.contains(name) {
                        candidates.extend(ids.clone());
                    }
                }
            }

            // Alias index
            {
                let alias_idx = self.alias_index.read().await;
                for (alias, ids) in alias_idx.iter() {
                    if alias.contains(&term_lower) || term_lower.contains(alias) {
                        candidates.extend(ids.clone());
                    }
                }
            }

            // Keyword index
            {
                let kw_idx = self.keyword_index.read().await;
                if let Some(ids) = kw_idx.get(&term_lower) {
                    candidates.extend(ids.clone());
                }
            }

            // Capability index
            if let Some(cap) = &query.capability {
                let cap_idx = self.capability_index.read().await;
                if let Some(ids) = cap_idx.get(&cap.to_lowercase()) {
                    candidates.extend(ids.clone());
                }
            }
        }

        // Location filter
        if let Some(loc) = &query.location {
            let loc_lower = loc.to_lowercase();
            let loc_idx = self.location_index.read().await;
            for (location, ids) in loc_idx.iter() {
                if location.contains(&loc_lower) || loc_lower.contains(location) {
                    candidates.extend(ids.clone());
                }
            }
        }

        // Type filter
        if let Some(rt) = &query.resource_type {
            let type_idx = self.type_index.read().await;
            if let Some(ids) = type_idx.get(rt) {
                candidates.extend(ids.clone());
            }
        }

        // Deduplicate and score
        let resources = self.resources.read().await;
        let mut results: Vec<SearchResult> = candidates
            .into_iter()
            .filter_map(|id| resources.get(&id).cloned())
            .map(|resource| {
                let (score, matched_fields, highlights) = self.score_resource(&resource, query);
                SearchResult {
                    resource,
                    score,
                    matched_fields,
                    highlights,
                }
            })
            .filter(|r| r.score >= query.min_score)
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit results
        results.truncate(query.limit);
        results
    }

    /// Search by natural language query string.
    pub async fn search_string(&self, query_str: &str) -> Vec<SearchResult> {
        let terms = Self::extract_terms(query_str);

        // Detect query intent
        let resource_type = if terms
            .iter()
            .any(|t| t.contains("设备") || t.contains("device"))
        {
            None // Don't filter, let results speak
        } else if terms
            .iter()
            .any(|t| t.contains("通道") || t.contains("channel") || t.contains("告警"))
        {
            Some("channel".to_string())
        } else {
            None
        };

        let capability = terms
            .iter()
            .find(|t| {
                t.contains("温度")
                    || t.contains("湿度")
                    || t.contains("亮度")
                    || t.contains("temperature")
                    || t.contains("humidity")
                    || t.contains("brightness")
            })
            .cloned();

        let location = terms
            .iter()
            .find(|t| {
                t.contains("客厅")
                    || t.contains("卧室")
                    || t.contains("厨房")
                    || t.contains("living")
                    || t.contains("bedroom")
                    || t.contains("kitchen")
            })
            .cloned();

        let query = SearchQuery {
            terms: terms.clone(),
            resource_type,
            location,
            capability,
            min_score: 0.1,
            limit: 20,
        };

        self.search(&query).await
    }

    /// Extract search terms from a natural language query.
    fn extract_terms(query: &str) -> Vec<String> {
        let mut terms = Vec::new();

        // Split by common delimiters
        for part in query.split([' ', '、', ',', '，']) {
            let part = part.trim();
            if !part.is_empty() {
                terms.push(part.to_string());
            }
        }

        // Also extract individual characters for Chinese
        if query.chars().any(|c| c.is_alphabetic() && !c.is_ascii()) {
            for ch in query.chars() {
                if ch.is_alphabetic() {
                    terms.push(ch.to_string());
                }
            }
        }

        terms
    }

    /// Score a resource against a search query.
    fn score_resource(
        &self,
        resource: &Resource,
        query: &SearchQuery,
    ) -> (f32, Vec<String>, Vec<String>) {
        let mut score = 0.0f32;
        let mut matched_fields = Vec::new();
        let mut highlights = Vec::new();

        let resource_lower = resource.name.to_lowercase();

        for term in &query.terms {
            let term_lower = term.to_lowercase();

            // Exact name match - highest score
            if resource_lower == term_lower {
                score += 1.0;
                matched_fields.push("name".to_string());
                highlights.push(resource.name.clone());
            }
            // Name contains term
            else if resource_lower.contains(&term_lower) {
                score += 0.7;
                matched_fields.push("name".to_string());
                highlights.push(resource.name.clone());
            }
            // Term contains name (partial match)
            else if term_lower.contains(&resource_lower) && !resource_lower.is_empty() {
                score += 0.5;
                matched_fields.push("name".to_string());
            }

            // Alias match
            for alias in &resource.aliases {
                if alias.to_lowercase() == term_lower {
                    score += 0.8;
                    matched_fields.push("alias".to_string());
                    highlights.push(alias.clone());
                } else if alias.to_lowercase().contains(&term_lower) {
                    score += 0.5;
                    matched_fields.push("alias".to_string());
                }
            }

            // Keyword match
            for kw in &resource.keywords {
                if kw.to_lowercase() == term_lower {
                    score += 0.4;
                    matched_fields.push("keyword".to_string());
                }
            }

            // Capability match
            if let Some(_cap) = &query.capability {
                for capability in resource.data.capabilities() {
                    if capability.to_lowercase().contains(&term_lower)
                        || term_lower.contains(&capability.to_lowercase())
                    {
                        score += 0.6;
                        matched_fields.push("capability".to_string());
                    }
                }
            }
        }

        // Location match bonus
        if let Some(query_loc) = &query.location {
            if let Some(resource_loc) = resource.data.location() {
                if resource_loc
                    .to_lowercase()
                    .contains(&query_loc.to_lowercase())
                {
                    score += 0.3;
                    matched_fields.push("location".to_string());
                }
            }
        }

        // Normalize score to 0-1 using sigmoid-like function
        // This prevents raw score accumulation from growing too large
        // while maintaining relative differences between matches
        score = (score / 3.0).min(1.0);

        (score, matched_fields, highlights)
    }

    /// List all resources of a given type.
    pub async fn list_by_type(&self, resource_type: &str) -> Vec<Resource> {
        let resources = self.resources.read().await;
        resources
            .values()
            .filter(|r| r.id.resource_type == resource_type)
            .cloned()
            .collect()
    }

    /// Get all devices.
    pub async fn list_devices(&self) -> Vec<Resource> {
        self.list_by_type("device").await
    }
}

/// Helper trait to extract common data from ResourceData.
pub trait ResourceDataHelper {
    /// Get location if applicable.
    fn location(&self) -> Option<String>;
    /// Get capabilities.
    fn capabilities(&self) -> Vec<String>;
}

impl ResourceDataHelper for ResourceData {
    fn location(&self) -> Option<String> {
        match self {
            ResourceData::Device(d) => d.location.clone(),
            ResourceData::DeviceType(_) => None,
            ResourceData::AlertChannel(_) => None,
            ResourceData::Generic(_) => None,
        }
    }

    fn capabilities(&self) -> Vec<String> {
        match self {
            ResourceData::Device(d) => d.capabilities.iter().map(|c| c.name.clone()).collect(),
            ResourceData::DeviceType(d) => d
                .default_capabilities
                .iter()
                .map(|c| c.name.clone())
                .collect(),
            ResourceData::AlertChannel(_) => vec![],
            ResourceData::Generic(_) => vec![],
        }
    }
}

impl Resource {
    /// Create a new device resource.
    pub fn device(
        id: impl Into<String>,
        name: impl Into<String>,
        device_type: impl Into<String>,
    ) -> Self {
        Self {
            id: ResourceId::device(id),
            name: name.into(),
            aliases: Vec::new(),
            keywords: Vec::new(),
            data: ResourceData::Device(DeviceResourceData {
                device_type: device_type.into(),
                location: None,
                capabilities: Vec::new(),
                state: HashMap::new(),
                online: true,
            }),
            registered_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            active: true,
        }
    }

    /// Add an alias.
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add a keyword.
    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    /// Set location.
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        if let ResourceData::Device(ref mut d) = self.data {
            d.location = Some(location.into());
        }
        self
    }

    /// Add a capability.
    pub fn with_capability(mut self, cap: Capability) -> Self {
        if let ResourceData::Device(ref mut d) = self.data {
            d.capabilities.push(cap);
        }
        self
    }

    /// Set online status.
    pub fn with_online(mut self, online: bool) -> Self {
        if let ResourceData::Device(ref mut d) = self.data {
            d.online = online;
        }
        self
    }

    /// Get the device data if this is a device resource.
    pub fn as_device(&self) -> Option<&DeviceResourceData> {
        match &self.data {
            ResourceData::Device(d) => Some(d),
            _ => None,
        }
    }

    /// Check if resource matches a search term.
    pub fn matches(&self, term: &str) -> bool {
        let term_lower = term.to_lowercase();
        self.name.to_lowercase().contains(&term_lower)
            || self
                .aliases
                .iter()
                .any(|a| a.to_lowercase().contains(&term_lower))
            || self
                .keywords
                .iter()
                .any(|k| k.to_lowercase().contains(&term_lower))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_registration() {
        let index = ResourceIndex::new();

        let device = Resource::device("temp_1", "客厅温度传感器", "dht22")
            .with_alias("温度传感器")
            .with_location("客厅")
            .with_capability(Capability {
                name: "temperature".to_string(),
                cap_type: CapabilityType::Metric,
                data_type: "float".to_string(),
                valid_values: None,
                unit: Some("°C".to_string()),
                access: AccessType::Read,
            });

        index.register(device).await.unwrap();

        let results = index.search_string("温度").await;
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_fuzzy_search() {
        let index = ResourceIndex::new();

        let device = Resource::device("light_living", "客厅灯", "switch")
            .with_alias("灯")
            .with_location("客厅");

        index.register(device).await.unwrap();

        // Search by alias
        let results = index.search_string("灯").await;
        assert!(!results.is_empty());

        // Search by location
        let results = index.search_string("客厅").await;
        assert!(!results.is_empty());

        // Search with action prefix - "打开客厅灯"
        let results = index.search_string("打开客厅灯").await;
        assert!(!results.is_empty());
        assert_eq!(results[0].resource.name, "客厅灯");
    }
}
