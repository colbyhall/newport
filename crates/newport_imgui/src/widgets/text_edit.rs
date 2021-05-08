use crate::{
    Builder,
    Id,
    ToId,
    ColorStyle,
    TextStyle,
    LayoutStyle,
    Shape,

    Retained,

    math,
    os,
};

use math::{
    Rect,
    Vector2,
};

pub struct TextEdit<'a> {
    text: &'a mut String,
    hint: String,
    id:   Id,
}

impl<'a> TextEdit<'a> {
    pub fn singleline(text: &'a mut String) -> Self {
        let id = (text as *mut String).to_id();
        Self {
            text,
            hint: String::new(),
            id,
        }
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = hint.into();
        self
    }
}

#[derive(Copy, Clone)]
pub enum TextEditResponse {
    None,
    Hovered,
    InputRecieved,
    Entered,
}

impl TextEditResponse {
    pub fn entered(self) -> bool {
        match self {
            TextEditResponse::Entered => true,
            _ => false,
        }
    }

    pub fn input_recieved(self) -> bool {
        match self {
            TextEditResponse::Entered => true,
            TextEditResponse::InputRecieved => true,
            _ => false,
        }
    }

    pub fn hovered(self) -> bool {
        match self {
            TextEditResponse::Hovered => true,
            _ => false,
        }
    }
}

#[derive(Default, Clone)]
struct TextEditRetained {
    cursor:    usize,
    selection: usize,
    cursor_t:  f32,
    showing_cursor: bool,
}

impl TextEditRetained {
    fn reset_cursor_blink(&mut self) {
        self.cursor_t = 0.0;
        self.showing_cursor = true;
    }

    fn seek(&self, left: bool, text: &str) -> usize {
        if left {
            for i in (0..self.cursor - 1).rev() {
                let c = text.chars().nth(i).unwrap();
                if c == ' ' || c == '.' || c == '/' || c == '{' || c == '}' || c == '(' || c == ')' || c == '[' || c == ']' {
                    return i + 1;
                }
            }
            return 0;
        } else {
            for i in self.cursor + 1..text.len() {
                let c = text.chars().nth(i).unwrap();
                if c == ' ' || c == '.' || c == '/' || c == '{' || c == '}' || c == '(' || c == ')' || c == '[' || c == ']' {
                    return i;
                }
            }
            return text.len();
        }
    }

    fn remove_selection(&mut self, text: &mut String) {
        if self.cursor != self.selection {
            if self.cursor > self.selection {
                text.replace_range(self.selection..self.cursor, "");
                self.cursor = self.selection;
            } else {
                text.replace_range(self.cursor..self.selection, "");
                self.selection = self.cursor;
            }
        }
    }
}

impl Retained for TextEditRetained { }

