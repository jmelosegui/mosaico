//! The space `subscribe` keeps for its bar: what `--reserve` holds, and
//! what the bar reserved through it, sent again on every new session and
//! after every switch of provider.
//!
//! The daemon keeps one reservation per session and monitor, so the holds
//! and the bar's own reservation on a monitor are sent as one.

use std::str::FromStr;

use serde_json::{Map, Value, json};

use crate::tiler::Side;

use super::{JsonState, MonitorRef};

/// The first id of the requests `subscribe` sends on its own, whose answers
/// it does not print. A bar's own requests use ids below it.
pub const OWN_IDS: u64 = 1 << 63;

/// Space `subscribe --reserve` holds for as long as it runs, written
/// `EDGE=PIXELS` or `EDGE=PIXELS@MONITOR`, such as `top=32@m-41c6e2`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hold {
    /// The edge.
    pub side: Side,
    /// How far in from it, in the desktop's pixels.
    pub pixels: u32,
    /// The monitor, or `None` for the one `--monitor` names, or else all.
    pub monitor: Option<MonitorRef>,
}

impl FromStr for Hold {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (side, rest) = (text.split_once('='))
            .ok_or_else(|| format!("`{text}` is not EDGE=PIXELS, such as top=32"))?;
        let side = (Side::ALL.into_iter())
            .find(|s| s.name().eq_ignore_ascii_case(side.trim()))
            .ok_or_else(|| format!("`{side}` is not top, bottom, left or right"))?;
        let (pixels, monitor) = match rest.split_once('@') {
            Some((pixels, monitor)) => (pixels, Some(monitor.parse::<MonitorRef>()?)),
            None => (rest, None),
        };
        let pixels = (pixels.trim().parse::<u32>())
            .map_err(|_| format!("`{pixels}` is not a number of pixels"))?;
        Ok(Self {
            side,
            pixels,
            monitor,
        })
    }
}

/// The space one `subscribe` keeps.
pub(super) struct Space {
    holds: Vec<Hold>,
    /// The bar's reservations the daemon acknowledged, as they last stood.
    stood: Vec<Value>,
    /// The bar's `reserve` and `release` requests not answered yet.
    asked: Vec<(u64, Value)>,
    /// The requests it sent on its own and not answered yet.
    own: Vec<u64>,
    next: u64,
    /// The provider the space was last sent under, `None` until a
    /// session's snapshot comes or when it is to be sent again.
    provider: Option<Value>,
}

/// Where the space is sent from: the state, and the monitor `--monitor`
/// names, for the holds that name none.
pub(super) struct Context<'a> {
    state: &'a JsonState,
    monitor: Option<&'a MonitorRef>,
}

impl<'a> Context<'a> {
    pub fn new(state: &'a JsonState, monitor: Option<&'a MonitorRef>) -> Self {
        Self { state, monitor }
    }
}

impl Space {
    pub fn new(holds: Vec<Hold>) -> Self {
        Self {
            holds,
            stood: Vec::new(),
            asked: Vec::new(),
            own: Vec::new(),
            next: OWN_IDS,
            provider: None,
        }
    }

    /// Notes `request`, one of the bar's, as it is sent, and has a
    /// reservation keep the holds on its monitor too.
    pub fn forwarded(&mut self, request: &mut Value, at: &Context) {
        if let Some(Value::Object(reserve)) = request.get_mut("reserve") {
            let target = reserve.get("monitor").cloned().unwrap_or_default();
            for (side, pixels) in self.held_on(&target, at) {
                widen(reserve, side, pixels);
            }
        } else if request.get("release").is_none() {
            return;
        }
        if let Some(id) = request["id"].as_u64() {
            self.asked.push((id, request.clone()));
        }
    }

    /// Takes in `message` from the daemon. `false` when it answers a
    /// request sent on its own, which is not printed.
    pub fn answered(&mut self, message: &Value) -> bool {
        let Some(id) = message["id"].as_u64() else {
            return true;
        };
        if let Some(at) = self.own.iter().position(|own| *own == id) {
            self.own.remove(at);
            return false;
        }
        if let Some(at) = self.asked.iter().position(|(asked, _)| *asked == id) {
            let (_, request) = self.asked.remove(at);
            if message.get("ok").is_some() {
                self.stand(&request);
            }
        }
        true
    }

