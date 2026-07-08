//! Protocol Mapping Layer
//!
//! This module provides the abstraction layer for mapping device capabilities
//! to protocol-specific addresses and data formats. It decouples the device
//! type definitions (MDL) from protocol implementations (MQTT, etc.).
//!
//! ## Architecture
//!
//! ```text
//! Device Type Definition (MDL)       Protocol Mapping
//! ├─ temperature capability  ──────→  ├─ MQTT: sensor/${id}/temperature
//! ├─ humidity capability     ──────→  └─ MQTT: sensor/${id}/humidity
//! └─ relay_state capability  ──────→  └─ MQTT: relay/${id}/state
//! ```

pub mod mapping;
pub mod mqtt_mapping;

// Re-exports
pub use mapping::{
    Address, Capability, CapabilityType, MappingConfig, MappingError, MetricParser,
    PayloadSerializer, ProtocolMapping, SharedMapping,
};
pub use mqtt_mapping::{BinaryFormat, MqttMapping, MqttMappingBuilder, MqttValueParser};
