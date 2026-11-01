//! The strokes of each [`BarIcon`].

use super::BarIcon;
use super::Stroke::{self, Circle, Closed, Open};

const fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> [(f32, f32); 4] {
    [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

impl BarIcon {
    /// The lines this icon is drawn with.
    pub fn strokes(self) -> &'static [Stroke] {
        match self {
            Self::Grid => &[
                Closed(&const { rect(3.0, 3.0, 10.0, 10.0) }),
                Closed(&const { rect(14.0, 3.0, 21.0, 10.0) }),
                Closed(&const { rect(3.0, 14.0, 10.0, 21.0) }),
                Closed(&const { rect(14.0, 14.0, 21.0, 21.0) }),
            ],
            Self::Window => &[
                Closed(&const { rect(3.0, 4.0, 21.0, 20.0) }),
                Open(&[(3.0, 8.0), (21.0, 8.0)]),
            ],
            Self::Clock => &[
                Circle((12.0, 12.0), 9.0),
                Open(&[(12.0, 7.0), (12.0, 12.0), (15.0, 14.0)]),
            ],
            Self::Calendar => &[
                Closed(&const { rect(3.0, 5.0, 21.0, 21.0) }),
                Open(&[(3.0, 10.0), (21.0, 10.0)]),
                Open(&[(8.0, 3.0), (8.0, 7.0)]),
                Open(&[(16.0, 3.0), (16.0, 7.0)]),
            ],
            Self::Memory => &[
                Closed(&const { rect(3.0, 7.0, 21.0, 17.0) }),
                Open(&[(7.0, 10.0), (7.0, 14.0)]),
                Open(&[(11.0, 10.0), (11.0, 14.0)]),
                Open(&[(15.0, 10.0), (15.0, 14.0)]),
                Open(&[(5.0, 17.0), (5.0, 20.0)]),
                Open(&[(19.0, 17.0), (19.0, 20.0)]),
            ],
            Self::Processor => &[
                Closed(&const { rect(6.0, 6.0, 18.0, 18.0) }),
                Closed(&const { rect(10.0, 10.0, 14.0, 14.0) }),
                Open(&[(9.0, 2.0), (9.0, 6.0)]),
                Open(&[(15.0, 2.0), (15.0, 6.0)]),
                Open(&[(9.0, 18.0), (9.0, 22.0)]),
                Open(&[(15.0, 18.0), (15.0, 22.0)]),
                Open(&[(2.0, 9.0), (6.0, 9.0)]),
                Open(&[(2.0, 15.0), (6.0, 15.0)]),
                Open(&[(18.0, 9.0), (22.0, 9.0)]),
                Open(&[(18.0, 15.0), (22.0, 15.0)]),
            ],
            Self::Download => &[
                Open(&[(12.0, 3.0), (12.0, 15.0)]),
                Open(&[(7.0, 10.0), (12.0, 15.0), (17.0, 10.0)]),
                Open(&[(4.0, 20.0), (20.0, 20.0)]),
            ],
            Self::Pause => &[
                Open(&[(8.0, 5.0), (8.0, 19.0)]),
                Open(&[(16.0, 5.0), (16.0, 19.0)]),
            ],
            Self::Music => &[
                Open(&[(9.0, 18.0), (9.0, 5.0), (20.0, 3.0), (20.0, 16.0)]),
                Circle((6.0, 18.0), 3.0),
                Circle((17.0, 16.0), 3.0),
            ],
            Self::Columns => &[
                Closed(&const { rect(3.0, 3.0, 21.0, 21.0) }),
                Open(&[(12.0, 3.0), (12.0, 21.0)]),
                Open(&[(12.0, 12.0), (21.0, 12.0)]),
            ],
        }
    }
}