    /// Keeps what the acknowledged `request` leaves standing. A release
    /// may have ended holds too, so the space is sent again.
    fn stand(&mut self, request: &Value) {
        apply(&mut self.stood, request);
        if request.get("release").is_some() {
            self.provider = None;
        }
    }

    /// The session ended: nothing asked will be answered, and the space is
    /// sent again once the next snapshot comes.
    pub fn lost(&mut self) {
        self.asked.clear();
        self.own.clear();
        self.provider = None;
    }

    /// The requests to send now: the holds and what the bar reserved, one
    /// per monitor, after a session's snapshot, a switch of provider or a
    /// release; none otherwise.
    pub fn due(&mut self, at: &Context) -> Vec<Value> {
        let provider = at
            .state
            .topic("daemon")
            .map(|daemon| daemon["provider"].clone());
        let provider = provider.unwrap_or_default();
        if self.provider.as_ref() == Some(&provider) {
            return Vec::new();
        }
        self.provider = Some(provider);
        let mut bodies: Vec<Map<String, Value>> = Vec::new();
        let mut body_for = |target: &Value| -> usize {
            let found = bodies
                .iter()
                .position(|body| body.get("monitor") == Some(target));
            found.unwrap_or_else(|| {
                let mut body = Map::new();
                body.insert("monitor".to_string(), target.clone());
                bodies.push(body);
                bodies.len() - 1
            })
        };
        let mut widened = Vec::new();
        for hold in &self.holds {
            if let Some(target) = self.target_of(hold, at) {
                widened.push((body_for(&target), hold.side.name(), u64::from(hold.pixels)));
            }
        }
        // What the bar asked and is not answered yet is sent before these,
        // and the daemon keeps the last reservation on a monitor, so they
        // carry it too rather than undo it.
        let mut standing = self.stood.clone();
        for (_, request) in &self.asked {
            apply(&mut standing, request);
        }
        for reserve in &standing {
            let index = body_for(&reserve["monitor"]);
            for side in Side::ALL {
                let pixels = reserve[side.name()].as_u64().unwrap_or(0);
                widened.push((index, side.name(), pixels));
            }
        }
        for (index, side, pixels) in widened {
            widen(&mut bodies[index], side, pixels);
        }
        (bodies.into_iter())
            .map(|reserve| {
                let id = self.next;
                self.next += 1;
                self.own.push(id);
                json!({"id": id, "reserve": reserve})
            })
            .collect()
    }

    /// The monitor `hold` is for, as the daemon names it: `"all"`, or an
    /// id, once the state has the monitor an index names.
    fn target_of(&self, hold: &Hold, at: &Context) -> Option<Value> {
        let target = match hold.monitor.as_ref().or(at.monitor) {
            None => Some("all".to_string()),
            Some(named) => named.resolve(at.state),
        };
        target.map(Value::String)
    }

    /// The edges the holds keep on `target`.
    fn held_on(&self, target: &Value, at: &Context) -> Vec<(&'static str, u64)> {
        (self.holds.iter())
            .filter(|hold| self.target_of(hold, at).as_ref() == Some(target))
            .map(|hold| (hold.side.name(), u64::from(hold.pixels)))
            .collect()
    }
}

/// Changes the reservations `standing` as the daemon does on taking in
/// `request`, a `reserve` or a `release`.
fn apply(standing: &mut Vec<Value>, request: &Value) {
    if let Some(reserve) = request.get("reserve") {
        standing.retain(|r| r["monitor"] != reserve["monitor"]);
        let reaches = Side::ALL
            .iter()
            .any(|s| reserve[s.name()].as_u64() > Some(0));
        if reaches {
            standing.push(reserve.clone());
        }
    } else if let Some(monitor) = request.get("release").map(|r| &r["monitor"]) {
        standing.retain(|r| !monitor.is_null() && r["monitor"] != *monitor);
    }
}

/// Has `reserve` reach `pixels` in from `side`, unless it reaches further.
fn widen(reserve: &mut Map<String, Value>, side: &str, pixels: u64) {
    let now = reserve.get(side).and_then(Value::as_u64).unwrap_or(0);
    if pixels > now {
        reserve.insert(side.to_string(), pixels.into());
    }
}
