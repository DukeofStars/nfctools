// This file was automatically generated. Do not modify.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Fleet {
    #[serde(rename = "@xmlns:xsd")]
    pub xmlns_xsd: String,
    #[serde(rename = "@xmlns:xsi")]
    pub xmlns_xsi: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ModDependencies")]
    pub mod_dependencies: Option<FleetModDependencies>,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "CraftTypes")]
    pub craft_types: Option<CraftTypes>,
    #[serde(rename = "MissileTypes")]
    pub missile_types: Option<MissileTypes>,
    #[serde(rename = "Name")]
    pub name: String,
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FleetModDependencies {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "unsignedLong")]
    pub unsigned_long: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CraftTypes {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CraftTemplate")]
    pub craft_template: Vec<CraftTemplate>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CraftTemplate {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "AssociatedTemplateName")]
    pub associated_template_name: Option<String>,
    #[serde(rename = "Nickname")]
    pub nickname: String,
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
    pub mod_dependencies: CraftTemplateModDependencies,
    #[serde(rename = "LongDescription")]
    pub long_description: Option<String>,
    #[serde(rename = "DesignationSuffix")]
    pub designation_suffix: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InstalledComponents {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SerializedCraftSocket")]
    pub serialized_craft_socket: Vec<SerializedCraftSocket>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SerializedCraftSocket {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SocketKey")]
    pub socket_key: String,
    #[serde(rename = "ComponentKey")]
    pub component_key: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Loadouts {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CraftLoadout")]
    pub craft_loadout: Vec<CraftLoadout>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CraftLoadout {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "LoadoutName")]
    pub loadout_name: String,
    #[serde(rename = "Elements")]
    pub elements: Elements,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Elements {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "GeneralLoadoutElement")]
    pub general_loadout_element: Vec<GeneralLoadoutElement>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GeneralLoadoutElement {
    #[serde(rename = "@type")]
    pub xsi_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileKeys")]
    pub missile_keys: Option<GeneralLoadoutElementMissileKeys>,
    #[serde(rename = "AmmoKey")]
    pub ammo_key: Option<String>,
    #[serde(rename = "Loadout")]
    pub loadout: Option<Loadout>,
    #[serde(rename = "ComponentKey")]
    pub component_key: Option<String>,
    #[serde(rename = "SocketKey")]
    pub socket_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GeneralLoadoutElementMissileKeys {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub string: Vec<GeneralLoadoutElementMissileKeysString>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GeneralLoadoutElementMissileKeysString {
    #[serde(rename = "@nil")]
    pub xsi_nil: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Loadout {
    #[serde(rename = "@type")]
    pub xsi_type: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "AmmoKey")]
    pub ammo_key: Option<String>,
    #[serde(rename = "MissileKeys")]
    pub missile_keys: Option<LoadoutMissileKeys>,
    #[serde(rename = "SocketKey")]
    pub socket_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoadoutMissileKeys {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub string: Vec<LoadoutMissileKeysString>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoadoutMissileKeysString {
    #[serde(rename = "@nil")]
    pub xsi_nil: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CraftTemplateTemplateMissileTypes {
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CraftTemplateModDependencies {
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissileTypes {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileTemplate")]
    pub missile_template: Vec<MissileTemplate>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct BaseColor {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StripeColor {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sockets {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MissileSocket")]
    pub missile_socket: Vec<MissileSocket>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissileSocket {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "InstalledComponent")]
    pub installed_component: Option<InstalledComponent>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InstalledComponent {
    #[serde(rename = "@type")]
    pub xsi_type: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ValidatorMem")]
    pub validator_mem: Option<String>,
    #[serde(rename = "Interval")]
    pub interval: Option<String>,
    #[serde(rename = "SubmunitionKey")]
    pub submunition_key: Option<String>,
    #[serde(rename = "Range")]
    pub range: Option<String>,
    #[serde(rename = "SpreadOption")]
    pub spread_option: Option<String>,
    #[serde(rename = "TargetType")]
    pub target_type: Option<String>,
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct SortOverrideOrder {
    #[serde(rename = "@nil")]
    pub xsi_nil: Option<String>,
    #[serde(rename = "@nil")]
    pub nil: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ships {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Ship")]
    pub ship: Vec<Ship>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ship {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "HullConfig")]
    pub hull_config: Option<HullConfig>,
    #[serde(rename = "TemplateSpacecraftTypes")]
    pub template_spacecraft_types: Option<TemplateSpacecraftTypes>,
    #[serde(rename = "Callsign")]
    pub callsign: Option<String>,
    #[serde(rename = "SaveID")]
    pub save_id: SaveId,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Cost")]
    pub cost: String,
    #[serde(rename = "Number")]
    pub number: String,
    #[serde(rename = "SymbolOption")]
    pub symbol_option: String,
    #[serde(rename = "HullType")]
    pub hull_type: String,
    #[serde(rename = "SocketMap")]
    pub socket_map: SocketMap,
    #[serde(rename = "WeaponGroups")]
    pub weapon_groups: WeaponGroups,
    #[serde(rename = "TemplateMissileTypes")]
    pub template_missile_types: ShipTemplateMissileTypes,
    #[serde(rename = "InitialFormation")]
    pub initial_formation: Option<InitialFormation>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HullConfig {
    #[serde(rename = "@type")]
    pub xsi_type: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "TextureVariation")]
    pub texture_variation: Option<TextureVariation>,
    #[serde(rename = "HullTint")]
    pub hull_tint: Option<HullTint>,
    #[serde(rename = "SecondaryStructure")]
    pub secondary_structure: Option<SecondaryStructure>,
    #[serde(rename = "PrimaryStructure")]
    pub primary_structure: Option<PrimaryStructure>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextureVariation {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub x: String,
    pub y: String,
    pub z: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HullTint {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub r: String,
    pub g: String,
    pub b: String,
    pub a: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SecondaryStructure {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SecondaryStructureConfig")]
    pub secondary_structure_config: SecondaryStructureConfig,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct PrimaryStructure {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SegmentConfiguration")]
    pub segment_configuration: Vec<SegmentConfiguration>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SegmentConfiguration {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Key")]
    pub key: String,
    #[serde(rename = "Dressing")]
    pub dressing: Dressing,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Dressing {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub int: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TemplateSpacecraftTypes {
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveId {
    #[serde(rename = "@nil")]
    pub xsi_nil: Option<String>,
    #[serde(rename = "@nil")]
    pub nil: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SocketMap {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "HullSocket")]
    pub hull_socket: Vec<HullSocket>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct ComponentData {
    #[serde(rename = "@type")]
    pub xsi_type: Option<String>,
    #[serde(rename = "@type")]
    pub component_data_type: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "IdentityOption")]
    pub identity_option: Option<String>,
    #[serde(rename = "ConfiguredSize")]
    pub configured_size: Option<ConfiguredSize>,
    #[serde(rename = "StoredCraft")]
    pub stored_craft: Option<StoredCraft>,
    #[serde(rename = "FireOutsideLimits")]
    pub fire_outside_limits: Option<String>,
    #[serde(rename = "MissileLoad")]
    pub missile_load: Option<MissileLoad>,
    #[serde(rename = "Load")]
    pub load: Option<Load>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfiguredSize {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub y: Option<String>,
    pub x: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StoredCraft {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "SavedStoredCraft")]
    pub saved_stored_craft: Vec<SavedStoredCraft>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SavedStoredCraft {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CraftTemplateKey")]
    pub craft_template_key: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissileLoad {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagSaveData")]
    pub mag_save_data: Vec<MissileLoadMagSaveData>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct Load {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MagSaveData")]
    pub mag_save_data: Vec<LoadMagSaveData>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct WeaponGroups {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "WepGroup")]
    pub wep_group: Vec<WepGroup>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WepGroup {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "MemberKeys")]
    pub member_keys: MemberKeys,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MemberKeys {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub string: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShipTemplateMissileTypes {
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InitialFormation {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "RelativePosition")]
    pub relative_position: Option<RelativePosition>,
    #[serde(rename = "GuideKey")]
    pub guide_key: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RelativePosition {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    pub x: String,
    pub y: String,
    pub z: String,
}

