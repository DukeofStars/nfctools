use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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
    pub ships: Option<Ships>,
    #[serde(rename = "MissileTypes")]
    pub missile_types: Option<MissileTypes>,
    #[serde(rename = "CraftTypes")]
    pub craft_types: Option<CraftTypes>,
    #[serde(rename = "ModDependencies")]
    pub mod_dependencies: Option<ModDependencies>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ModDependencies {
    #[serde(rename = "unsignedLong")]
    pub unsigned_long: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CraftTypes {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CraftTemplate")]
    pub craft_template: Option<Vec<CraftTemplate>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CraftTemplate {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "DesignationSuffix")]
    pub designation_suffix: Option<String>,
    #[serde(rename = "Nickname")]
    pub nickname: String,
    #[serde(rename = "LongDescription")]
    pub long_description: String,
    #[serde(rename = "Cost")]
    pub cost: String,
    #[serde(rename = "FrameKey")]
    pub frame_key: String,
    #[serde(rename = "TemplateKey")]
    pub template_key: String,
    #[serde(rename = "InstalledComponents")]
    pub installed_components: InstalledComponents,
    #[serde(rename = "Loadouts")]
    pub loadouts: Loadouts,
    #[serde(rename = "TemplateMissileTypes")]
    pub template_missile_types: CraftTemplateTemplateMissileTypes,
    #[serde(rename = "ModDependencies")]
    pub mod_dependencies: ModDependencies,
    #[serde(rename = "AssociatedTemplateName")]
    pub associated_template_name: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CraftTemplateTemplateMissileTypes {}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct InstalledComponents {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SerializedCraftSocket")]
    pub serialized_craft_socket: Option<Vec<SerializedCraftSocket>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SerializedCraftSocket {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SocketKey")]
    pub socket_key: Option<String>,
    #[serde(rename = "ComponentKey")]
    pub component_key: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Loadouts {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CraftLoadout")]
    pub craft_loadout: Option<Vec<CraftLoadout>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CraftLoadout {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "LoadoutName")]
    pub loadout_name: String,
    #[serde(rename = "Elements")]
    pub elements: Elements,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Elements {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "GeneralLoadoutElement")]
    pub general_loadout_element: Option<Vec<GeneralLoadoutElement>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct GeneralLoadoutElement {
    #[serde(rename(serialize = "@xsi:type", deserialize = "@type"))]
    pub xsi_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "AmmoKey")]
    pub ammo_key: Option<String>,
    #[serde(rename = "SocketKey")]
    pub socket_key: Option<String>,
    #[serde(rename = "Loadout")]
    pub loadout: Option<Loadout>,
    #[serde(rename = "ComponentKey")]
    pub component_key: Option<String>,
    #[serde(rename = "MissileKeys")]
    pub missile_keys: Option<MissileKeys>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Loadout {
    #[serde(rename(serialize = "@xsi:type", deserialize = "@type"))]
    pub xsi_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "AmmoKey")]
    pub ammo_key: Option<String>,
    #[serde(rename = "MissileKeys")]
    pub missile_keys: Option<MissileKeys>,
    #[serde(rename = "SocketKey")]
    pub socket_key: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MissileKeys {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub string: Vec<StringX>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[serde(rename = "string")]
pub struct StringX {
    #[serde(rename = "@nil")]
    pub xsi_nil: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SortOverrideOrder {
    #[serde(rename(serialize = "@xsi:nil", deserialize = "@nil"))]
    pub xsi_nil: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Ships {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Ship")]
    pub ship: Option<Vec<Ship>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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
    pub callsign: Option<String>,
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
    pub weapon_groups: Option<WeaponGroups>,
    #[serde(rename = "TemplateMissileTypes")]
    pub template_missile_types: Option<TemplateMissileTypes>,
    #[serde(rename = "TemplateSpacecraftTypes")]
    pub template_spacecraft_types: Option<TemplateSpacecraftTypes>,
    #[serde(rename = "InitialFormation")]
    pub initial_formation: Option<InitialFormation>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct InitialFormation {
    #[serde(rename = "GuideKey")]
    pub guide_key: String,
    #[serde(rename = "RelativePosition")]
    pub relative_position: RelativePosition,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RelativePosition {
    x: f64,
    y: f64,
    z: f64,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SaveId {
    #[serde(rename(serialize = "@xsi:nil", deserialize = "@nil"))]
    pub xsi_nil: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HullConfig {
    #[serde(rename(serialize = "@xsi:type", deserialize = "@type"))]
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PrimaryStructure {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SegmentConfiguration")]
    pub segment_configuration: Vec<SegmentConfiguration>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SegmentConfiguration {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Dressing")]
    pub dressing: Dressing,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Dressing {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub int: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SecondaryStructure {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SecondaryStructureConfig")]
    pub secondary_structure_config: SecondaryStructureConfig,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct HullTint {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TextureVariation {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub x: String,
    pub y: String,
    pub z: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SocketMap {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "HullSocket")]
    pub hull_socket: Vec<HullSocket>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ComponentData {
    #[serde(rename(serialize = "@xsi:type", deserialize = "@type"))]
    pub xsi_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileLoad")]
    pub missile_load: Option<MissileLoad>,
    #[serde(rename = "Load")]
    pub load: Option<Load>,
    #[serde(rename = "StoredCraft")]
    pub stored_craft: Option<StoredCraft>,
    #[serde(rename = "FireOutsideLimits")]
    pub fire_outside_limits: Option<bool>,
    #[serde(rename = "ConfiguredSize")]
    pub configured_size: Option<ConfiguredSize>,
    #[serde(rename = "IdentityOption")]
    pub identity_option: Option<u8>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfiguredSize {
    pub x: u8,
    pub y: u8,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoredCraft {
    #[serde(rename = "SavedStoredCraft")]
    pub saved_stored_craft: Option<Vec<SavedStoredCraft>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SavedStoredCraft {
    #[serde(rename = "CraftTemplateKey")]
    pub craft_template_key: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MissileLoad {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagSaveData")]
    pub mag_save_data: Option<Vec<MissileLoadMagSaveData>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Load {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagSaveData")]
    pub mag_save_data: Option<Vec<LoadMagSaveData>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct WeaponGroups {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "WepGroup")]
    pub wep_group: Option<Vec<WepGroup>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct WepGroup {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MemberKeys")]
    pub member_keys: MemberKeys,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MemberKeys {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub string: Option<Vec<String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TemplateMissileTypes {}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TemplateSpacecraftTypes {}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MissileTypes {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileTemplate")]
    pub missile_template: Option<Vec<MissileTemplate>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MissileTemplate {
    #[serde(rename = "@xmlns:xsd")]
    pub xmlns_xsd: Option<String>,
    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: Option<String>,
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

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct BaseColor {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StripeColor {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Sockets {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileSocket")]
    pub missile_socket: Vec<MissileSocket>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MissileSocket {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "InstalledComponent")]
    pub installed_component: Option<InstalledComponent>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct InstalledComponent {
    #[serde(rename(serialize = "@xsi:type", deserialize = "@type"))]
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
    pub component_key: Option<String>,
    #[serde(rename = "BalanceValues")]
    pub balance_values: Option<BalanceValues>,
    #[serde(rename = "TargetType")]
    pub target_type: Option<String>,
    #[serde(rename = "TargetSizeMask")]
    pub target_size_mask: Option<u8>,
    #[serde(rename = "SpreadOption")]
    pub spread_option: Option<u8>,
    #[serde(rename = "Range")]
    pub range: Option<f64>,
    #[serde(rename = "Interval")]
    pub interval: Option<u8>,
    #[serde(rename = "SubmunitionKey")]
    pub submunition_key: Option<String>,
    #[serde(rename = "ValidatorMem")]
    pub validator_mem: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum SeekerMode {
    Targeting,
    Uniform,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DefensiveDoctrine {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "TargetSizeMask")]
    pub target_size_mask: String,
    #[serde(rename = "TargetType")]
    pub target_type: Option<String>,
    #[serde(rename = "TargetSizeOrdering")]
    pub target_size_ordering: String,
    #[serde(rename = "SalvoSize")]
    pub salvo_size: String,
    #[serde(rename = "FarthestFirst")]
    pub farthest_first: String,
    #[serde(rename = "Mode")]
    pub mode: Option<String>,
    #[serde(rename = "ConvSalvo")]
    pub conv_salvo: Option<u32>,
    #[serde(rename = "HybridSalvo")]
    pub hybrid_salvo: Option<u32>,
    #[serde(rename = "CraftSalvo")]
    pub craft_salvo: Option<u32>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
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
