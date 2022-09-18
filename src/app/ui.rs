//! # ui
//!
//! `ui` implements methods to render the UI for the application in the terminal

use tui::{
    backend::Backend,
    Frame,
    layout::{
        Constraint,
        Direction,
        Layout,
    },
    widgets::{
        Block,
        Borders,
        List,
        ListItem,
        ListState,
        Paragraph,
        Wrap,
    },
    style::{
        Color,
        Modifier,
        Style,
    },
    // text::{
    //     Span,
    //     Spans,
    //     Text,
    // },
};

use super::{
    App,
    InputStatus,
    SelElement,
};

impl App {
    /// Draw the app UI
    pub fn draw_ui<B: Backend>(&mut self, frame: &mut Frame<B>) {
        // Holds the keyboard shortcuts span and request elements
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(100),
            ].as_ref())
            .split(frame.size());

        // Show keyboard shortcuts (too much screen clutter?)
        // let (msg, shortcut_style) = match self.input_status {
        //     InputStatus::NORMAL => (
        //         vec![
        //             Span::raw("Use the "),
        //             Span::styled("arrow keys ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("or "),
        //             Span::styled("h,j,k,l ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("to navigate. Press "),
        //             Span::styled("i ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("when hovering over request value to edit it. "),
        //             Span::raw("Press "),
        //             Span::styled("Enter ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("to execute selected request. "),
        //             Span::raw("Press "),
        //             Span::styled("n ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("to make a new request. "),
        //             Span::raw("Press "),
        //             Span::styled("x ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("to delete the selected request."),
        //         ],
        //         Style::default()
        //     ),
        //     InputStatus::INSERT => (
        //         vec![
        //             Span::raw("Press "),
        //             Span::styled("Enter ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("to update value. Press "),
        //             Span::styled("Esc ", Style::default()
        //                 .add_modifier(Modifier::BOLD)),
        //             Span::raw("to cancel editing."),
        //         ],
        //         Style::default().add_modifier(Modifier::RAPID_BLINK)
        //     )
        // };

        // let mut shortcut_help = Text::from(Spans::from(msg));
        // shortcut_help.patch_style(shortcut_style);
        // let shortcut_text = Paragraph::new(shortcut_help);
        // frame.render_widget(shortcut_text, main_layout[0]);

        // Holds the request list and request info panel
        let req_element_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ].as_ref())
            .split(main_layout[0]);

        // Render the list of elements to scroll through
        let list_block = Block::default()
            .title("Requests")
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White));

        let mut req_list_state = ListState::default();
        req_list_state.select(Some(self.req_index));

        let req_list: Vec<_> = self.requests
            .iter()
            .map(|req| ListItem::new(req.name.clone()))
            .collect();

        let req_list_widget = List::new(req_list)
            .block(list_block)
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Yellow)
            );

        frame.render_stateful_widget(req_list_widget, req_element_layout[0],
            &mut req_list_state);

        // Render the info for the selected request
        let info_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(40),
            ].as_ref())
            .split(req_element_layout[1]);

        // Render input boxes
        let norm_style = Style::default().fg(Color::White);
        let sel_style = Style::default().fg(Color::White).bg(Color::Yellow);
        for i in 0..self.inputs.len() {
            let input_box = Paragraph::new(self.inputs[i].value())
                .block(Block::default().title(
                    match i {
                        0 => "Name",
                        1 => "Request Type",
                        2 => "Url",
                        3 => "Body",
                        _ => "",
                    }
                ).borders(Borders::ALL))
                .style(
                    if (i == self.input_index) &&
                    (self.sel_element == SelElement::INFOPANEL) {
                        sel_style
                    } else {
                        norm_style
                    }
                )
                .wrap(Wrap {trim: false});

            frame.render_widget(input_box, info_layout[i]);
        }

        // Render cursor
        if self.input_status == InputStatus::INSERT {
            let sel_layout = &info_layout[self.input_index];
            let sel_input = &self.inputs[self.input_index];

            frame.set_cursor(
                sel_layout.x + (sel_input.cursor() as u16) + 1,
                sel_layout.y + (sel_input.value().len() as u16 / sel_layout.width) + 1
            );
        }

        // Render response
        let (response_text, status_text) = match self.requests
            .get(self.req_index) {

            Some(cur_req) => (cur_req.resp.clone(), cur_req.status.clone()),
            None => (String::new(), String::new()),
        };

        let response_code_box = Paragraph::new(status_text)
            .block(Block::default().title("Status Code").borders(Borders::ALL))
            .style(norm_style);
        frame.render_widget(
            response_code_box, info_layout[4]
        );

        let response_text_box = Paragraph::new(response_text)
            .block(Block::default().title("Response").borders(Borders::ALL))
            .style(norm_style);
        frame.render_widget(
            response_text_box.scroll(self.text_offset), info_layout[5]
        );
    }
}
