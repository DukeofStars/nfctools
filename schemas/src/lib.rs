use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fleet {
    #[serde(rename = "@xmlns:xsd")]
    pub xmlns_xsd: String,
    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "TotalPoints")]
    pub total_points: String,
    #[serde(rename = "FactionKey")]
    pub faction_key: String,
    #[serde(rename = "SortOverrideOrder")]
    pub sort_override_order: SortOverrideOrder,
    #[serde(rename = "Ships")]
    pub ships: Ships,
    #[serde(rename = "MissileTypes")]
    pub missile_types: Option<MissileTypes>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SortOverrideOrder {
    #[serde(rename = "@nil")]
    pub xsi_nil: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ships {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Ship")]
    pub ship: Option<Vec<Ship>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ship {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SaveID")]
    pub save_id: SaveId,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Cost")]
    pub cost: String,
    #[serde(rename = "Callsign")]
    pub callsign: Option<Callsign>,
    #[serde(rename = "Number")]
    pub number: String,
    #[serde(rename = "SymbolOption")]
    pub symbol_option: String,
    #[serde(rename = "HullType")]
    pub hull_type: String,
    #[serde(rename = "HullConfig")]
    pub hull_config: Option<HullConfig>,
    #[serde(rename = "SocketMap")]
    pub socket_map: SocketMap,
    #[serde(rename = "WeaponGroups")]
    pub weapon_groups: WeaponGroups,
    #[serde(rename = "TemplateMissileTypes")]
    pub template_missile_types: TemplateMissileTypes,
    #[serde(rename = "TemplateSpacecraftTypes")]
    pub template_spacecraft_types: Option<TemplateSpacecraftTypes>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveId {
    #[serde(rename = "@nil")]
    pub xsi_nil: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Callsign {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HullConfig {
    #[serde(rename = "@type")]
    pub xsi_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "PrimaryStructure")]
    pub primary_structure: PrimaryStructure,
    #[serde(rename = "SecondaryStructure")]
    pub secondary_structure: SecondaryStructure,
    #[serde(rename = "HullTint")]
    pub hull_tint: HullTint,
    #[serde(rename = "TextureVariation")]
    pub texture_variation: TextureVariation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrimaryStructure {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SegmentConfiguration")]
    pub segment_configuration: Vec<SegmentConfiguration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SegmentConfiguration {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Dressing")]
    pub dressing: Dressing,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Dressing {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub int: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecondaryStructure {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SecondaryStructureConfig")]
    pub secondary_structure_config: SecondaryStructureConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecondaryStructureConfig {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Segment")]
    pub segment: String,
    #[serde(rename = "SnapPoint")]
    pub snap_point: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HullTint {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextureVariation {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub x: String,
    pub y: String,
    pub z: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketMap {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "HullSocket")]
    pub hull_socket: Vec<HullSocket>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HullSocket {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ComponentData")]
    pub component_data: Option<ComponentData>,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "ComponentName")]
    pub component_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentData {
    #[serde(rename = "@type")]
    pub xsi_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileLoad")]
    pub missile_load: Option<MissileLoad>,
    #[serde(rename = "Load")]
    pub load: Option<Load>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MissileLoad {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagSaveData")]
    pub mag_save_data: Option<Vec<MissileLoadMagSaveData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MissileLoadMagSaveData {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagazineKey")]
    pub magazine_key: String,
    #[serde(rename = "MunitionKey")]
    pub munition_key: String,
    #[serde(rename = "Quantity")]
    pub quantity: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Load {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagSaveData")]
    pub mag_save_data: Option<Vec<LoadMagSaveData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoadMagSaveData {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagazineKey")]
    pub magazine_key: String,
    #[serde(rename = "MunitionKey")]
    pub munition_key: String,
    #[serde(rename = "Quantity")]
    pub quantity: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeaponGroups {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "WepGroup")]
    pub wep_group: Option<Vec<WepGroup>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WepGroup {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MemberKeys")]
    pub member_keys: MemberKeys,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemberKeys {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub string: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateMissileTypes {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateSpacecraftTypes {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MissileTypes {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileTemplate")]
    pub missile_template: Option<Vec<MissileTemplate>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MissileTemplate {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "AssociatedTemplateName")]
    pub associated_template_name: Option<String>,
    #[serde(rename = "Designation")]
    pub designation: String,
    #[serde(rename = "Nickname")]
    pub nickname: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "LongDescription")]
    pub long_description: String,
    #[serde(rename = "Cost")]
    pub cost: String,
    #[serde(rename = "BodyKey")]
    pub body_key: String,
    #[serde(rename = "TemplateKey")]
    pub template_key: String,
    #[serde(rename = "BaseColor")]
    pub base_color: BaseColor,
    #[serde(rename = "StripeColor")]
    pub stripe_color: StripeColor,
    #[serde(rename = "Sockets")]
    pub sockets: Sockets,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BaseColor {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StripeColor {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sockets {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileSocket")]
    pub missile_socket: Vec<MissileSocket>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MissileSocket {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "InstalledComponent")]
    pub installed_component: Option<InstalledComponent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstalledComponent {
    #[serde(rename = "@type")]
    pub xsi_type: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "DetectPDTargets")]
    pub detect_pdtargets: Option<String>,
    #[serde(rename = "RejectUnvalidated")]
    pub reject_unvalidated: Option<String>,
    #[serde(rename = "Mode")]
    pub mode: Option<String>,
    #[serde(rename = "ApproachAngleControl")]
    pub approach_angle_control: Option<String>,
    #[serde(rename = "DefensiveDoctrine")]
    pub defensive_doctrine: Option<DefensiveDoctrine>,
    #[serde(rename = "Maneuvers")]
    pub maneuvers: Option<String>,
    #[serde(rename = "SelfDestructOnLost")]
    pub self_destruct_on_lost: Option<String>,
    #[serde(rename = "HotLaunch")]
    pub hot_launch: Option<String>,
    #[serde(rename = "Role")]
    pub role: Option<String>,
    #[serde(rename = "ComponentKey")]
    pub component_key: String,
    #[serde(rename = "BalanceValues")]
    pub balance_values: Option<BalanceValues>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefensiveDoctrine {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "TargetSizeMask")]
    pub target_size_mask: String,
    #[serde(rename = "TargetType")]
    pub target_type: String,
    #[serde(rename = "TargetSizeOrdering")]
    pub target_size_ordering: String,
    #[serde(rename = "SalvoSize")]
    pub salvo_size: String,
    #[serde(rename = "FarthestFirst")]
    pub farthest_first: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BalanceValues {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "A")]
    pub a: String,
    #[serde(rename = "B")]
    pub b: String,
    #[serde(rename = "C")]
    pub c: String,
}
