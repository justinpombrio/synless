use geometry::*;

// Step 2: determine where the screen should be in doc coords

/// Position the screen on the document.
/// `screen_bound` is the size of the screen,
/// in terminal coordinates.
/// `cursor_region` is the region the cursor takes up,
/// in document coordinates.
/// `center` is the row the cursor should be centered at,
/// as a fraction of the screen height (0 is the top).
pub(in render) fn position_screen(
    screen_bound: Bound, cursor_region: Region, center: f32)
    -> Region
{
    // Number of rows from the top of the screen to the centerline.
    let centerline = (center * screen_bound.height as f32) as Row;
    let row =
        if centerline >= cursor_region.beginning().row {
            // We're at the very beginning of the document.
            0
        } else {
            cursor_region.beginning().row - centerline
        };
    Region{
        pos: Pos{
            row: row,
            col: 0
        },
        bound: screen_bound
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_position_screen() {
        let cursor_bound = Bound{ width: 2, height: 2, indent: 1};

        let screen_bound = Bound{ width: 5, height: 2, indent: 3 };
        let cursor_region = Region{
            pos: Pos{ row: 1, col: 0 },
            bound: cursor_bound
        };
        let center = 0.0;
        assert_eq!(position_screen(screen_bound, cursor_region, center),
                   Region{
                       pos: Pos{ row: 1, col: 0 },
                       bound: screen_bound
                   });

        let screen_bound = Bound{ width: 5, height: 2, indent: 3 };
        let cursor_region = Region{
            pos: Pos{ row: 1, col: 0 },
            bound: cursor_bound
        };
        let center = 0.5;
        assert_eq!(position_screen(screen_bound, cursor_region, center),
                   Region{
                       pos: Pos{ row: 0, col: 0 },
                       bound: screen_bound
                   });

        let screen_bound = Bound{ width: 5, height: 2, indent: 3 };
        let cursor_region = Region{
            pos: Pos{ row: 20, col: 20 },
            bound: cursor_bound
        };
        let center = 0.5;
        assert_eq!(position_screen(screen_bound, cursor_region, center),
                   Region{
                       pos: Pos{ row: 19, col: 0 },
                       bound: screen_bound
                   });
    }
}
