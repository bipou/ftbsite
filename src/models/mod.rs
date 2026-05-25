pub mod category;
pub mod football;
pub mod topic;
pub mod user;

pub use category::Category;
#[cfg(feature = "oth")]
pub use football::{Calc, Line};
pub use football::{
    Football, FootballAnalysis, FootballsResult, LineupPlayer, FootballEvent, FootballStats, TeamLineup,
};
pub use topic::Topic;
pub use user::{AuthUser, User, UserSummary, UsersResult};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PageInfo {
    pub current_page: u32,
    pub total_pages: u32,
    pub total_count: u64,
    pub has_previous: bool,
    pub has_next: bool,
}
