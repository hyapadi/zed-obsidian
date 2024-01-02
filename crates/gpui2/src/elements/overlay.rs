use smallvec::SmallVec;
use taffy::style::{Display, Position};

use crate::{
    point, AnyElement, BorrowWindow, Bounds, Element, IntoElement, LayoutId, ParentElement, Pixels,
    Point, Size, Style, WindowContext,
};

pub struct OverlayState {
    child_layout_ids: SmallVec<[LayoutId; 4]>,
}

pub struct Overlay {
    children: SmallVec<[AnyElement; 2]>,
    anchor_corner: AnchorCorner,
    fit_mode: OverlayFitMode,
    // todo!();
    anchor_position: Option<Point<Pixels>>,
    // position_mode: OverlayPositionMode,
}

/// overlay gives you a floating element that will avoid overflowing the window bounds.
/// Its children should have no margin to avoid measurement issues.
pub fn overlay() -> Overlay {
    Overlay {
        children: SmallVec::new(),
        anchor_corner: AnchorCorner::TopLeft,
        fit_mode: OverlayFitMode::SwitchAnchor,
        anchor_position: None,
    }
}

impl Overlay {
    /// Sets which corner of the overlay should be anchored to the current position.
    pub fn anchor(mut self, anchor: AnchorCorner) -> Self {
        self.anchor_corner = anchor;
        self
    }

    /// Sets the position in window co-ordinates
    /// (otherwise the location the overlay is rendered is used)
    pub fn position(mut self, anchor: Point<Pixels>) -> Self {
        self.anchor_position = Some(anchor);
        self
    }

    /// Snap to window edge instead of switching anchor corner when an overflow would occur.
    pub fn snap_to_window(mut self) -> Self {
        self.fit_mode = OverlayFitMode::SnapToWindow;
        self
    }
}

impl ParentElement for Overlay {
    fn children_mut(&mut self) -> &mut SmallVec<[AnyElement; 2]> {
        &mut self.children
    }
}

impl Element for Overlay {
    type State = OverlayState;

    fn request_layout(
        &mut self,
        _: Option<Self::State>,
        cx: &mut WindowContext,
    ) -> (crate::LayoutId, Self::State) {
        let child_layout_ids = self
            .children
            .iter_mut()
            .map(|child| child.layout(cx))
            .collect::<SmallVec<_>>();

        let mut overlay_style = Style::default();
        overlay_style.position = Position::Absolute;
        overlay_style.display = Display::Flex;

        let layout_id = cx.request_layout(&overlay_style, child_layout_ids.iter().copied());

        (layout_id, OverlayState { child_layout_ids })
    }

    fn paint(
        &mut self,
        bounds: crate::Bounds<crate::Pixels>,
        element_state: &mut Self::State,
        cx: &mut WindowContext,
    ) {
        if element_state.child_layout_ids.is_empty() {
            return;
        }

        let mut child_min = point(Pixels::MAX, Pixels::MAX);
        let mut child_max = Point::default();
        for child_layout_id in &element_state.child_layout_ids {
            let child_bounds = cx.layout_bounds(*child_layout_id);
            child_min = child_min.min(&child_bounds.origin);
            child_max = child_max.max(&child_bounds.lower_right());
        }
        let size: Size<Pixels> = (child_max - child_min).into();
        let origin = self.anchor_position.unwrap_or(bounds.origin);

        let mut desired = self.anchor_corner.get_bounds(origin, size);
        let limits = Bounds {
            origin: Point::default(),
            size: cx.viewport_size(),
        };

        if self.fit_mode == OverlayFitMode::SwitchAnchor {
            let mut anchor_corner = self.anchor_corner;

            if desired.left() < limits.left() || desired.right() > limits.right() {
                let switched = anchor_corner
                    .switch_axis(Axis::Horizontal)
                    .get_bounds(origin, size);
                if !(switched.left() < limits.left() || switched.right() > limits.right()) {
                    anchor_corner = anchor_corner.switch_axis(Axis::Horizontal);
                    desired = switched
                }
            }

            if desired.top() < limits.top() || desired.bottom() > limits.bottom() {
                let switched = anchor_corner
                    .switch_axis(Axis::Vertical)
                    .get_bounds(origin, size);
                if !(switched.top() < limits.top() || switched.bottom() > limits.bottom()) {
                    desired = switched;
                }
            }
        }

        // Snap the horizontal edges of the overlay to the horizontal edges of the window if
        // its horizontal bounds overflow, aligning to the left if it is wider than the limits.
        if desired.right() > limits.right() {
            desired.origin.x -= desired.right() - limits.right();
        }
        if desired.left() < limits.left() {
            desired.origin.x = limits.origin.x;
        }

        // Snap the vertical edges of the overlay to the vertical edges of the window if
        // its vertical bounds overflow, aligning to the top if it is taller than the limits.
        if desired.bottom() > limits.bottom() {
            desired.origin.y -= desired.bottom() - limits.bottom();
        }
        if desired.top() < limits.top() {
            desired.origin.y = limits.origin.y;
        }

        let offset = point(desired.origin.x.round(), desired.origin.y.round());
        cx.with_absolute_element_offset(offset, |cx| {
            cx.break_content_mask(|cx| {
                for child in &mut self.children {
                    child.paint(cx);
                }
            })
        })
    }
}

impl IntoElement for Overlay {
    type Element = Self;

    fn element_id(&self) -> Option<crate::ElementId> {
        None
    }

    fn into_element(self) -> Self::Element {
        self
    }
}

enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, PartialEq)]
pub enum OverlayFitMode {
    SnapToWindow,
    SwitchAnchor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnchorCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl AnchorCorner {
    fn get_bounds(&self, origin: Point<Pixels>, size: Size<Pixels>) -> Bounds<Pixels> {
        let origin = match self {
            Self::TopLeft => origin,
            Self::TopRight => Point {
                x: origin.x - size.width,
                y: origin.y,
            },
            Self::BottomLeft => Point {
                x: origin.x,
                y: origin.y - size.height,
            },
            Self::BottomRight => Point {
                x: origin.x - size.width,
                y: origin.y - size.height,
            },
        };

        Bounds { origin, size }
    }

    pub fn corner(&self, bounds: Bounds<Pixels>) -> Point<Pixels> {
        match self {
            Self::TopLeft => bounds.origin,
            Self::TopRight => bounds.upper_right(),
            Self::BottomLeft => bounds.lower_left(),
            Self::BottomRight => bounds.lower_right(),
        }
    }

    fn switch_axis(self, axis: Axis) -> Self {
        match axis {
            Axis::Vertical => match self {
                AnchorCorner::TopLeft => AnchorCorner::BottomLeft,
                AnchorCorner::TopRight => AnchorCorner::BottomRight,
                AnchorCorner::BottomLeft => AnchorCorner::TopLeft,
                AnchorCorner::BottomRight => AnchorCorner::TopRight,
            },
            Axis::Horizontal => match self {
                AnchorCorner::TopLeft => AnchorCorner::TopRight,
                AnchorCorner::TopRight => AnchorCorner::TopLeft,
                AnchorCorner::BottomLeft => AnchorCorner::BottomRight,
                AnchorCorner::BottomRight => AnchorCorner::BottomLeft,
            },
        }
    }
}
