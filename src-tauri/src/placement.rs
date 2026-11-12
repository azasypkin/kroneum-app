//! Menu-bar panel coordinates, in macOS logical points.

/// An AppKit rectangle: global points, bottom-left origin, y increasing upwards.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Fit the whole panel inside the tray screen's visible frame, excluding menu/Dock.
pub fn below_tray(tray: Rect, visible: Rect, requested_size: (f64, f64)) -> Rect {
    let width = requested_size.0.min((visible.width - 16.0).max(1.0));
    let height = requested_size.1.min((visible.height - 16.0).max(1.0));
    let min_y = visible.y + 8.0;
    let max_y = (visible.y + visible.height - height - 8.0).max(min_y);
    Rect {
        x: horizontal_origin(tray.x + tray.width / 2.0, width, visible.x, visible.width),
        y: (tray.y - height - 8.0).clamp(min_y, max_y),
        width,
        height,
    }
}

pub fn horizontal_origin(
    center: f64,
    panel_width: f64,
    screen_left: f64,
    screen_width: f64,
) -> f64 {
    let left = screen_left + 8.0;
    let right = (screen_left + screen_width - panel_width - 8.0).max(left);
    (center - panel_width / 2.0).clamp(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_stays_inside_the_right_edge() {
        assert_eq!(horizontal_origin(1430.0, 392.0, 0.0, 1440.0), 1040.0);
    }

    #[test]
    fn panel_stays_inside_the_left_edge() {
        assert_eq!(horizontal_origin(10.0, 392.0, 0.0, 1440.0), 8.0);
    }

    #[test]
    fn display_left_of_the_primary_display_uses_negative_coordinates() {
        assert_eq!(horizontal_origin(-20.0, 392.0, -1920.0, 1920.0), -400.0);
    }

    #[test]
    fn a_narrow_display_does_not_panic() {
        assert_eq!(horizontal_origin(100.0, 392.0, 0.0, 300.0), 8.0);
    }

    #[test]
    fn panel_fits_fully_on_a_secondary_screen_in_global_points() {
        let tray = Rect {
            x: 3960.0,
            y: 1416.0,
            width: 40.0,
            height: 24.0,
        };
        let visible = Rect {
            x: 1440.0,
            y: 0.0,
            width: 2560.0,
            height: 1416.0,
        };
        assert_eq!(
            below_tray(tray, visible, (392.0, 684.0)),
            Rect {
                x: 3600.0,
                y: 724.0,
                width: 392.0,
                height: 684.0
            }
        );
    }

    #[test]
    fn negative_screen_origins_do_not_move_the_panel_to_the_primary_screen() {
        let tray = Rect {
            x: -40.0,
            y: 776.0,
            width: 32.0,
            height: 24.0,
        };
        let visible = Rect {
            x: -1280.0,
            y: 0.0,
            width: 1280.0,
            height: 776.0,
        };
        let result = below_tray(tray, visible, (392.0, 684.0));
        assert_eq!(result.x, -400.0);
        assert_eq!(result.y, 84.0);
    }

    #[test]
    fn a_screen_above_the_primary_screen_keeps_its_vertical_origin() {
        let tray = Rect {
            x: 1300.0,
            y: 1776.0,
            width: 40.0,
            height: 24.0,
        };
        let visible = Rect {
            x: 0.0,
            y: 900.0,
            width: 1440.0,
            height: 876.0,
        };
        let result = below_tray(tray, visible, (392.0, 684.0));
        assert_eq!(result.y, 1084.0);
        assert!(result.y >= visible.y && result.y + result.height <= visible.y + visible.height);
    }

    #[test]
    fn small_visible_frames_resize_the_panel_instead_of_clipping_it() {
        let tray = Rect {
            x: 580.0,
            y: 476.0,
            width: 40.0,
            height: 24.0,
        };
        let visible = Rect {
            x: 0.0,
            y: 40.0,
            width: 360.0,
            height: 436.0,
        };
        assert_eq!(
            below_tray(tray, visible, (392.0, 684.0)),
            Rect {
                x: 8.0,
                y: 48.0,
                width: 344.0,
                height: 420.0
            }
        );
    }

    #[test]
    fn the_panel_stays_above_the_dock() {
        let tray = Rect {
            x: 800.0,
            y: 760.0,
            width: 40.0,
            height: 24.0,
        };
        let visible = Rect {
            x: 0.0,
            y: 80.0,
            width: 1280.0,
            height: 680.0,
        };
        let result = below_tray(tray, visible, (392.0, 684.0));
        assert_eq!(result.y, 88.0);
        assert_eq!(result.height, 664.0);
    }
}
