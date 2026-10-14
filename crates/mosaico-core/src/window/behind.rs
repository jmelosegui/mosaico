//! Where a window goes when it is put behind another.

/// Where [`WindowControl::put_behind`](super::WindowControl::put_behind)
/// puts a window in the stacking order.
///
/// The windows that are always on top form a layer in front of every
/// window that is not, and putting a window behind another must not move
/// either of them out of its layer. So a window is only ordered against
/// another of its own layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Behind {
    /// Directly behind the other window, which is in the same layer.
    Other,
    /// Where it is, as the two windows are in different layers. It is
    /// behind the other already when only the other is always on top,
    /// and cannot go behind it when only this one is.
    Stay,
}

impl Behind {
    /// Where a window goes when it is put behind another, from whether
    /// each of them is always on top.
    pub fn new(window_on_top: bool, other_on_top: bool) -> Self {
        if window_on_top == other_on_top {
            Self::Other
        } else {
            Self::Stay
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_goes_directly_behind_another_of_its_layer() {
        // Arrange / Act / Assert -- in either layer.
        assert_eq!(Behind::new(false, false), Behind::Other);
        assert_eq!(Behind::new(true, true), Behind::Other);
    }

    #[test]
    fn a_window_stays_where_it_is_behind_one_of_the_other_layer() {
        // Arrange / Act / Assert -- going directly behind one always on
        // top would put the window on top too, and going behind one that
        // is not would take a window that is out of the layer on top.
        assert_eq!(Behind::new(false, true), Behind::Stay);
        assert_eq!(Behind::new(true, false), Behind::Stay);
    }
}
