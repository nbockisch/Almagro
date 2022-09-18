//! # tui
//!
//! Contains the methods to initialize and destruct the terminal interface used
//! to render the app's UI, and provides a method to draw the app's UI

use crossterm::terminal::{
    self,
    EnterAlternateScreen,
    LeaveAlternateScreen,
};
use std::io;
use tui::{
    backend::Backend,
    Terminal
};

use crate::app::App;

/// Representation of the terminal UI
///
/// Holds the metadata and methods for rendering the App UI to the terminal
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
}

impl<B: Backend> Tui<B> {
    /// Constructs a new Tui object
    ///
    /// Takes in a Terminal
    pub fn new(terminal: Terminal<B>) -> Self {
        Self { terminal }
    }

    /// Prepares the Tui's Terminal object for displaying the UI
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        Ok(())
    }

    /// Clean up the terminal on application exit
    pub fn exit(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;

        Ok(())
    }

    /// Draw the app UI
    pub fn draw(&mut self, app: &mut App) -> Result<(), Box<dyn std::error::Error>>{
        self.terminal.draw(|frame| app.draw_ui(frame))?;
        Ok(())
    }
}
