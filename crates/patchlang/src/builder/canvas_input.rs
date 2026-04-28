//! Input types for the canvas → PatchLang emit direction.
//! TypeScript assembles this JSON bundle; Rust does all language work.

use serde::Deserialize;
use std::collections::HashMap;
use ts_rs::TS;

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CanvasEmitInput {
    pub instances: Vec<InstanceEmitInput>,
    pub connections: Vec<ConnectionEmitInput>,
    pub manufacturer_cards: Vec<CardEmitInput>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct InstanceEmitInput {
    /// Pre-resolved human-readable name (TypeScript does UUID → name mapping).
    pub name: String,
    pub device_type: String,
    pub manufacturer: Option<String>,
    pub model: String,
    pub category: Option<String>,
    pub kind: Option<String>,
    pub location: Option<String>,
    pub dante_chipset: Option<String>,
    pub rf_subtype: Option<String>,
    pub rf_min_channels: Option<u32>,
    pub rf_max_channels: Option<u32>,
    pub rf_band: Option<String>,
    pub interfaces: Vec<InterfaceEmitInput>,
    pub card_slot_groups: Vec<CardSlotGroupEmitInput>,
    pub installed_cards: Vec<InstalledCardEmitInput>,
    /// Map from interface id → list of channel labels.
    pub channel_labels: HashMap<String, Vec<ChannelLabelEmitInput>>,
    pub route_rules: Vec<RouteRuleEmitInput>,
    pub internal_buses: Vec<BusEmitInput>,
    pub tx_streams: Vec<StreamEmitInput>,
    pub rx_streams: Vec<StreamEmitInput>,
    pub is_ring_container: bool,
    pub ring_protocol: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct InterfaceEmitInput {
    pub id: String,
    pub label: String,
    /// "in" | "out" | "io" | "asymmetric"
    pub direction: String,
    pub connector: Option<String>,
    pub transport: Option<String>,
    pub channel_count: u32,
    pub attributes: Vec<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CardSlotGroupEmitInput {
    pub label: String,
    pub slot_count: u32,
    pub slot_format: String,
    pub direction: String,
    pub channel_count: u32,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct InstalledCardEmitInput {
    pub slot_label: String,
    pub slot_index: u32,
    pub card_template_name: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ChannelLabelEmitInput {
    pub channel_index: u32,
    pub label: String,
    pub phantom: bool,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct RouteRuleEmitInput {
    pub from_interface: String,
    pub from_channel: u32,
    pub to_interface: String,
    pub to_channel: u32,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct BusEmitInput {
    pub label: String,
    pub input_interface: String,
    pub input_channels: Vec<u32>,
    pub output_interface: String,
    pub output_channels: Vec<u32>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct StreamEmitInput {
    pub label: String,
    pub protocol: String,
    pub channel_count: u32,
    pub interface_id: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ConnectionEmitInput {
    pub from_instance_name: String,
    pub to_instance_name: String,
    pub from_port_id: String,
    pub to_port_id: String,
    pub is_backbone: bool,
    pub channel_mappings: Vec<ChannelMappingEmitInput>,
    pub properties: Vec<KvEmitInput>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ChannelMappingEmitInput {
    pub from_channel: u32,
    pub to_channel: u32,
    /// "forward" | "return" | "direct"
    pub mapping_type: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct KvEmitInput {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CardEmitInput {
    pub template_name: String,
    pub manufacturer: Option<String>,
    pub model: String,
    pub fits: String,
    pub interfaces: Vec<InterfaceEmitInput>,
}
