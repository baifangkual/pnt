use super::event::{AppEvent, Event, EventHandler};
use ratatui::{DefaultTerminal, crossterm::event::{KeyCode, KeyEvent, KeyModifiers}, crossterm};
use crate::app::runtime::PntRuntimeContext;
use anyhow::Result;

/// TUI Application.
pub struct TUIRunning {
    /// Is the application running?
    pub running: bool,
    /// Counter.
    pub pnt: PntRuntimeContext,
    /// Event handler.
    pub events: EventHandler,
}

impl TUIRunning {
    pub fn with_pnt(pnt: PntRuntimeContext) -> Self {
        Self {
            running: true,
            pnt,
            events: EventHandler::new(),
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event) => self.handle_key_event(key_event)?,
                _ => {}
            },
            Event::App(app_event) => match app_event {
                AppEvent::Increment => {todo!("increment");},
                AppEvent::Decrement => {todo!("decrement");},
                AppEvent::Quit => self.quit(),
            },
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`TUIRunning`].
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => {
                self.events.send(AppEvent::Quit)
            }
            KeyCode::Right => self.events.send(AppEvent::Increment),
            KeyCode::Left => self.events.send(AppEvent::Decrement),
            // Other handlers you could add here.
            _ => {}
        }
        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&self) {}
    
    pub fn quit(&mut self) {
        self.running = false;
    }

    // todo impl check
    // pub fn increment_counter(&mut self) {
    //     self.counter = self.counter.saturating_add(1);
    // }
    // 
    // pub fn decrement_counter(&mut self) {
    //     self.counter = self.counter.saturating_sub(1);
    // }
}