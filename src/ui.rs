pub mod running;

use std::sync::Arc;

use ratatui::
    Frame
;
use tokio::sync::RwLock;

use crate::app::App;

/// Renders the user interface widgets.
pub fn render(app: Arc<RwLock<App>>, frame: &mut Frame) {
    if let Ok(app) = app.try_read(){
        app.draw(frame);
    }
}
