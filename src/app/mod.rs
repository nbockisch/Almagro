//! # app
//!
//! The `app` crate provides the struct with data for the app and methods to
//! run its functionality.

use std::{
    collections::BTreeMap,
    fs,
};

use crossterm::event::{
    KeyCode,
    KeyEvent,
    Event, KeyModifiers,
};
use home;
use jfs::Store;
use reqwest::Method;
use tui_input::{
    backend::crossterm as input_backend,
    Input
};

use self::request::{
    REQ_FIELD_COUNT,
    Request,
};

pub mod request;
pub mod ui;

/// Are we inserting text or navigating the UI?
#[derive(PartialEq)]
enum InputStatus {
    INSERT,
    NORMAL,
}

/// Are we navigating the request list or the info panel
#[derive(PartialEq)]
pub enum SelElement {
    LIST,
    INFOPANEL,
}

/// Contains the data and methods to run the app
pub struct App {
    pub is_running: bool,
    // Index of the current selected request
    pub req_index: usize,
    pub db: Store,
    pub requests: Vec<Request>,
    // Inputs to change request info
    pub inputs: Vec<Input>,
    pub input_index: usize,
    // Offset for selected text box
    pub text_offset: (u16, u16),
    input_status: InputStatus,
    sel_element: SelElement,
}

impl App {
    /// Construct a new App object
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut db_config = jfs::Config::default();
        db_config.single = true;


        // Create/get path to saved requests database
        let home_dir = home::home_dir()
            .ok_or("Couldn't get user home directory")?;
        fs::create_dir_all(home_dir.join(".almagro"))?;
        let data_path = home_dir.join(".almagro").join("data");

        let mut app = Self {
            is_running: true,
            req_index: 0,
            db: Store::new_with_cfg(data_path, db_config).unwrap(),
            requests: Vec::new(),
            input_status: InputStatus::NORMAL,
            sel_element: SelElement::LIST,
            inputs: vec!["".into(); REQ_FIELD_COUNT],
            input_index: 0,
            text_offset: (0, 0),
        };

        // Load all requests from the database
        let req_btree: BTreeMap<String, Request> = app.db.all()?;
        for (_, req) in req_btree {
            app.requests.push(req);
        }

        app.update_inputs();

