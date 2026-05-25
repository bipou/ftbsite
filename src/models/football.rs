use crate::models::{Category, PageInfo, Topic};
use serde::{Deserialize, Serialize};

// ── 赔率（footballs.lines 内嵌数组）────────────────────────────────────
// 数组按时间顺序排列，首条 = 初始赔率，末条 = 最新赔率
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Line {
    pub win: f32,
    pub draw: f32,
    pub loss: f32,
    pub created_at: String,
}

// ── 计算（footballs.calcs 内嵌数组）────────────────────────────────────
// 数组按时间顺序排列，首条 = 初始计算，末条 = 最新
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Calc {
    pub s: String,
    pub wdl: String,
    pub tg: String,
    pub gd: String,
    pub created_at: String,
}

/// A football match with all resolved relations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Football {
    pub id: String,
    pub category_id: String,
    pub season: String,
    pub home_team: String,
    pub away_team: String,
    /// Formatted "MM-DD HH:MM" UTC
    pub kick_off_at_mdhm: String,
    /// Formatted "MM-DD HH:MM" UTC+8
    pub kick_off_at_mdhm8: String,
    pub created_at: String,
    pub updated_at: String,
    pub hits: u64,
    pub stars: u64,
    /// Status: 4=both,3=picks,2=hot,1=published,0=draft,-1=deleted
    pub status: i8,
    /// 赛前赔率，il_pair 取首尾：[初始, 最新]
    /// il = Initial/Last，即历史序列首尾对
    pub il_odds: Vec<Line>,
    /// 赔率全量记录（详情页用）
    pub all_odds: Vec<Line>,
    /// 计算，il_pair 取首尾：[初始, 最新]
    pub il_calcs: Vec<Calc>,
    /// 计算全量记录（详情页用）
    pub all_calcs: Vec<Calc>,
    /// 正式赛果——比分，如 "3:1"（footballs 表直存，未完成则为 None）
    pub result_s: Option<String>,
    /// 正式赛果——胜平负（3=胜 / 1=平 / 0=负）
    pub result_wdl: Option<u8>,
    /// 正式赛果——总进球（≥0）
    pub result_tg: Option<u8>,
    /// 正式赛果——净胜球（可负）
    pub result_gd: Option<i8>,
    pub category: Option<Category>,
    pub topics: Vec<Topic>,
}

impl Football {
    pub fn match_title(&self) -> String {
        format!("{} vs {}", self.home_team, self.away_team)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FootballsResult {
    pub page_info: PageInfo,
    pub items: Vec<Football>,
}
