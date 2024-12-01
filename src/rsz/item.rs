use super::*;
use crate::rsz_enum;
use crate::rsz_newtype;
use crate::rsz_struct;
use serde::*;
/*
// snow.data.GameItemEnum.CarriableFilter
rsz_enum! {
#[rsz(i32)]
#[derive(Debug, Serialize)]
pub enum CarriableFilter {
All = 0,
Quest = 1,
Hyakuryu = 2,
Lobby = 3,
}
}

// snow.data.DataDef.ItemTypes
rsz_enum! {
#[rsz(i32)]
#[derive(Debug, Serialize, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ItemTypes {
Consume = 0,
Tool = 1,
Material = 2,
OffcutsMaterial = 3,
Bullet = 4,
Bottle = 5,
Present = 6,
PayOff = 7,
CarryPayOff = 8,
Carry = 9,
Judge = 10,
Antique = 11,
}
}

// snow.data.GameItemEnum.IconRank
rsz_enum! {
#[rsz(i32)]
#[derive(Debug, Serialize)]
pub enum IconRank {
None = 0,
Great = 1,
Lv1 = 2,
Lv2 = 3,
Lv3 = 4,
Mystery = 5,
}
}

// snow.data.DataDef.RankTypes
rsz_enum! {
#[rsz(i32)]
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum RankTypes {
Low = 0,
Upper = 1,
Master = 2,
}
}

// snow.data.NormalItemData.ItemGroupTypes
rsz_enum! {
#[rsz(i32)]
#[derive(Debug, Serialize)]
pub enum ItemGroupTypes {
Drink = 0,
Food = 1,
Others = 2,
}
}

// snow.data.ContentsIdSystem.ItemId
rsz_enum! {
#[rsz(u32)]
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub enum ItemId {
    Null = 0, // not defined in TDB, but appears in some overwear data
    None = 0x04000000,
    Normal(u32) = 0x04100000..=0x0410FFFF,
    Ec(u32) = 0x04200000..=0x0420FFFF, // TODO: I_EC_0057 and up is offseted
}
}

// snow.data.DataDef.RareTypes
rsz_newtype! {
    #[rsz_offset(1)]
    #[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
    #[serde(transparent)]
    pub struct RareTypes(pub u8);
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum MaterialCategory {
        None = 0,
        LrHr(i32) = 1..=85,
        ArmorSphere = 86,
        Mr(i32) = 153..=1000,
    }
}

// Eh, please
impl MaterialCategory {
    pub fn from_msg_id(id: i32) -> MaterialCategory {
        if id < 200 {
            MaterialCategory::LrHr(id - 1)
        } else {
            MaterialCategory::Mr(id - 200)
        }
    }
}

rsz_struct! {
    #[rsz("snow.data.ItemUserData.Param",
        0x4A0D92B0 = 15_00_00,
        0x76FB6537 = 14_00_00,
        0xD0C19D16 = 13_00_00,
        0xc4940266 = 10_00_02,
        0xB8376E37 = 11_00_01,
        0xBF248F26 = 12_00_00
    )]
        #[derive(Debug, Serialize)]
        pub struct ItemUserDataParam {
            pub id: ItemId,
            pub cariable_filter: CarriableFilter,
            pub type_: ItemTypes,
            pub rare: RareTypes,
            pub pl_max_count: u32,
            pub ot_max_count: u32,
            pub sort_id: u32,
            pub supply: bool,
            pub can_put_in_dog_pouch: bool,
            pub show_item_window: bool,
            pub show_action_window: bool,
            pub infinite: bool,
            pub default: bool,
            pub icon_can_eat: bool,
            pub icon_item_rank: IconRank,
            pub effect_rare: bool,
            pub icon_chara: i32, // snow.gui.SnowGuiCommonUtility.Icon.ItemIconPatternNo
            pub icon_color: i32, // snow.gui.SnowGuiCommonUtility.Icon.ItemIconColor
            pub se_type: i32, // snow.data.GameItemEnum.SeType
            pub sell_price: u32,
            pub buy_price: u32,
            pub item_action_type: i32, // snow.data.GameItemEnum.ItemActionType
            pub rank_type: RankTypes,
            pub item_group: ItemGroupTypes,
            pub category_worth: u32,
            pub material_category: Vec<MaterialCategory>,
            pub evaluation_value: u32,
        }
}

rsz_struct! {
    #[rsz("snow.data.ItemUserData",
        path = "data/System/ContentsIdSystem/Item/Normal/ItemData.user",
        0x66200423 = 0
    )]
        #[derive(Debug, Serialize)]
        pub struct ItemUserData {
            pub param: Vec<ItemUserDataParam>,
        }
}

// snow.data.ContentsIdSystem.LvBuffCageId
rsz_enum! {
    #[rsz(u32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum LvBuffCageId {
        CommonNone = 0x18000000,
        CommonError = 0x18000001,
        CommonMax = 0x18000002,
        Normal(u32) = 0x18100000..= 0x1810FFFF
    }
}

rsz_struct! {
    #[rsz("snow.data.NormalLvBuffCageBaseUserData.Param",
        0x1026C5DC = 10_00_02
    )]
        #[derive(Debug, Serialize)]
        pub struct NormalLvBuffCageBaseUserDataParam {
            pub id: LvBuffCageId,
            pub sort_index: u32,
            pub rarity: RareTypes,
            pub model_lv: i32, // snow.equip.LvBuffCageModelLv
            pub model_color_index: ColorTypes,
            pub status_buff_limit: [u32; 5], // Health, Stamina, Attack, Defense, ?(always 3)
            pub status_buff_add_value: [u32; 4], // Health, Stamina, Attack, Defense
            pub status_buff_all_add_value: [u32; 4], // Health, Stamina, Attack, Defense
            pub status_start_revise_val: [u32; 5], // all zero?
            pub element_revise_val: [u32; 5], // all zero?
        }
}

rsz_struct! {
    #[rsz("snow.data.NormalLvBuffCageBaseUserData",
        path = "data/System/ContentsIdSystem/LvBuffCage/Normal/NormalLvBuffCageBaseData.user",
        0x849E4F82 = 10_00_02
    )]
        #[derive(Debug, Serialize)]
        pub struct NormalLvBuffCageBaseUserData {
            pub param: Vec<NormalLvBuffCageBaseUserDataParam>
        }
}*/