impl <'a> TextEdit<'a> {
    pub fn build(self, builder: &mut Builder) -> TextEditResponse{
        let text: TextStyle = builder.style().get();
        
        let label_size = text.string_rect(&self.text, text.label_size, None).size();
        let bounds = builder.content_bounds(label_size);
        
        let mut retained: TextEditRetained = builder.retained(self.id);

        let layout_style: LayoutStyle = builder.style().get();
        let at = Vector2::new(bounds.min.x + layout_style.padding.min.x, Rect::from_pos_size(bounds.pos(), label_size).top_left().y);
        let mut font = text.font.write();
        let font = font.font_at_size(text.label_size, builder.input().dpi).unwrap();

        let is_over = builder.input().mouse_is_over(bounds);
        if is_over {
            builder.hover(self.id);

            if builder.input().was_primary_clicked() {
                builder.focus(self.id);
                retained.cursor_t = 0.0;
                retained.showing_cursor = true;
            }
        } else {
            if builder.is_focused(self.id) && builder.input().was_primary_clicked() {
                builder.unfocus(self.id);
            }
            builder.unhover(self.id);
        }

        let text_len = self.text.len();
        if retained.cursor > text_len || retained.selection > text_len {
            retained.cursor = 0;
            retained.selection = 0;
        }

        let is_focused = builder.is_focused(self.id);
        let is_hovered = builder.is_hovered(self.id);
        let response = if is_focused {
            retained.cursor_t += builder.input().dt;
            if retained.cursor_t > os::caret_blink_time() {
                retained.cursor_t = 0.0;
                retained.showing_cursor = !retained.showing_cursor;
            }

            let mut response = TextEditResponse::None;

            let input = &builder.input().text_input;
            if input.len() > 0 {
                response = TextEditResponse::InputRecieved;
                retained.reset_cursor_blink();
                retained.remove_selection(self.text);
                self.text.insert_str(retained.cursor, &builder.input().text_input);

                let input_len = builder.input().text_input.len();
                retained.cursor += input_len;
                retained.selection += input_len;
            }

            let shift_down = builder.input().key_down[os::input::KEY_SHIFT.as_key().0 as usize];
            let ctrl_down = builder.input().key_down[os::input::KEY_CTRL.as_key().0 as usize];

            if builder.input().was_key_pressed(os::input::KEY_LEFT) {
                response = TextEditResponse::InputRecieved;

                if retained.cursor != retained.selection {
                    if shift_down && retained.cursor > 0 {
                        if ctrl_down {
                            retained.cursor = retained.seek(true, &self.text);
                        } else {
                            retained.cursor -= 1;
                        }
                    } else {
                        retained.selection = retained.cursor;
                    }
                } else if retained.cursor > 0 {
                    if ctrl_down {
                        retained.cursor = retained.seek(true, &self.text);
                    } else {
                        retained.cursor -= 1;
                    }
                    
                    if !shift_down {
                        retained.selection = retained.cursor;
                    }
                }
                retained.reset_cursor_blink();
            }

            if builder.input().was_key_pressed(os::input::KEY_RIGHT) {
                response = TextEditResponse::InputRecieved;

                if retained.cursor != retained.selection {
                    if shift_down && retained.cursor < self.text.len() {
                        if ctrl_down {
                            retained.cursor = retained.seek(false, &self.text);
                        } else {
                            retained.cursor += 1;
                        }
                    } else {
                        retained.selection = retained.cursor;
                    }
                } else if retained.cursor < self.text.len() {
                    if ctrl_down {
                        retained.cursor = retained.seek(false, &self.text);
                    } else {
                        retained.cursor += 1;
                    }
                    
                    if !shift_down {
                        retained.selection = retained.cursor;
                    }
                }
                retained.reset_cursor_blink();
            }

            if builder.input().was_key_pressed(os::input::KEY_BACKSPACE) {
                response = TextEditResponse::InputRecieved;

                if retained.cursor != retained.selection {
                    retained.remove_selection(self.text);
                } else {
                    if retained.cursor > 0 {
                        if ctrl_down {
                            retained.cursor = retained.seek(true, self.text);
                            retained.remove_selection(self.text);
                        } else {
                            retained.cursor -= 1;
                            retained.selection = retained.cursor;
                            self.text.remove(retained.cursor);
                        }
                    }
                }
                retained.reset_cursor_blink();
            }

            if builder.input().was_key_pressed(os::input::KEY_DELETE) {
                response = TextEditResponse::InputRecieved;

                if retained.cursor != retained.selection {
                    retained.remove_selection(self.text);
                } else {
                    if retained.cursor < self.text.len() {
                        if ctrl_down {
                            retained.cursor = retained.seek(false, self.text);
                            retained.remove_selection(self.text);
                        } else {
                            self.text.remove(retained.cursor);
                            retained.selection = retained.cursor;
                        }
                    }

                }
            }

            if builder.input().was_key_pressed(os::input::KEY_HOME) {
                response = TextEditResponse::InputRecieved;
                
                retained.cursor = 0;
                if !shift_down {
                    retained.selection = retained.cursor;
                }
            }

            if builder.input().was_key_pressed(os::input::KEY_END) {
                response = TextEditResponse::InputRecieved;

                retained.cursor = self.text.len();
                if !shift_down {
                    retained.selection = retained.cursor;
                }
            }

            if builder.input().was_key_pressed(os::input::KEY_ENTER) { 
                response = TextEditResponse::Entered;
                builder.unfocus(self.id);
            }

            if builder.input().was_primary_clicked() {
                for (index, bounds) in font.bounds_iter(self.text, at).enumerate() {
                    if builder.input().mouse_is_over(bounds) {
                        retained.cursor = index;
                        retained.selection = index;
                        response = TextEditResponse::InputRecieved;
                    }
                }

                if builder.input().mouse_is_over(bounds) && builder.input().mouse_location.unwrap_or_default().x > at.x + label_size.x {
                    retained.cursor = self.text.len();
                    retained.selection = retained.cursor;
                    response = TextEditResponse::InputRecieved;
                }
            }

            if builder.input().mouse_button_down[os::input::MOUSE_BUTTON_LEFT.as_mouse_button() as usize] {
                for (index, bounds) in font.bounds_iter(self.text, at).enumerate() {
                    if builder.input().mouse_is_over(bounds) {
                        retained.cursor = index;
                        response = TextEditResponse::InputRecieved;
                    }
                }

                if builder.input().mouse_is_over(bounds) && builder.input().mouse_location.unwrap_or_default().x > at.x + label_size.x {
                    retained.cursor = self.text.len();
                    response = TextEditResponse::InputRecieved;
                }
            }
            
            if response.input_recieved() {
                retained.reset_cursor_blink();
            }

            response
        } else {
            TextEditResponse::None
        }; 

        let color: ColorStyle = builder.style().get();
        let (background_color, foreground_color) = {
            let background_color = if is_focused {
                color.focused_background
            } else if is_hovered {
                color.hovered_background
            } else {
                color.unhovered_background
            };

            let foreground_color = if is_focused {
                color.focused_foreground
            } else if is_hovered {
                color.hovered_foreground
            } else {
                color.unhovered_foreground
            };

            (background_color, foreground_color)
        };

        builder.painter.push_shape(Shape::solid_rect(bounds, background_color, 0.0));

        let cursor_width = 3.0;

        if self.text.len() > 0 {
            // Show the blinking caret if we're focused
            if is_focused {
                builder.painter.push_shape(
                    Shape::text(
                        self.text.clone(), 
                        at, 
                        &text.font, 
                        text.label_size, 
                        builder.input().dpi, 
                        foreground_color
                    )
                );

                // Change the color of the cursor when we have a selection
                let cursor_color = if retained.cursor != retained.selection {
                    color.selected_foreground
                } else {
                    foreground_color
                };

                // Draw the cursor on the end if its not in the middle of the text
                if retained.cursor == self.text.len() && retained.showing_cursor {
                    let at = at + Vector2::new(label_size.x, -font.height);
                    let bounds = (at, at + Vector2::new(cursor_width, font.height));
                    builder.painter.push_shape(Shape::solid_rect(bounds, cursor_color, 0.0));
                } 
                
                // Iterate through the text if the cursor is in the middle or we have a valid selection
                if retained.cursor < self.text.len() || retained.cursor != retained.selection {
                    for (index, bounds) in font.bounds_iter(self.text, at).enumerate() {
                        if retained.cursor != retained.selection {
                            let show_selection = if retained.cursor > retained.selection {
                                index >= retained.selection && index < retained.cursor
                            } else {
                                index >= retained.cursor && index < retained.selection
                            };

                            if show_selection {
                                builder.painter.push_shape(Shape::solid_rect(bounds, color.selected_background, 0.0));

                                let c = self.text.chars().nth(index).unwrap();
                                builder.painter.push_shape(Shape::text(
                                    format!("{}", c), 
                                    bounds.top_left(), 
                                    &text.font, 
                                    text.label_size, 
                                    builder.input().dpi, 
                                    color.selected_foreground
                                ));
                            }
                        }

                        if index == retained.cursor && retained.showing_cursor {
                            let bounds = (bounds.min, bounds.min + Vector2::new(cursor_width, bounds.height()));
                            builder.painter.push_shape(Shape::solid_rect(bounds, cursor_color, 0.0));
                        }
                    }
                }
            } else {
                builder.painter.push_shape(
                    Shape::text(
                        self.text.clone(), 
                        at, 
                        &text.font, 
                        text.label_size, 
                        builder.input().dpi, 
                        foreground_color
                    )
                );
            }
        } else {
            if is_focused {
                if retained.showing_cursor {
                    let bounds = (at - Vector2::new(0.0, font.height), at + Vector2::new(cursor_width, 0.0));
                    builder.painter.push_shape(Shape::solid_rect(bounds, foreground_color, 0.0));
                }
            } else {
                builder.painter.push_shape(
                    Shape::text(
                        self.hint, 
                        at, 
                        &text.font, 
                        text.label_size, 
                        builder.input().dpi, 
                        foreground_color
                    )
                );
            }
        }

        builder.set_retained(self.id, retained);

        response
    }
}