use iced::{
    alignment, executor, font, highlighter, keyboard, theme, widget, Application, Command, Element,
    Result, Settings, Subscription, Theme,
};

use std::{env, fs, path::PathBuf};

fn main() -> Result {
    Editor::run(Settings {
        default_font: font::Font::with_name("Noto Sans Mono"),
        ..Settings::default()
    })
}

struct Editor {
    path: Option<PathBuf>,
    content: widget::text_editor::Content,
    dirty: bool,
    color_theme: Theme,
    highlighter_theme: highlighter::Theme,
}

#[derive(Debug, Clone)]
enum Message {
    EditorAction(widget::text_editor::Action),
    ColorThemeChange(theme::Theme),
    HighlighterThemeChange(highlighter::Theme),
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
                color_theme: Theme::GruvboxDark,
                highlighter_theme: highlighter::Theme::Base16Mocha,
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
            Message::ColorThemeChange(theme) => {
                self.color_theme = theme;
            }
            Message::HighlighterThemeChange(theme) => {
                self.highlighter_theme = theme;
            }
        };

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::on_key_press(|key_code, modifiers| match key_code {
            keyboard::Key::Character(c) if modifiers.command() => match c.to_string().as_str() {
                "s" => Some(Message::SaveFile),
                "o" => Some(Message::OpenFile),
                "n" => Some(Message::NewFile),
                _ => None,
            },
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
                    theme: self.highlighter_theme,
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
            let controls_l: Element<'_, Message> = widget::row![
                widget::button("New").on_press(Message::NewFile),
                widget::button("Open").on_press(Message::OpenFile),
                widget::button("Save")
                    .style(if self.dirty {
                        theme::Button::Primary
                    } else {
                        theme::Button::Secondary
                    })
                    .on_press_maybe(self.dirty.then_some(Message::SaveFile)),
            ]
            .align_items(alignment::Alignment::Center)
            .spacing(10)
            .into();

            let file_path: Element<'_, Message> = widget::text(if self.path.is_some() {
                self.path.as_ref().unwrap().to_str().unwrap()
            } else {
                "New file"
            })
            .into();

            let controls_r: Element<'_, Message> = widget::row!(
                widget::pick_list(
                    Theme::ALL,
                    Some(self.color_theme.clone()),
                    Message::ColorThemeChange,
                ),
                widget::pick_list(
                    highlighter::Theme::ALL,
                    Some(self.highlighter_theme),
                    Message::HighlighterThemeChange,
                ),
                {
                    let (line, column): (usize, usize) = self.content.cursor_position();
                    widget::text(format!("{}:{}", line, column))
                }
            )
            .align_items(alignment::Alignment::Center)
            .spacing(10)
            .into();

            widget::row!(
                controls_l,
                widget::horizontal_space(),
                file_path,
                widget::horizontal_space(),
                controls_r
            )
            .align_items(alignment::Alignment::Center)
            .into()
        };

        widget::container(widget::column![status, input,].spacing(10))
            .padding(10)
            .into()
    }

    fn theme(&self) -> Theme {
        self.color_theme.clone()
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
