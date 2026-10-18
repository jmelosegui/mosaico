//! The heights of the windows of a column the user resized: the stack of
//! the vertical stack, or a side column of the three columns.
//!
//! A column nobody resized keeps none, and its windows share it equally,
//! as columns always did. A resized one gives each of its windows a share
//! of its height without its gaps, the shares summing to 1. They are kept
//! as where each window but the last ends, a chain of splits down the
//! column, so that moving one of them leaves every other edge where it
//! was, to the pixel. Every window keeps at least [`MIN_SPLIT_RATIO`] of
//! an equal share. A window that joins gets an equal share, which the
//! others give up in proportion, and one that leaves gives its share to
//! the others in proportion.

use super::MIN_SPLIT_RATIO;

/// The heights of the windows of a column, or none for a column nobody
/// resized.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct Heights(Option<Vec<f64>>);

impl Heights {
    /// Heights that give each window of a column its share in `shares`,
    /// top down, scaled to sum to 1 and each kept at [`least`] or more;
    /// none for a column of fewer than two windows.
    pub(super) fn of(shares: &[f64]) -> Self {
        if shares.len() < 2 {
            return Self(None);
        }
        let shares = floored(shares);
        let ends = shares.iter().scan(0.0, |end, share| {
            *end += share;
            Some(*end)
        });
        Self(Some(ends.take(shares.len() - 1).collect()))
    }

    /// Where each window but the last ends, as a share of the column
    /// without its gaps, top down; none for a column nobody resized.
    pub(super) fn ends(&self) -> Option<&[f64]> {
        self.0.as_deref()
    }

    /// The share of the column each window has, top down.
    pub(super) fn shares(&self) -> Option<Vec<f64>> {
        let ends = self.0.as_ref()?;
        let tops = std::iter::once(0.0).chain(ends.iter().copied());
        let bottoms = ends.iter().copied().chain(std::iter::once(1.0));
        Some(
            bottoms
                .zip(tops)
                .map(|(bottom, top)| bottom - top)
                .collect(),
        )
    }

    /// The share of the column the window at `at` has.
    pub(super) fn share(&self, at: usize) -> Option<f64> {
        self.shares()?.get(at).copied()
    }

    /// A window joins the column at `at` with an equal share, which the
    /// others give up in proportion. A column nobody resized stays so.
    pub(super) fn join(&mut self, at: usize) {
        if let Some(mut shares) = self.shares() {
            let count = shares.len() as f64;
            shares
                .iter_mut()
                .for_each(|share| *share *= count / (count + 1.0));
            shares.insert(at.min(shares.len()), 1.0 / (count + 1.0));
            *self = Self::of(&shares);
        }
    }

    /// The window at `at` leaves the column, and its share goes to the
    /// others in proportion.
    pub(super) fn leave(&mut self, at: usize) {
        if let Some(mut shares) = self.shares() {
            shares.remove(at.min(shares.len() - 1));
            *self = Self::of(&shares);
        }
    }

    /// The column of `count` windows once those of `back` are back at their
    /// places with the shares they had: they keep them, and the others share
    /// what is left, or with no others they fill the column in proportion. A
    /// column none is back to stays as it is.
    pub(super) fn restore(&mut self, back: &[(usize, f64)], count: usize) {
        let back: Vec<&(usize, f64)> = back.iter().filter(|(at, _)| *at < count).collect();
        if back.is_empty() || count < 2 {
            return;
        }
        let brought: f64 = back.iter().map(|(_, share)| share).sum();
        let others = count.saturating_sub(back.len()).max(1) as f64;
        let mut shares = vec![((1.0 - brought) / others).max(0.0); count];
        back.iter().for_each(|&&(at, share)| shares[at] = share);
        *self = Self::of(&shares);
    }

    /// The column once the end of the window at `at`, in `ends`, moves to
    /// `end`: no closer than [`least`] to the top of that window or to the
    /// bottom of the one below, so that only those two change, and as the
    /// heights always keep that much, never against the move.
    pub(super) fn moved(ends: &[f64], at: usize, end: f64) -> Self {
        let least = least(ends.len() + 1);
        let above = at.checked_sub(1).map_or(0.0, |i| ends[i]);
        let below = ends.get(at + 1).copied().unwrap_or(1.0);
        let mut ends = ends.to_vec();
        if let Some(moved) = ends.get_mut(at).filter(|_| above + least <= below - least) {
            *moved = end.max(above + least).min(below - least);
        }
        Self(Some(ends))
    }
}

/// The least share of a column of `count` windows a window keeps:
/// [`MIN_SPLIT_RATIO`] of an equal share.
pub(super) fn least(count: usize) -> f64 {
    MIN_SPLIT_RATIO / count as f64
}

/// `shares` scaled to sum to 1, with each one below [`least`] raised to it
/// and the others giving up what that takes in proportion.
fn floored(shares: &[f64]) -> Vec<f64> {
    let total: f64 = shares.iter().sum();
    let (fair, equal) = (total > 0.0 && total.is_finite(), 1.0 / shares.len() as f64);
    let scale = |share: &f64| if fair { share / total } else { equal };
    let mut shares: Vec<f64> = shares.iter().map(scale).collect();
    let least = least(shares.len());
    let mut raised = vec![false; shares.len()];
    while let Some(low) = (0..shares.len()).find(|&i| !raised[i] && shares[i] < least) {
        raised[low] = true;
        let kept: f64 = (0..shares.len())
            .filter(|&i| !raised[i])
            .map(|i| shares[i])
            .sum();
        let room = 1.0 - least * raised.iter().filter(|&&up| up).count() as f64;
        for (share, &up) in shares.iter_mut().zip(&raised) {
            *share = if up { least } else { *share * room / kept };
        }
    }
    shares
}

#[cfg(test)]
#[path = "heights_tests.rs"]
mod tests;
