use super::MonitorKey;

/// How strongly two keys are believed to be the same display, strongest
/// first. Matching runs one tier at a time so a weak match can never take
/// a display that a stronger one would have claimed.
#[derive(Clone, Copy)]
enum Tier {
    /// Same platform stable identifier.
    StableId,
    /// Same EDID including a known serial number.
    EdidSerial,
    /// Same EDID model plugged into the same port.
    EdidConnector,
    /// Same EDID model, and it is the only one of that model on both sides.
    UniqueModel,
}

const TIERS: [Tier; 4] = [
    Tier::StableId,
    Tier::EdidSerial,
    Tier::EdidConnector,
    Tier::UniqueModel,
];

/// Pairs displays from before a configuration change with displays after.
///
/// Returns one entry per element of `new`: the index into `old` of the
/// same physical display, or `None` if it was not connected before (or
/// could not be identified). Each old display is used at most once.
pub fn match_monitors(old: &[MonitorKey], new: &[MonitorKey]) -> Vec<Option<usize>> {
    let mut result: Vec<Option<usize>> = vec![None; new.len()];
    let mut old_used = vec![false; old.len()];

    for tier in TIERS {
        for (ni, new_key) in new.iter().enumerate() {
            if result[ni].is_some() {
                continue;
            }
            let found = old.iter().enumerate().position(|(oi, old_key)| {
                !old_used[oi] && tier_matches(tier, old_key, new_key, old, new, &old_used, &result)
            });
            if let Some(oi) = found {
                result[ni] = Some(oi);
                old_used[oi] = true;
            }
        }
    }
    result
}

fn tier_matches(
    tier: Tier,
    old_key: &MonitorKey,
    new_key: &MonitorKey,
    old: &[MonitorKey],
    new: &[MonitorKey],
    old_used: &[bool],
    new_matched: &[Option<usize>],
) -> bool {
    match tier {
        Tier::StableId => old_key.stable_id.is_some() && old_key.stable_id == new_key.stable_id,
        Tier::EdidSerial => match (&old_key.edid, &new_key.edid) {
            (Some(a), Some(b)) => a.serial.is_some() && a == b,
            _ => false,
        },
        Tier::EdidConnector => match (&old_key.edid, &new_key.edid) {
            (Some(a), Some(b)) => {
                a.same_model(b)
                    && old_key.connector.is_some()
                    && old_key.connector == new_key.connector
            }
            _ => false,
        },
        Tier::UniqueModel => {
            let (Some(a), Some(b)) = (&old_key.edid, &new_key.edid) else {
                return false;
            };
            if !a.same_model(b) {
                return false;
            }
            let same_model = |key: &MonitorKey| key.edid.as_ref().is_some_and(|e| e.same_model(b));
            let old_candidates = old
                .iter()
                .zip(old_used)
                .filter(|(k, used)| !**used && same_model(k))
                .count();
            let new_candidates = new
                .iter()
                .zip(new_matched)
                .filter(|(k, matched)| matched.is_none() && same_model(k))
                .count();
            old_candidates == 1 && new_candidates == 1
        }
    }
}
