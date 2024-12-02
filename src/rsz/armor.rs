use super::common::*;
use super::*;
use crate::{rsz_enum, rsz_newtype, rsz_struct};
use serde::*;

rsz_struct! {
    #[rsz("app.ArmorDef.ARMOR_COLOR_TYPE", 0xcba23004 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefARMORCOLORTYPE {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.ARMOR_COLOR_TYPE_Fixed", 0x7fe302c6 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefARMORCOLORTYPEFixed {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.ARMOR_COLOR_TYPE_Serializable", 0xb8cc9ff2 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefARMORCOLORTYPESerializable {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.ARMOR_PARTS", 0x23bab15b = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefARMORPARTS {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.ARMOR_PARTS_Fixed", 0xb075452d = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefARMORPARTSFixed {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.ARMOR_PARTS_Serializable", 0xf1f7ae0f = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefARMORPARTSSerializable {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.ARM_PARTS", 0x9ba4269c = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefARMPARTS {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.AmuletColorType", 0xcec65855 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefAmuletColorType {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.AmuletColorType_Fixed", 0x14cf6ce6 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefAmuletColorTypeFixed {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.AmuletColorType_Serializable", 0x9796a20 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefAmuletColorTypeSerializable {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.AmuletType", 0x5d5613e9 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefAmuletType {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.AmuletType_Fixed", 0x58acd076 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefAmuletTypeFixed {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.AmuletType_Serializable", 0x785c7998 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefAmuletTypeSerializable {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.BODY_PARTS", 0x190b4706 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefBODYPARTS {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.MODEL_VARIETY", 0x996653ac = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefMODELVARIETY {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.MODEL_VARIETY_Fixed", 0x8b2fc2ae = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefMODELVARIETYFixed {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.MODEL_VARIETY_Serializable", 0x8829fd9d = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefMODELVARIETYSerializable {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.SERIES", 0x951752f7 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefSERIES {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.SERIES_Fixed", 0x57236311 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefSERIESFixed {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.SERIES_Serializable", 0x23b1beba = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefSERIESSerializable {
        value: i32,
    }
}
rsz_struct! {
    #[rsz("app.ArmorDef.WAIST_PARTS", 0xe79903c1 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ArmorDefWAISTPARTS {
        value: i32,
    }
}

