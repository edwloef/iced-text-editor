use iced::{
    executor, highlighter, keyboard, theme, widget, Application, Command, Element, Settings,
    Subscription, Theme,
};

use std::{env, fs, path::PathBuf};

fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: iced::font::Font::with_name("Noto Sans Mono"),
        ..Settings::default()
    })
}

struct Editor {
    path: Option<PathBuf>,
    content: widget::text_editor::Content,
    dirty: bool,
}

#[derive(Debug, Clone)]
enum Message {
    EditorAction(widget::text_editor::Action),
    OpenFile,
    NewFile,
    SaveFile,
}

impl Application for Editor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                path: None,
                content: widget::text_editor::Content::new(),
                dirty: true,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Iced Text Editor".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EditorAction(action) => {
                if action.is_edit() {
                    self.dirty = true;
                }

                self.content.perform(action);
            }
            Message::OpenFile => {
                let (path, contents): (Option<PathBuf>, String) = open_file();

                if path.is_some() {
                    self.path = path;
                    self.content = widget::text_editor::Content::with_text(contents.as_str());
                    self.dirty = false;
                }
            }
            Message::NewFile => {
                self.path = None;
                self.content = widget::text_editor::Content::new();
            }
            Message::SaveFile => {
                if self.path.is_none() {
                    self.path = save_new_file();
                }

                if self.path.is_some() {
                    fs::write(self.path.as_ref().unwrap(), self.content.text()).ok();
                    self.dirty = false;
                }
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key_code, modifiers| match key_code {
            keyboard::Key::Character(_s) if modifiers.command() => Some(Message::SaveFile),
            _ => None,
        })
    }

    fn view(&self) -> Element<'_, Message> {
        let input: Element<'_, Message> = widget::text_editor(&self.content)
            .on_action(|action: widget::text_editor::Action| -> Message {
                Message::EditorAction(action)
            })
            .highlight::<highlighter::Highlighter>(
                highlighter::Settings {
                    theme: highlighter::Theme::SolarizedDark,
                    extension: self
                        .path
                        .as_ref()
                        .and_then(|path: &PathBuf| path.extension()?.to_str())
                        .unwrap_or("txt")
                        .to_string(),
                },
                |highlight: &highlighter::Highlight, _theme: &Theme| highlight.to_format(),
            )
            .into();

        let status: Element<'_, Message> = {
            let controls: Element<'_, Message> = widget::row![
                widget::button("New").on_press(Message::NewFile),
                widget::Space::with_width(10),
                widget::button("Open").on_press(Message::OpenFile),
                widget::Space::with_width(10),
                widget::button("Save")
                    .style(if self.dirty {
                        theme::Button::Primary
                    } else {
                        theme::Button::Secondary
                    })
                    .on_press_maybe(self.dirty.then_some(Message::SaveFile)),
            ]
            .into();

            let file_path: Element<'_, Message> = widget::text(if self.path.is_some() {
                self.path.as_ref().unwrap().to_str().unwrap()
            } else {
                "New file"
            })
            .into();

            let position: Element<'_, Message> = {
                let (line, column): (usize, usize) = self.content.cursor_position();
                widget::text(format!("{}:{}", line, column)).into()
            };

            widget::row!(
                controls,
                widget::horizontal_space(),
                file_path,
                widget::horizontal_space(),
                position
            )
            .into()
        };

        widget::container(widget::column![
            status,
            widget::Space::with_height(10),
            input,
        ])
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn open_file() -> (Option<PathBuf>, String) {
    let mut path: Option<PathBuf> = rfd::FileDialog::new()
        .set_directory(env::current_dir().unwrap())
        .pick_file();
    let mut contents: String = String::new();

    if path.is_some() {
        let read_contents: Option<String> = fs::read_to_string(path.clone().unwrap()).ok();

        if read_contents.is_some() {
            contents = read_contents.unwrap();
        } else {
            path = None;
        }
    }

    (path, contents)
}

fn save_new_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory(env::current_dir().unwrap())
        .save_file()
}
