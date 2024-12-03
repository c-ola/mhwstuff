use super::*;
use crate::rsz_struct;
use serde::*;
use enums::*;

rsz_struct! {
    #[rsz("app.user_data.ItemData", 0xbba858c = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ItemData {
        values: Vec<ItemDatacData>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.ItemData.cData", 0x8a3d34ff = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ItemDatacData {
        index: i32,
        item_id: ItemDefIdFixed,
        raw_name: Guid,
        raw_explain: Guid,
        sort_id: i16,
        r#type: ItemDefTypeFixed,
        text_type: ItemDefTextTypeFixed,
        icon_type: IconDefItemFixed,
        equip_icon: IconDefEquipFixed,
        icon_color: ColorPresetTypeFixed,
        add_icon_type: IconDefAddIconFixed,
        rare: ItemDefRareFixed,
        max_count: i16,
        otomo_max: i16,
        enable_on_raptor: bool,
        sell_price: i32,
        buy_price: i32,
        fix: bool,
        shikyu: bool,
        eatable: bool,
        window: bool,
        infinit: bool,
        heal: bool,
        battle: bool,
        special: bool,
        for_money: bool,
        out_box: bool,
        non_level_shell: bool,
        get_rank: Vec<ItemDefGetRankFixed>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.FixItems", 0xbdc8bc3e = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct FixItems {
        values: Vec<FixItemscData>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.FixItems.cData", 0x15516abc = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct FixItemscData {
        index: i32,
        item_id: ItemDefIdFixed,
        story_package: StoryPackageFlagTypeFixed,
    }
}
rsz_struct! {
    #[rsz("app.user_data.cItemRecipe", 0x89969066 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ItemRecipe {
        values: Vec<ItemRecipecData>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.cItemRecipe.cData", 0x2641cdc7 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct ItemRecipecData {
        index: i32,
        item_recipe_id: ItemRecipeDefIdFixed,
        result_item: ItemDefIdFixed,
        item: Vec<ItemDefIdFixed>,
        num: i16,
        open_status: bool,
        enable_auto: bool,
        default_auto_flag: bool,
    }
}

rsz_struct! {
    #[rsz("app.user_data.AutoUseHealthItemData", 0x892932eb = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct AutoUseHealthItemData {
        values: Vec<AutoUseHealthItemDataData>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.AutoUseHealthItemData.Data", 0x4fe2a48 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct AutoUseHealthItemDataData {
        lack_health: i32,
        item_id: ItemDefIdFixed,
    }
}

rsz_struct! {
    #[rsz("app.user_data.AutoUseStatusItemData", 0xdf669fc9 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct AutoUseStatusItemData {
        values: Vec<AutoUseStatusItemDataData>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.AutoUseStatusItemData.Data", 0xf5274b8e = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct AutoUseStatusItemDataData {
        bad_condition_fixed: HunterDefBadConditionFixed,
        item_id: ItemDefIdFixed,
    }
}
