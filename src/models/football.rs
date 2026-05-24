use crate::models::{Category, PageInfo, Topic};
use serde::{Deserialize, Serialize};

// ── 赔率（footballs_lines 表）────────────────────────────────────────────
// 同一球赛可有多行，按 created_at ASC 排列。
// il_pair 取首尾：第一条 = 初始赔率，最后一条 = 最新赔率（赛前更新）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FootballLine {
    pub id: String,
    /// 主胜赔率
    pub win: String,
    /// 平局赔率
    pub draw: String,
    /// 客胜赔率
    pub loss: String,
    pub created_at: String,
}

// ── 计算 / 赛果（footballs_overs 表）─────────────────────────────────────
// 同一球赛可有多行，按 created_at ASC 排列。
// il_pair 取首尾：第一条 = 初始计算，最后一条 = 最新。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FootballOver {
    pub id: String,
    /// 比分，如 "2:1"
    pub s: String,
    /// 胜平负，如 "胜"
    pub wdl: String,
    /// 总进球数
    pub tg: String,
    /// 净胜球，如 "+1"
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
    pub il_odds: Vec<FootballLine>,
    /// 赔率全量记录（详情页用）
    pub all_odds: Vec<FootballLine>,
    /// 赛前计算，il_pair 取首尾：[初始, 最新]
    /// il = Initial/Last
    pub il_calc_over: Vec<FootballOver>,
    /// 计算全量记录（详情页用）
    pub all_calc_over: Vec<FootballOver>,
    /// 赛后正式结果，取最后一条
    pub football_over: Option<FootballOver>,
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
