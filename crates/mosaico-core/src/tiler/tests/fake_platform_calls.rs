//! The changes a [`FakePlatform`](super::FakePlatform) records.

use crate::config::{CornerStyle, HidingBehaviour};
use crate::frames::Frame;
use crate::tiler::BorderStyle;
use crate::{Rect, WindowId};

/// A change the manager asked the platform to make.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Call {
    Place(WindowId, Rect),
    PrePlace(WindowId, Rect),
    PlaceFrames(Vec<Frame>),
    SettleResponding,
    Repaint(WindowId),
    Focus(WindowId),
    PutBehind(WindowId, WindowId),
    Close(WindowId),
    Minimize(WindowId),
    Hide(WindowId, HidingBehaviour),
    Show(WindowId, HidingBehaviour),
    Reveal(WindowId),
    MoveCursor(i32, i32),
    ShowBorder(WindowId, BorderStyle),
    HideBorder(WindowId),
    MoveBorder(WindowId, Rect),
    DropBorder(WindowId),
    SetCorners(WindowId, CornerStyle),
    ResetCorners(WindowId),
}
