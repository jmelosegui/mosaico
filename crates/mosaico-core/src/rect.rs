/// A rectangle representing a window's position and size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Horizontal center of the rectangle.
    pub fn center_x(&self) -> i32 {
        self.x + self.width / 2
    }

    /// Vertical center of the rectangle.
    pub fn center_y(&self) -> i32 {
        self.y + self.height / 2
    }

    /// Returns the number of overlapping pixels along the vertical axis.
    ///
    /// A positive value means the rectangles share vertical space,
    /// which is useful for determining left/right neighbors.
    pub fn vertical_overlap(&self, other: &Rect) -> i32 {
        let top = self.y.max(other.y);
        let bottom = (self.y + self.height).min(other.y + other.height);
        (bottom - top).max(0)
    }
}