        return Ok(app);
    }

    /// Handle key events in insert mode
    fn insert_mode_keys(&mut self, modifiers: KeyModifiers, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.input_status = InputStatus::NORMAL;
                self.update_inputs();
            },
            KeyCode::Enter => {
                if self.requests.len() > 0 {
                    self.update_req_info();
                    self.save_current_request();
                    self.input_status = InputStatus::NORMAL;
                }
            },
            _ => {
                input_backend::to_input_request(Event::Key(KeyEvent {
                    modifiers,
                    code,
                }))
                    .and_then(|req| self.inputs[self.input_index]
                        .handle(req));
            },
        }
    }

    /// Handle key events in normal mode
    fn normal_mode_keys(&mut self, _modifiers: KeyModifiers, code: KeyCode) {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => self.is_running = false,
            KeyCode::Char('j') => {
                match self.sel_element {
                    SelElement::LIST => {
                        self.req_index = (self.req_index + 1)
                            % self.requests.len();

                        self.update_inputs();
                    },
                    SelElement::INFOPANEL => {
                        self.text_offset = (0, 0);
                        self.input_index = (self.input_index + 1)
                            % self.inputs.len();
                    }
                }
            },
            KeyCode::Down => {
                if self.sel_element == SelElement::INFOPANEL {
                    // Scroll text
                    self.text_offset.0 += 1;
                }
            },
            // Allow swapping with next request in list
            KeyCode::Char('J') => {
                if self.sel_element == SelElement::LIST 
                    && self.req_index < self.requests.len() - 1 {

                    self.requests
                        .swap(self.req_index, self.req_index + 1);
                    self.req_index += 1;
                }
            },
            KeyCode::Char('k') => {
                match self.sel_element {
                    SelElement::LIST => {
                        if self.req_index == 0 {
                            self.req_index = self.requests.len() - 1;
                            return;
                        }
                        self.req_index -= 1;

                        self.update_inputs();
                    },
                    SelElement::INFOPANEL => {
                        self.text_offset = (0, 0);
                        if self.input_index == 0 {
                            self.input_index = self.inputs.len() - 1;
                            return;
                        }
                        self.input_index -= 1;
                    }
                }
            },
            // Scroll text
            KeyCode::Up => {
                if self.sel_element == SelElement::INFOPANEL {
                    if self.text_offset.0 > 0 {
                        self.text_offset.0 -= 1;
                    }
                }
            },
            // Allow swapping with previous request in list
            KeyCode::Char('K') => {
                if self.sel_element == SelElement::LIST
                    && self.req_index > 0 {

                    self.requests
                        .swap(self.req_index, self.req_index - 1);
                    self.req_index -= 1;
                }
            },
            KeyCode::Char('i') => {
                self.input_status = InputStatus::INSERT;
                self.sel_element = SelElement::INFOPANEL;
            },
            // Select list or info panel
            KeyCode::Char('h') | KeyCode::Char('l') => self.sel_element
                    = match self.sel_element {
                    SelElement::LIST => SelElement::INFOPANEL,
                    SelElement::INFOPANEL => {
                            self.input_index = 0;
                            self.text_offset = (0, 0);
                            SelElement::LIST
                    },
            },
            KeyCode::Left => if self.text_offset.1 > 0 {
                self.text_offset.1 -= 1;
            },
            KeyCode::Right => self.text_offset.1 += 1,
            KeyCode::Enter => if self.requests.len() > 0 {
                self.requests[self.req_index].run_req();
            },
            KeyCode::Char('n') => {
                // Create a new request
                let new_req = Request::new(format!("Request #{}",
                    self.requests.len() + 1));

                self.requests.push(new_req);
                self.req_index = self.requests.len() - 1;
                self.update_inputs();

                // Save the new request
                self.save_current_request();
            },
            KeyCode::Char('x') => {
                // Delete a request
                if self.requests.len() > 0 {
                    let cur_id = &self.requests[self.req_index].db_id;
                    // Delete the request from the database
                    self.db.delete(&cur_id)
                        .expect("Couldn't delete request from the database");
                    self.requests.remove(self.req_index);
                    self.update_inputs();

                    if self.req_index > 0 { self.req_index -= 1 }
                }
            }
            _ => (),
        }
    }

    /// Handle key events in the app
    pub fn handle_keys(&mut self, event: Event) {
        match event {
            Event::Key(KeyEvent {
                modifiers,
                code,
                ..
            }) => match self.input_status {
                    InputStatus::INSERT =>
                        self.insert_mode_keys(modifiers, code),
                    InputStatus::NORMAL =>
                        self.normal_mode_keys(modifiers, code),
                },
            _ => (),
        };
    }

    /// Update the text input boxes to hold the values of the current request
    fn update_inputs(&mut self) {
        if let Some(cur_req) = self.requests.get(self.req_index) {
            (self.inputs[0], self.inputs[1], self.inputs[2], self.inputs[3])
                = (cur_req.name.clone().into(),
                    cur_req.req_type.as_str().into(),
                    cur_req.url.clone().into(),
                    cur_req.body.clone().into());
        } else {
            for input in self.inputs.iter_mut() {
                input.reset();
            }
        }
    }

    /// Update data in a request with input field data in the UI
    fn update_req_info(&mut self) {
        let input_text = self.inputs[self.input_index].value();

        match self.input_index {
            0 => self.requests[self.req_index].name = input_text.to_string(),
            1 => if let Ok(req_type) = Method::from_bytes(input_text.as_bytes()) {
                self.requests[self.req_index].req_type = req_type.to_string();
                self.update_inputs();
            } else {
                self.update_inputs();
            },
            2 => self.requests[self.req_index].url = input_text.to_string(),
            3 => self.requests[self.req_index].body = input_text.to_string(),
            _ => (),
        };
    }

    /// Save the currently selected request to the database
    fn save_current_request(&mut self) {
        let mut cur_req = &mut self.requests[self.req_index];

        // With this library, you must save once to get the ID, then I store
        // that ID with the request and save it again to preserve it in the
        // database
        if cur_req.db_id.is_empty() {
            cur_req.db_id = self.db.save(cur_req)
                .expect("Couldn't save request to database");
        }

        self.db.save_with_id(cur_req, &cur_req.db_id)
            .expect("Couldn't save request to database");
    }
}
