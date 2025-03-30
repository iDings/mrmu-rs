
use clap::ValueEnum;
use strum::EnumString;
use strum::EnumIter;

use super::BitInfo;
use crate::bitinfo_comb_deflat;
use crate::bitinfo_comb_flat;

macro_rules! impl_into_bitinfo {
    ($bitinfo: ty) => {
        impl Into<BitInfo> for $bitinfo {
            fn into(self) -> BitInfo {
                let comb = self as u16;
                bitinfo_comb_deflat!(comb)
            }
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[repr(u8)]
pub enum PortRegister {
    PortStatus = 0x0,
    PhysicalControl,
    FlowControl,
    SwitchIdentifier,
    PortControl0,
    PortControl1,
    PortBasedVlanMap,
    DefaultVlanIdPriority,
    PortControl2,
    EgressRateControl,
    EgressRateControl2,
    PortAssociationVector,
    PortAtuControl,
    Override,
    PolicyMgmtControl,

    ExtendedPortControlCmd = 16,
    ExtendedPortControlData,

    PreemptionControl = 21,
    LedControl,
    IpPriorityMappingTable,
    IeeePriorityMappingTable,
    PortControl3,
    PortMiscScratch,
    QueueCounters,
    QueueControl,
    QueueControl2,
    EnableSelect,
    DebugCounters,
}

impl_into_bitinfo!(PortSTatus);
impl_into_bitinfo!(PhysicalControl);
impl_into_bitinfo!(FlowControl);
impl_into_bitinfo!(SwitchIdentifier);

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PortSTatus {
    TxPauseEn = bitinfo_comb_flat!(1, 15),
    RxPauseEn = bitinfo_comb_flat!(1, 14),
    AltSpdValue = bitinfo_comb_flat!(1, 13),
    PhyDetect = bitinfo_comb_flat!(1, 12),
    Link = bitinfo_comb_flat!(1, 11),
    Duplex = bitinfo_comb_flat!(1, 10),
    Speed = bitinfo_comb_flat!(2, 8),
    DuplexFixed = bitinfo_comb_flat!(1, 7),
    EeeEnabled = bitinfo_comb_flat!(1, 6),
    TxPaused = bitinfo_comb_flat!(1, 5),
    FlowCtrl = bitinfo_comb_flat!(1, 4),
    CMode = bitinfo_comb_flat!(4, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PhysicalControl {
    RgmiiRxTiming = bitinfo_comb_flat!(1, 15),
    RgmiiTxTiming = bitinfo_comb_flat!(1, 14),
    ForcedSpd = bitinfo_comb_flat!(1, 13),
    AltSpeed = bitinfo_comb_flat!(1, 12),
    MiiPhy = bitinfo_comb_flat!(1, 11),
    EeeValue = bitinfo_comb_flat!(1, 9),
    ForceEee = bitinfo_comb_flat!(1, 8),
    LinkValue = bitinfo_comb_flat!(1, 5),
    ForcedLink = bitinfo_comb_flat!(1, 4),
    DpxValue = bitinfo_comb_flat!(1, 3),
    ForcedDpx = bitinfo_comb_flat!(1, 2),
    SpdValue = bitinfo_comb_flat!(2, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum FlowControl {
    Update = bitinfo_comb_flat!(1, 15),
    Pointer = bitinfo_comb_flat!(7, 8),
    Data = bitinfo_comb_flat!(8, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum SwitchIdentifier {
    ProductNum = bitinfo_comb_flat!(12, 4),
    Rev = bitinfo_comb_flat!(4, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PortControl0 {
    SaFiltering = bitinfo_comb_flat!(2, 14),
    EgressMode = bitinfo_comb_flat!(2, 12),
    Header = bitinfo_comb_flat!(1, 11),
    IgmpMldSnoop = bitinfo_comb_flat!(1, 10),
    FrameMode = bitinfo_comb_flat!(2, 8),
    VlanTunnel = bitinfo_comb_flat!(1, 7),
    TagIfBoth = bitinfo_comb_flat!(1, 6),
    InitialPri = bitinfo_comb_flat!(2, 4),
    EgressFloods = bitinfo_comb_flat!(2, 2),
    PortState = bitinfo_comb_flat!(2, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PortControl1 {
    MessagePort = bitinfo_comb_flat!(1, 15),
    LagPort = bitinfo_comb_flat!(1, 14),
    VtuPage = bitinfo_comb_flat!(1, 13),
    LagId = bitinfo_comb_flat!(5, 8),
    Fid11_4 = bitinfo_comb_flat!(8, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PortBasedVlanMap {
    Fid3_0 = bitinfo_comb_flat!(4, 12),
    ForceMap = bitinfo_comb_flat!(1, 11),
    VlanTable = bitinfo_comb_flat!(10, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum DefaultVlanIdPriority {
    DefFPri = bitinfo_comb_flat!(3, 13),
    ForceDefaultVid = bitinfo_comb_flat!(1, 12),
    DefaultVid = bitinfo_comb_flat!(12, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PortControl2 {
    ForceGoodFcs = bitinfo_comb_flat!(1, 15),
    AllowBad = bitinfo_comb_flat!(1, 14),
    JumboMode = bitinfo_comb_flat!(2, 12),
    X8021QMode = bitinfo_comb_flat!(2, 10),
    DiscardTagged = bitinfo_comb_flat!(1, 9),
    DiscardUntagged = bitinfo_comb_flat!(1, 8),
    MapDa = bitinfo_comb_flat!(1, 7),
    ArpMirror = bitinfo_comb_flat!(1, 6),
    EgressMonitorSource = bitinfo_comb_flat!(1, 5),
    IngressMonitorSource = bitinfo_comb_flat!(1, 4),
    AllowVid0 = bitinfo_comb_flat!(1, 3),
    DefQPri = bitinfo_comb_flat!(3, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum EgressRateControl {
    FrameOverhead = bitinfo_comb_flat!(4, 8),
    EgressDec = bitinfo_comb_flat!(7, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum EgressRateControl2 {
    CountMode = bitinfo_comb_flat!(2, 14),
    EgressRate = bitinfo_comb_flat!(14, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PortAssociationVector {
    HoldAt1 = bitinfo_comb_flat!(1, 15),
    IntOnAgeOut = bitinfo_comb_flat!(1, 14),
    LockedPort = bitinfo_comb_flat!(1, 13),
    IgnoreWrongdata = bitinfo_comb_flat!(1, 12),
    RefreshLocked = bitinfo_comb_flat!(1, 11),
    Pav = bitinfo_comb_flat!(10, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PortAtuControl {
    ReadLearnCnt = bitinfo_comb_flat!(1, 15),
    LimitReached = bitinfo_comb_flat!(1, 14),
    OverLimitIntEn = bitinfo_comb_flat!(1, 13),
    KeepOldLearnLimit = bitinfo_comb_flat!(1, 12),
    LearnLimitLearnCnt = bitinfo_comb_flat!(10, 0),
}

#[derive(Debug, Clone, Copy, EnumIter, EnumString, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum Override {
    DaQPriOverride = bitinfo_comb_flat!(1, 15),
    DaFPriOverride = bitinfo_comb_flat!(1, 14),
    SaQPriOverride = bitinfo_comb_flat!(1, 13),
    SaFPriOverride = bitinfo_comb_flat!(1, 12),
    VtuQPriOverride = bitinfo_comb_flat!(1, 11),
    VtuFPriOverride = bitinfo_comb_flat!(1, 10),
    MirrorSaMiss = bitinfo_comb_flat!(1, 9),
    MirrorVtuMiss = bitinfo_comb_flat!(1, 8),
    TrapDaMiss = bitinfo_comb_flat!(1, 7),
    TrapSaMiss = bitinfo_comb_flat!(1, 6),
    TrapVtuMiss = bitinfo_comb_flat!(1, 5),
    TrapTcamMiss = bitinfo_comb_flat!(1, 4),
    TcamMode = bitinfo_comb_flat!(3, 0),
}

pub enum PolicyMgmtControl {
    IndexMode = bitinfo_comb_flat!(2, 14),
    Pointer = bitinfo_comb_flat!(6, 8),
    Data = bitinfo_comb_flat!(8, 0),
}

pub enum ExtendedPortControlCmd {
    EpcBusy = bitinfo_comb_flat!(1, 15),
    EpcOp = bitinfo_comb_flat!(3, 12),
    EpcIndex = bitinfo_comb_flat!(8, 0),
}

pub enum ExtendedPortControlData {
    Data = bitinfo_comb_flat!(16, 0),
}

pub enum PreemptionControl {
    PreemptVerify = bitinfo_comb_flat!(1, 15),
    PreemptStatus = bitinfo_comb_flat!(1, 14),
    PreemptQbv = bitinfo_comb_flat!(1, 13),
    PreemptDrop = bitinfo_comb_flat!(1, 11),
    PreemptEnable = bitinfo_comb_flat!(1, 10),
    PreemptSize = bitinfo_comb_flat!(2, 8),
    PreemptQueue = bitinfo_comb_flat!(8, 0),
}

pub enum LedControl {
    Update = bitinfo_comb_flat!(1, 15),
    Pointer = bitinfo_comb_flat!(3, 12),
    Data = bitinfo_comb_flat!(12, 0),
}

pub enum IpPriorityMappingTable {
    Update = bitinfo_comb_flat!(1, 15),
    Pointer = bitinfo_comb_flat!(6, 9),
    IpYellow = bitinfo_comb_flat!(1, 8),
    DislpQPri = bitinfo_comb_flat!(1, 7),
    IpQPri = bitinfo_comb_flat!(3, 4),
    DislpFPri = bitinfo_comb_flat!(1, 3),
    IpFPri = bitinfo_comb_flat!(3, 0),
}

pub enum IeeePriorityMappingTable {
    Update = bitinfo_comb_flat!(1, 15),
    Table = bitinfo_comb_flat!(3, 12),
    Pointer = bitinfo_comb_flat!(3, 9),
    Data = bitinfo_comb_flat!(9, 0),
}

pub enum PortControl3 {
    RtagStripEn = bitinfo_comb_flat!(1, 9),
    DsaStripEn = bitinfo_comb_flat!(1, 8),
    FloodUnknowns = bitinfo_comb_flat!(1, 7),
    LearnDisable = bitinfo_comb_flat!(1, 6),
    UpdateDscp = bitinfo_comb_flat!(1, 5),
    UpdateDei = bitinfo_comb_flat!(1, 3),
    UpdateCfi = bitinfo_comb_flat!(1, 2),
    UseDeiYellow = bitinfo_comb_flat!(1, 1),
    UseCfiYellow = bitinfo_comb_flat!(1, 0),
}

pub enum PortMiscScratch {
    Update = bitinfo_comb_flat!(1, 15),
    Pointer = bitinfo_comb_flat!(7, 8),
    Data = bitinfo_comb_flat!(8, 0),
}

pub enum QueueCounters {
    Mode = bitinfo_comb_flat!(4, 12),
    SelfInc = bitinfo_comb_flat!(1, 11),
    Data = bitinfo_comb_flat!(9, 0),
}

pub enum QueueControl {
    Update = bitinfo_comb_flat!(1, 15),
    Pointer = bitinfo_comb_flat!(7, 8),
    Data = bitinfo_comb_flat!(8, 0),
}

pub enum QueueControl2 {
    IndexMode = bitinfo_comb_flat!(2, 14),
    Pointer = bitinfo_comb_flat!(6, 8),
    Data = bitinfo_comb_flat!(8, 0),
}

pub enum EnableSelect {
    EnableSelect = bitinfo_comb_flat!(4, 12),
}

pub enum DebugCounters {
    RxBadTxCollisions = bitinfo_comb_flat!(8, 8),
    RxGoodTxTransmit = bitinfo_comb_flat!(8, 0),
}
