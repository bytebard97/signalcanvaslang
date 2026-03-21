use serde::Serialize;

use crate::error::Span;

/// Top-level program — a list of statements.
#[derive(Debug, Clone, Serialize)]
pub struct PatchProgram {
    pub statements: Vec<Statement>,
}

/// A top-level statement.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Statement {
    Template(TemplateDecl),
    Instance(InstanceDecl),
    Connect(ConnectDecl),
    Bridge(BridgeDecl),
    BridgeGroup(BridgeGroupDecl),
    LinkGroup(LinkGroupDecl),
    Signal(SignalDecl),
    Flag(FlagDecl),
    Stream(StreamDecl),
    Config(ConfigDecl),
    Use(UseDecl),
    /// Placeholder for recovered error regions.
    Error(Span),
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateDecl {
    pub name: String,
    pub params: Vec<String>,
    pub meta: Vec<KeyValue>,
    pub ports: Vec<PortDef>,
    pub bridges: Vec<BridgeDecl>,
    pub instances: Vec<InstanceDecl>,
    pub connects: Vec<ConnectDecl>,
    pub slots: Vec<SlotDef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceDecl {
    pub name: String,
    pub template_name: String,
    pub args: Vec<String>,
    pub properties: Vec<KeyValue>,
    pub routes: Vec<RouteEntry>,
    pub buses: Vec<BusEntry>,
    pub slot_assignments: Vec<SlotAssignment>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectDecl {
    pub source: PortRef,
    pub target: PortRef,
    pub properties: Vec<KeyValue>,
    pub suppressions: Vec<String>,
    pub mapping: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeDecl {
    pub source: PortRef,
    pub target: PortRef,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeGroupDecl {
    pub target: PortRef,
    pub sources: Vec<PortRef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct LinkGroupDecl {
    pub target: PortRef,
    pub sources: Vec<PortRef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct SignalDecl {
    pub name: String,
    pub properties: Vec<KeyValue>,
    pub origin: Option<PortRef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlagDecl {
    pub name: String,
    pub properties: Vec<KeyValue>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamDecl {
    pub name: String,
    pub properties: Vec<KeyValue>,
    pub source: Option<PortRef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigDecl {
    pub name: String,
    pub labels: Vec<KeyValue>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct UseDecl {
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortDef {
    pub name: String,
    pub range: Option<RangeSpec>,
    pub direction: PortDirection,
    pub connector: Option<String>,
    pub attributes: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum PortDirection {
    In,
    Out,
    Io,
}

#[derive(Debug, Clone, Serialize)]
pub struct RangeSpec {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortRef {
    pub instance: Option<String>,
    pub port: String,
    pub index: Option<IndexSpec>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexSpec {
    pub elements: Vec<IndexElement>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum IndexElement {
    Single { value: u32 },
    Range { start: u32, end: u32 },
}

#[derive(Debug, Clone, Serialize)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotDef {
    pub name: String,
    pub properties: Vec<KeyValue>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteEntry {
    pub source: String,
    pub target: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct BusEntry {
    pub name: String,
    pub properties: Vec<KeyValue>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotAssignment {
    pub slot_name: String,
    pub card_name: String,
    pub span: Span,
}
