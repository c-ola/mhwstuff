use super::*;
use enums::*;
use crate::rsz_struct;
use serde::*;

rsz_struct! {
    #[rsz("app.user_data.SkillCommonData", 0x73facd33 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct SkillCommonData {
        values: Vec<SkillCommonDatacData>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.SkillCommonData.cData", 0x3646d59 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct SkillCommonDatacData {
        index: i32,
        skill_id: HunterDefSkillFixed,
        skill_type: HunterDefSkillTypeFixed,
        skill_category: HunterDefSkillCategoryFixed,
        skill_icon_type: IconDefSkillFixed,
        skill_name: Guid,
        skill_explain: Guid,
        sort_id: i32,
    }
}
rsz_struct! {
    #[rsz("app.user_data.SkillData", 0x7a93f660 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct SkillData {
        values: Vec<SkillDatacData>,
    }
}
rsz_struct! {
    #[rsz("app.user_data.SkillData.cData", 0x334f6407 = 0)]
    #[derive(Debug, Serialize)]
    #[allow(dead_code)]
    pub struct SkillDatacData {
        index: i32,
        data_id: i32,
        skill_id: HunterDefSkillFixed,
        skill_lv: i32,
        skill_name: Guid,
        skill_explain: Guid,
        open_skill: Vec<HunterDefSkillFixed>,
        value: Vec<i32>,
    }
}
