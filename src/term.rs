use console::{Key, Term};
use dialoguer::theme::{SelectionStyle, Theme};
use std::fmt;
use std::io;
use std::iter::repeat;
use std::ops::Rem;

/// Renders the terminal
pub struct Terminal<'a> {
    items: Vec<String>,
    clear: bool,
    theme: &'a Theme,
    capacity: usize,
}

impl<'a> Terminal<'a> {
    /// Creates a new terminal item with a theme
    pub fn new(theme: &'a Theme, capacity: usize) -> Terminal<'a> {
        Terminal {
            items: vec![],
            clear: true,
            theme,
            capacity,
        }
    }

    /// Adds multiple items to the Terminal options
    pub fn items(&mut self, items: &[&str]) -> &mut Terminal<'a> {
        for item in items {
            self.items.push(item.to_string());
        }
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// The user can select the items with the space bar and on enter
    /// the selected items will be returned.
    pub fn interact(&self) -> io::Result<Vec<usize>> {
        self.interact_on(&Term::stderr())
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<Vec<usize>> {
        // Define the first page and the number of pages
        let mut page: usize = 0;
        let pages: usize = (self.items.len() / self.capacity) + 1;

        // Define the theme renderer
        let mut renderer = TerminalRenderer::new(term, self.theme);

        // Render the prompt
        renderer.prompt("Select the technologies for `.gitignore` using `space`")?;
        renderer.prompt("Search pattern")?;

        // Define the current selection
        let mut sel = 0;

        // Make a vector with the size of every item to be displayed
        let mut size_vec = Vec::new();
        for item in self.items.iter().as_slice() {
            let size = item.len();
            size_vec.push(size);
        }

        // Make a vector to remember which value was checked
        let mut checked: Vec<_> = repeat(false).take(self.items.len()).collect();

        loop {
            // Render the items
            for (idx, item) in self
                .items
                .iter()
                .enumerate()
                .skip(page * self.capacity)
                .take(self.capacity)
            {
                renderer.selection(
                    item,
                    match (checked[idx], sel == idx) {
                        (true, true) => SelectionStyle::CheckboxCheckedSelected,
                        (true, false) => SelectionStyle::CheckboxCheckedUnselected,
                        (false, true) => SelectionStyle::CheckboxUncheckedSelected,
                        (false, false) => SelectionStyle::CheckboxUncheckedUnselected,
                    },
                )?;
            }
            // Handle key presses
            match term.read_key()? {
                Key::ArrowDown => {
                    if sel == !0 {
                        sel = 0;
                    } else {
                        sel = (sel as u64 + 1).rem(self.items.len() as u64) as usize;
                    }
                }
                Key::ArrowUp => {
                    if sel == !0 {
                        sel = self.items.len() - 1;
                    } else {
                        sel = ((sel as i64 - 1 + self.items.len() as i64)
                            % (self.items.len() as i64)) as usize;
                    }
                }
                Key::ArrowLeft => {
                    if page == 0 {
                        page = pages - 1;
                    } else {
                        page = page - 1;
                    }
                    sel = page * self.capacity;
                }
                Key::ArrowRight => {
                    if page == pages - 1 {
                        page = 0;
                    } else {
                        page = page + 1;
                    }
                    sel = page * self.capacity;
                }
                Key::Char(' ') => {
                    checked[sel] = !checked[sel];
                }
                Key::Escape => {
                    if self.clear {
                        renderer.clear()?;
                    }
                    return Ok(vec![]);
                }
                Key::Enter => {
                    if self.clear {
                        renderer.clear()?;
                    }
                    return Ok(checked
                        .into_iter()
                        .enumerate()
                        .filter_map(|(idx, checked)| if checked { Some(idx) } else { None })
                        .collect());
                }
                _ => {}
            }

            // Update page if needed
            if sel < page * self.capacity || sel > (page + 1) * self.capacity {
                page = sel / self.capacity;
            }
            renderer.clear_preserve_prompt(&size_vec)?;
        }
    }
}

/// Helper struct to render a terminal.
pub(crate) struct TerminalRenderer<'a> {
    term: &'a Term,
    theme: &'a Theme,
    height: usize,
    prompt_height: usize,
    prompts_reset_height: bool,
}

impl<'a> TerminalRenderer<'a> {
    pub fn new(term: &'a Term, theme: &'a Theme) -> TerminalRenderer<'a> {
        TerminalRenderer {
            term,
            theme,
            height: 0,
            prompt_height: 0,
            prompts_reset_height: true,
        }
    }

    fn write_formatted_line<F: FnOnce(&mut TerminalRenderer, &mut fmt::Write) -> fmt::Result>(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        let mut buf = String::new();
        f(self, &mut buf).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        self.height += buf.chars().filter(|&x| x == '\n').count() + 1;
        self.term.write_line(&buf)
    }

    fn write_formatted_prompt<F: FnOnce(&mut TerminalRenderer, &mut fmt::Write) -> fmt::Result>(
        &mut self,
        f: F,
    ) -> io::Result<()> {
        self.write_formatted_line(f)?;
        if self.prompts_reset_height {
            self.prompt_height = self.height;
            self.height = 0;
        }
        Ok(())
    }

    pub fn prompt(&mut self, prompt: &str) -> io::Result<()> {
        self.write_formatted_prompt(|this, buf| this.theme.format_prompt(buf, prompt))
    }

    pub fn selection(&mut self, text: &str, style: SelectionStyle) -> io::Result<()> {
        self.write_formatted_line(|this, buf| this.theme.format_selection(buf, text, style))
    }

    pub fn clear(&mut self) -> io::Result<()> {
        self.term
            .clear_last_lines(self.height + self.prompt_height)?;
        self.height = 0;
        Ok(())
    }

    pub fn clear_preserve_prompt(&mut self, size_vec: &Vec<usize>) -> io::Result<()> {
        let mut new_height = self.height;
        //Check each item size, increment on finding an overflow
        for size in size_vec {
            if *size > self.term.size().1 as usize {
                new_height += 1;
            }
        }
        self.term.clear_last_lines(new_height)?;
        self.height = 0;
        Ok(())
    }
}
