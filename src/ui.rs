pub mod running;

use ratatui::
    Frame
;

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: &mut App, frame: &mut Frame) {

    app.draw(frame);

}
