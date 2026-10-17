//! The splits and the heights of a workspace's layouts, kept in step with
//! its windows as they come and go, and what each window owns of them.
//!
//! The BSP chain keeps the split of every window that stays: a window that
//! joins gets one that follows the default, and one that leaves takes its
//! own with it. A resized stack of the vertical stack gives a window that
//! joins an equal share, which the others give up in proportion, and one
//! that leaves gives its share back to the others in proportion. A window
//! that takes the master's place brings the master to the top of the
//! stack, and a master that leaves brings the top of the stack up. In the
//! three columns, the windows after the master alternate between the left
//! column and the right one, so one that comes or goes before the last of
//! them moves every one after it to the other column, and both columns go
//! back to equal heights. One that joins after the last, or the last that
//! leaves, changes only its own column, as in the stack.

use crate::LayoutKind;

use super::SplitRatios;

/// What a window owns of the splits and heights of a workspace: its split
/// of the BSP chain, and its share of the stack and of its side column,
/// what it takes with it to another monitor, and brings back.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct OwnSplits {
    bsp: Option<f64>,
    stack: Option<f64>,
    column: Option<f64>,
}

impl SplitRatios {
    /// Makes room for a window inserted at `index` among `count` windows:
    /// the BSP chain gets a split at split `index`, or at its end, as the
    /// first window gets none, and the stacks a window, as this module
    /// says.
    pub(crate) fn insert(&mut self, index: usize, count: usize) {
        insert_split(&mut self.bsp, index, count);
        if count > 0 {
            self.stack.join(index.saturating_sub(1));
        }
        match index.checked_sub(1) {
            Some(side) if side + 1 >= count => self.columns[side % 2].join(side / 2),
            None if count < 2 => {}
            _ => self.columns = Default::default(),
        }
    }

    /// Forgets the split of the window removed from `index` among `count`
    /// windows: split `index` of the BSP chain, or the last one for the
    /// last window, and its share of the stacks, as this module says.
    pub(crate) fn remove(&mut self, index: usize, count: usize) {
        remove_split(&mut self.bsp, index, count);
        if count > 1 {
            self.stack.leave(index.saturating_sub(1));
        }
        match index.checked_sub(1) {
            Some(side) if side + 2 >= count => self.columns[side % 2].leave(side / 2),
            _ => self.columns = Default::default(),
        }
    }

    /// Forgets every ratio and every height the user gave, of every
    /// layout, so that each split follows the configured ratio and each
    /// stack is shared equally again, as after a retile. The BSP chain
    /// keeps a split for each window but the last.
    pub(crate) fn forget(&mut self) {
        self.bsp.fill(None);
        (self.vertical_stack, self.three_column) = (None, None);
        self.stack = Default::default();
        self.columns = Default::default();
    }

    /// What window `index` owns.
    pub(crate) fn owned_by(&self, index: usize) -> OwnSplits {
        let side = index.checked_sub(1);
        OwnSplits {
            bsp: self.bsp.get(index).copied().flatten(),
            stack: side.and_then(|side| self.stack.share(side)),
            column: side.and_then(|side| self.columns[side % 2].share(side / 2)),
        }
    }

    /// Gives each window of `back`, at `index` among `count` windows now and
    /// at `was` when it left, what it owned there, where it still has it:
    /// its split of the BSP chain, and its share of the stack unless it is
    /// now the master. In the stack and in each side column, the windows back
    /// keep their shares, and the others share what is left, or, with no
    /// others, the shares are scaled to fill it, so a whole stack back in its
    /// places has its heights back. A window back in the other side column
    /// moved the side windows after it there, as a close at home does, and
    /// sends both columns back to equal heights.
    pub(crate) fn give(&mut self, back: &[(usize, usize, OwnSplits)], count: usize) {
        for &(index, _, own) in back {
            if let Some(ratio) = own.bsp {
                self.set(LayoutKind::Bsp, index, ratio);
            }
        }
        let stack = |&(index, _, own): &(usize, usize, OwnSplits)| {
            Some((index.checked_sub(1)?, own.stack?))
        };
        let shares: Vec<(usize, f64)> = back.iter().filter_map(stack).collect();
        self.stack.restore(&shares, count.saturating_sub(1));
        let side = |window: usize| window.checked_sub(1).map(|at| at % 2);
        if back.iter().any(|&(index, was, _)| side(index) != side(was)) {
            self.columns = Default::default();
            return;
        }
        for column in 0..2 {
            let mine = |&(index, _, own): &(usize, usize, OwnSplits)| {
                let at = index.checked_sub(1).filter(|at| at % 2 == column)?;
                Some((at / 2, own.column?))
            };
            let shares: Vec<(usize, f64)> = back.iter().filter_map(mine).collect();
            let windows = count.saturating_sub(1 + column).div_ceil(2);
            self.columns[column].restore(&shares, windows);
        }
    }
}

/// Makes room in `chain` for a window inserted at `index` among `count`:
/// a split at `index`, or at the end for one that joins after the last.
fn insert_split(chain: &mut Vec<Option<f64>>, index: usize, count: usize) {
    if count > 0 {
        let at = index.min(count - 1).min(chain.len());
        chain.insert(at, None);
    }
}

/// Forgets the split in `chain` of the window removed from `index` among
/// `count`: its own, or the one before for the last window.
fn remove_split(chain: &mut Vec<Option<f64>>, index: usize, count: usize) {
    if count > 1 {
        let at = index.min(count - 2);
        if at < chain.len() {
            chain.remove(at);
        }
    }
}

#[cfg(test)]
#[path = "split_stack_chain_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "split_column_chain_tests.rs"]
mod column_tests;