// app.ItemDef.TYPE_Fixed
rsz_enum! {
    #[rsz(u32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum ItemDefTypeFixed {
        EXPENDABLE = 0,
        TOOL = 1,
        MATERIAL = 2,
        SHELL = 3,
        BOTTLE = 4,
        POINT = 5,
        GEM = 6,
    }
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum TEXT_TYPE_Fixed {
        INVALID = 0,
        TYPE_00 = 1,
        TYPE_01 = 2,
        TYPE_02 = 3,
        TYPE_03 = 4,
        TYPE_04 = 5,
        TYPE_05 = 6,
        TYPE_06 = 7,
        TYPE_07 = 8,
        TYPE_08 = 9,
        TYPE_09 = 10,
        TYPE_10 = 11,
        TYPE_11 = 12,
        TYPE_12 = 13,
        TYPE_13 = 15,
        MAX = 14,
    }
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum ITEM_Fixed {
        INVALID = 0,
        ITEM_0000 = 1,
        ITEM_0001 = 2,
        ITEM_0002 = 3,
        ITEM_0003 = 4,
        ITEM_0004 = 5,
        ITEM_0005 = 6,
        ITEM_0006 = 7,
        ITEM_0007 = 8,
        ITEM_0008 = 9,
        ITEM_0009 = 10,
        ITEM_0010 = 11,
        ITEM_0011 = 12,
        ITEM_0012 = 13,
        ITEM_0013 = 14,
        ITEM_0014 = 15,
        ITEM_0015 = 16,
        ITEM_0016 = 17,
        ITEM_0017 = 18,
        ITEM_0018 = 19,
        ITEM_0019 = 20,
        ITEM_0020 = 21,
        ITEM_0021 = 22,
        ITEM_0022 = 23,
        ITEM_0023 = 24,
        ITEM_0024 = 25,
        ITEM_0025 = 26,
        ITEM_0026 = 27,
        ITEM_0027 = 28,
        ITEM_0028 = 29,
        ITEM_0029 = 30,
        ITEM_0030 = 31,
        ITEM_0031 = 32,
        ITEM_0032 = 33,
        ITEM_0033 = 34,
        ITEM_0034 = 35,
        ITEM_0035 = 36,
        ITEM_0036 = 37,
        ITEM_0037 = 38,
        ITEM_0038 = 39,
        ITEM_0039 = 40,
        ITEM_0040 = 41,
        ITEM_0041 = 42,
        ITEM_0042 = 43,
        ITEM_0043 = 44,
        ITEM_0044 = 45,
        ITEM_0045 = 46,
        ITEM_0046 = 47,
        ITEM_0047 = 48,
        ITEM_0048 = 49,
        ITEM_0049 = 50,
        ITEM_0050 = 51,
        ITEM_0051 = 52,
        ITEM_0052 = 53,
        ITEM_0053 = 54,
        ITEM_0054 = 55,
        ITEM_0055 = 56,
        ITEM_0056 = 57,
        ITEM_0057 = 58,
        ITEM_0058 = 59,
        ITEM_0059 = 60,
        ITEM_0060 = 61,
        ITEM_0061 = 62,
        ITEM_0062 = 63,
        ITEM_0063 = 64,
        ITEM_0064 = 65,
        ITEM_0065 = 66,
        ITEM_0066 = 67,
        ITEM_0067 = 68,
        ITEM_0068 = 69,
        ITEM_0069 = 70,
        ITEM_0070 = 71,
        ITEM_0071 = 72,
        ITEM_0072 = 73,
        ITEM_0073 = 74,
        ITEM_0074 = 75,
        ITEM_0075 = 76,
        ITEM_0076 = 77,
        ITEM_0077 = 78,
        ITEM_0078 = 79,
        ITEM_0079 = 80,
        ITEM_0080 = 81,
        ITEM_0081 = 82,
        ITEM_0082 = 83,
        ITEM_0083 = 84,
        ITEM_0084 = 85,
        ITEM_0085 = 86,
        ITEM_0086 = 87,
        ITEM_0087 = 88,
        ITEM_0088 = 89,
        ITEM_0089 = 90,
        ITEM_0090 = 91,
        ITEM_0091 = 92,
        ITEM_0092 = 93,
        ITEM_0093 = 94,
        ITEM_0094 = 95,
        ITEM_0095 = 96,
        ITEM_0096 = 97,
        ITEM_0097 = 98,
        ITEM_0098 = 99,
        ITEM_0099 = 100,
        ITEM_0100 = 101,
        MAX = 102,
    }
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum EQUIP_Fixed {
        INVALID = 0,
        EQUIP_0000 = 1,
        EQUIP_0001 = 2,
        EQUIP_0002 = 3,
        EQUIP_0003 = 4,
        EQUIP_0004 = 5,
        EQUIP_0005 = 6,
        EQUIP_0006 = 7,
        EQUIP_0007 = 8,
        EQUIP_0008 = 9,
        EQUIP_0009 = 10,
        EQUIP_0010 = 11,
        EQUIP_0011 = 12,
        EQUIP_0012 = 13,
        EQUIP_0013 = 14,
        EQUIP_0014 = 15,
        EQUIP_0015 = 16,
        EQUIP_0016 = 17,
        EQUIP_0017 = 18,
        EQUIP_0018 = 19,
        EQUIP_0019 = 20,
        EQUIP_0020 = 21,
        EQUIP_0021 = 22,
        EQUIP_0022 = 23,
        EQUIP_0023 = 24,
        EQUIP_0024 = 25,
        EQUIP_0025 = 26,
        EQUIP_0026 = 27,
        EQUIP_0027 = 28,
        EQUIP_0028 = 29,
        EQUIP_0029 = 30,
        EQUIP_0030 = 31,
        EQUIP_0031 = 32,
        EQUIP_0032 = 33,
        EQUIP_0033 = 34,
        EQUIP_0034 = 35,
        EQUIP_0035 = 36,
        EQUIP_0036 = 37,
        EQUIP_0037 = 38,
        EQUIP_0038 = 39,
        EQUIP_0039 = 40,
        EQUIP_0040 = 41,
        EQUIP_0041 = 42,
        EQUIP_0042 = 43,
        EQUIP_0043 = 44,
        EQUIP_0044 = 45,
        EQUIP_0045 = 46,
        EQUIP_0046 = 47,
        EQUIP_0047 = 48,
        EQUIP_0048 = 49,
        EQUIP_0049 = 50,
        EQUIP_0050 = 51,
        EQUIP_0051 = 52,
        EQUIP_0052 = 53,
        MAX = 54,
    }
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    enum TYPE_Fixed {
        I_NONE = 0,
        I_WHITE = 1,
        I_GRAY = 2,
        I_ROSE = 3,
        I_PINK = 4,
        I_RED = 5,
        I_VERMILION = 6,
        I_ORANGE = 7,
        I_BROWN = 8,
        I_IVORY = 9,
        I_YELLOW = 10,
        I_LEMON = 11,
        I_SGREEN = 12,
        I_MOS = 13,
        I_GREEN = 14,
        I_EMERALD = 15,
        I_SKY = 16,
        I_BLUE = 17,
        I_ULTRAMARINE = 18,
        I_BPURPLE = 19,
        I_PURPLE = 20,
        I_DPURPLE = 21,
        RARE_01 = 22,
        RARE_02 = 23,
        RARE_03 = 24,
        RARE_04 = 25,
        RARE_05 = 26,
        RARE_06 = 27,
        RARE_07 = 28,
        RARE_08 = 29,
        RARE_09 = 30,
        RARE_10 = 31,
        RARE_11 = 32,
        RARE_12 = 33,
        Rank_Prog00 = 34,
        Rank_Prog01 = 35,
        Rank_Prog02 = 36,
        Rank_Prog03 = 37,
        TXT_White01 = 38,
        TXT_White02 = 39,
        TXT_White03 = 40,
        TXT_Gray01 = 41,
        TXT_Black01 = 42,
        TXT_Safe = 43,
        TXT_Danger = 44,
        TXT_Accent = 45,
        TXT_Accent2 = 46,
        TXT_Accent3 = 47,
        TXT_Sub = 48,
        TXT_Max = 49,
        TXT_CharaName = 50,
        TXT_Choice_01 = 51,
        TXT_Choice_02 = 52,
        TXT_Title = 53,
        TXT_currency00num = 54,
        TXT_currency00unit = 55,
        TXT_currency01num = 56,
        TXT_currency01unit = 57,
        TXT_currency02num = 58,
        TXT_currency02unit = 59,
        TXT_currency03num = 60,
        TXT_currency03unit = 61,
        GUI_White = 62,
        GUI_Black = 63,
        GUI_Disable = 64,
        GUI_Safe = 65,
        GUI_Danger = 66,
        GUI_Acrtive01 = 67,
        GUI_Acrtive02 = 68,
        GUI_DShadow = 69,
        GUI_Psolo = 70,
        GUI_P1 = 71,
        GUI_P2 = 72,
        GUI_P3 = 73,
        GUI_P4 = 74,
        GUI_PNPC = 75,
        GUI_PStealth = 76,
        GUI_Tab00 = 77,
        GUI_Tab01 = 78,
        GUI_Tab02 = 79,
        GUI_Tab03 = 80,
        GUI_Tab04 = 81,
        GUI_Tab05 = 82,
        GUI_Tab06 = 83,
        GUI_MapEmWarningLv1 = 84,
        GUI_MapEmWarningLv2 = 85,
        GUI_MapEmWarningLv3 = 86,
        GUI_MapEmWarningLv4 = 87,
        GUI_MapEmWarningLv5 = 88,
        GUI_Sharp00 = 89,
        GUI_Sharp01 = 90,
        GUI_Sharp02 = 91,
        GUI_Sharp03 = 92,
        GUI_Sharp04 = 93,
        GUI_Sharp05 = 94,
        GUI_Sharp06 = 95,
        GUI_LSword_Spr00 = 96,
        GUI_LSword_Spr01 = 97,
        GUI_LSword_Spr02 = 98,
        GUI_Insect_Ext00 = 99,
        GUI_Insect_Ext01 = 100,
        GUI_Insect_Ext02 = 101,
        GUI_Insect_Ext03 = 102,
        GUI_Insect_Ext02_2 = 103,
        GUI_Horn_Note00 = 104,
        GUI_Horn_Note01 = 105,
        GUI_Horn_Note02 = 106,
        GUI_Horn_Note03 = 107,
        GUI_Horn_Note04 = 108,
        GUI_Horn_Note05 = 109,
        GUI_Horn_Note06 = 110,
        GUI_Horn_Note07 = 111,
        GUI_Horn_Activation = 112,
        GUI_Horn_ActivationAdd = 113,
        MAX = 114,
    }
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum AddIcon_Fixed {
        INVALID = 0,
        GREAT = 1,
        SHELL_LV1 = 2,
        SHELL_LV2 = 3,
        SHELL_LV3 = 4,
        UNMAKABLE = 5,
        MAKABLE = 6,
        SUPPLY = 7,
        CLEAR_ITEM = 8,
        WISH_ITEM = 9,
        ATTR_FIRE = 10,
        ATTR_WATER = 11,
        ATTR_ICE = 12,
        ATTR_ELEC = 13,
        ATTR_DRAGON = 14,
        ATTR_POISON = 15,
        ATTR_PARALYSIS = 16,
        ATTR_SLEEP = 17,
        ATTR_BOMB = 18,
        NON_TARGET = 19,
        RECIPE = 20,
        POSSESS = 21,
        FITTING = 22,
        EQUIPPED = 23,
        FORGEABLE = 24,
        HIGHLIGHT_MAIN = 25,
        HIGHLIGHT_USER = 26,
        EX_RECOMMEND = 27,
        FIXEDITEM = 28,
        FIXEDITEM_OFF = 29,
        ROTTING_CORPSE = 30,
        INGREDIENTS = 31,
        LOCK = 32,
        POSSESS_MULTI = 33,
        CHECK = 34,
        REPLACE = 35,
        CRYSTAL_CORPSE = 36,
        UNIDENTIFIED = 37,
        FOR_ATTACK = 38,
        MAX = 39,
    }
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum RARE_Fixed {
        RARE0 = 18,
        RARE1 = 17,
        RARE2 = 16,
        RARE3 = 15,
        RARE4 = 14,
        RARE5 = 13,
        RARE6 = 12,
        RARE7 = 11,
        RARE8 = 10,
        RARE9 = 9,
        RARE10 = 8,
        RARE11 = 7,
        MAX = 2081494400,
    }
}

