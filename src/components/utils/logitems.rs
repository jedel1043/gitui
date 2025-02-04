use asyncgit::sync::{CommitId, CommitInfo};
use chrono::{DateTime, Duration, Local, NaiveDateTime, Utc};
use std::slice::Iter;

static SLICE_OFFSET_RELOAD_THRESHOLD: usize = 100;

pub struct LogEntry {
	pub time: DateTime<Local>,
	pub author: String,
	pub msg: String,
	pub hash_short: String,
	pub id: CommitId,
}

impl From<CommitInfo> for LogEntry {
	fn from(c: CommitInfo) -> Self {
		let time =
			DateTime::<Local>::from(DateTime::<Utc>::from_utc(
				NaiveDateTime::from_timestamp(c.time, 0),
				Utc,
			));
		Self {
			author: c.author,
			msg: c.message,
			time,
			hash_short: c.id.get_short_string(),
			id: c.id,
		}
	}
}

impl LogEntry {
	pub fn time_to_string(&self, now: DateTime<Local>) -> String {
		let delta = now - self.time;
		if delta < Duration::minutes(30) {
			let delta_str = if delta < Duration::minutes(1) {
				"<1m ago".to_string()
			} else {
				format!("{:0>2}m ago", delta.num_minutes())
			};
			format!("{: <10}", delta_str)
		} else if self.time.date() == now.date() {
			self.time.format("%T  ").to_string()
		} else {
			self.time.format("%Y-%m-%d").to_string()
		}
	}
}

///
#[derive(Default)]
pub struct ItemBatch {
	index_offset: usize,
	items: Vec<LogEntry>,
}

impl ItemBatch {
	fn last_idx(&self) -> usize {
		self.index_offset + self.items.len()
	}

	///
	pub const fn index_offset(&self) -> usize {
		self.index_offset
	}

	/// shortcut to get an `Iter` of our internal items
	pub fn iter(&self) -> Iter<'_, LogEntry> {
		self.items.iter()
	}

	/// clear curent list of items
	pub fn clear(&mut self) {
		self.items.clear();
	}

	/// insert new batch of items
	pub fn set_items(
		&mut self,
		start_index: usize,
		commits: Vec<CommitInfo>,
	) {
		self.items.clear();
		self.items.extend(commits.into_iter().map(LogEntry::from));
		self.index_offset = start_index;
	}

	/// returns `true` if we should fetch updated list of items
	pub fn needs_data(&self, idx: usize, idx_max: usize) -> bool {
		let want_min =
			idx.saturating_sub(SLICE_OFFSET_RELOAD_THRESHOLD);
		let want_max = idx
			.saturating_add(SLICE_OFFSET_RELOAD_THRESHOLD)
			.min(idx_max);

		let needs_data_top = want_min < self.index_offset;
		let needs_data_bottom = want_max >= self.last_idx();
		needs_data_bottom || needs_data_top
	}
}