rsz_enum! {
    #[rsz(i32)]
    #[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
    pub enum GET_RANK_Fixed {
        NONE = 0,
        NORMAL = 1,
        RARE = 2,
        SUPER_RARE = 3,
    }
}


rsz_struct! {
    #[rsz("app.user_data.ItemData.cData",
        0x5A8F4FB8 = 0
    )]
        #[derive(Debug, Serialize)]
        pub struct user_data_ItemData_cData {
            _Index: i32, // 0
            _ItemId: i32, // 4
            _RawName: Guid, // 8
            _RawExplain: Guid, // 24
            _align1: [u8; 2],
            _SortId: i16, // 40
            _Type: i32, // 44
            _TextType: i32, // 48
            _IconType: i32, // 52
            _EquipIcon: i32, // 56
            _IconColor: i32, // 60
            _AddIconType: i32, // 64
            _Rare: i32, // 68
            _MaxCount: i16, // 70
            _OtomoMax: i16, // 72 
            _EnableOnRaptor: bool,// 74
            _align2: [u8; 3], // 75
            _SellPrice: i32, // 75 -> 76
            _BuyPrice: i32, // 80
            _Fix: bool, // 84
            _Shikyu: bool, // 85
            _Eatable: bool, // 86
            _Window: bool, // 87
            _Infinit: bool, // 88
            _Heal: bool, // 89
            _Battle: bool, // 90
            _Special: bool, // 91
            _ForMoney: bool, // 92
            _OutBox: bool, // 93
            _NonLevelShell: bool, // 94
            _align3: [u8; 1], // 75
            _GetRank: i32, // 95 -> 96
        }

}
