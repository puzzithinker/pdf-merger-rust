#![windows_subsystem = "windows"]

use iced::alignment::Horizontal;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Subscription, Theme};
use iced::window;
use iced::keyboard;
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use parking_lot::Mutex;
use pdf_merger::merge_pdfs_with_progress;

pub fn main() -> iced::Result {
    PdfMergerApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(800.0, 500.0),
            min_size: Some(iced::Size::new(700.0, 400.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
enum Message {
    SelectFiles,
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    RemoveSelected,
    ClearList,
    MergePdfs,
    FileSelected(usize),
    MergeComplete(Result<PathBuf, String>),
    OpenOutputLocation,
    FilesDropped(Vec<PathBuf>),
    ProgressTick,
    KeyPressed(keyboard::Key, keyboard::Modifiers),
}

struct PdfMergerApp {
    files: Vec<FileEntry>,
    selected_index: Option<usize>,
    status: String,
    progress: f32,
    is_merging: bool,
    last_output: Option<PathBuf>,
    progress_state: Option<Arc<ProgressState>>,
}

#[derive(Clone)]
struct FileEntry {
    path: PathBuf,
    error: Option<String>,
}

struct ProgressState {
    current: AtomicUsize,
    total: usize,
    last_file: Mutex<String>,
}

impl Application for PdfMergerApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                files: Vec::new(),
                selected_index: None,
                status: "Ready.".to_string(),
                progress: 0.0,
                is_merging: false,
                last_output: None,
                progress_state: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("PDF Merger")
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::event;
        let mut subs = Vec::new();

        // File drop handling and keyboard events
        let event_sub = event::listen_with(|event, _status| match event {
            event::Event::Window(_, window::Event::FileDropped(path)) => {
                Some(Message::FilesDropped(vec![path]))
            }
            event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(Message::KeyPressed(key, modifiers))
            }
            _ => None,
        });
        subs.push(event_sub);

        // Progress polling while merging
        if self.is_merging {
            subs.push(iced::time::every(std::time::Duration::from_millis(150)).map(|_| Message::ProgressTick));
        }

        Subscription::batch(subs)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectFiles => {
                if let Some(paths) = FileDialog::new()
                    .add_filter("PDF Files", &["pdf"])
                    .set_title("Select PDF files")
                    .pick_files()
                {
                    let added = self.add_files(paths);
                    if added > 0 {
                        self.status = format!("Added {} file(s).", added);
                    } else {
                        self.status = "No new files added (duplicates ignored).".to_string();
                    }
                }
                Command::none()
            }
            Message::MoveUp => {
                if let Some(idx) = self.selected_index {
                    if idx > 0 {
                        self.files.swap(idx, idx - 1);
                        self.selected_index = Some(idx - 1);
                        self.status = format!(
                            "Moved '{}' up.",
                            self.files[idx - 1]
                                .path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("file")
                        );
                    }
                }
                Command::none()
            }
            Message::MoveDown => {
                if let Some(idx) = self.selected_index {
                    if idx < self.files.len().saturating_sub(1) {
                        self.files.swap(idx, idx + 1);
                        self.selected_index = Some(idx + 1);
                        self.status = format!(
                            "Moved '{}' down.",
                            self.files[idx + 1]
                                .path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("file")
                        );
                    }
                }
                Command::none()
            }
            Message::MoveTop => {
                if let Some(idx) = self.selected_index {
                    if idx > 0 && idx < self.files.len() {
                        let entry = self.files.remove(idx);
                        self.files.insert(0, entry);
                        self.selected_index = Some(0);
                        self.status = "Moved file to top.".to_string();
                    }
                }
                Command::none()
            }
            Message::MoveBottom => {
                if let Some(idx) = self.selected_index {
                    if idx < self.files.len().saturating_sub(1) {
                        let entry = self.files.remove(idx);
                        self.files.push(entry);
                        self.selected_index = Some(self.files.len().saturating_sub(1));
                        self.status = "Moved file to bottom.".to_string();
                    }
                }
                Command::none()
            }
            Message::RemoveSelected => {
                if let Some(idx) = self.selected_index {
                    if idx < self.files.len() {
                        let removed_name = self.files[idx]
                            .path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file")
                            .to_string();
                        self.files.remove(idx);
                        self.selected_index = if idx > 0 && idx <= self.files.len() {
                            Some(idx - 1)
                        } else if !self.files.is_empty() && idx < self.files.len() {
                            Some(idx)
                        } else {
                            None
                        };
                        self.status = format!("Removed '{}'.", removed_name);
                    }
                }
                Command::none()
            }
            Message::ClearList => {
                self.files.clear();
                self.selected_index = None;
                self.status = "Cleared file list.".to_string();
                Command::none()
            }
            Message::MergePdfs => {
                if self.files.is_empty() {
                    self.status = "Please select at least one PDF file to merge.".to_string();
                    return Command::none();
                }

                if let Some(err) = validate_inputs(&self.files.iter().map(|f| f.path.clone()).collect::<Vec<_>>()) {
                    self.status = err;
                    return Command::none();
                }

                if let Some(output_path) = FileDialog::new()
                    .add_filter("PDF Files", &["pdf"])
                    .set_title("Save Merged PDF As")
                    .set_file_name("merged.pdf")
                    .save_file()
                {
                    self.is_merging = true;
                    self.progress = 0.0;
                    self.last_output = None;
                    self.status = "Merging PDFs...".to_string();

                    let files: Vec<PathBuf> = self.files.iter().map(|f| f.path.clone()).collect();
                    let output_clone = output_path.clone();
                    let progress_state = Arc::new(ProgressState {
                        current: AtomicUsize::new(0),
                        total: files.len(),
                        last_file: Mutex::new(String::new()),
                    });
                    self.progress_state = Some(progress_state.clone());
                    return Command::perform(
                        async move {
                            merge_pdfs_async_with_progress(files, output_path, Some(progress_state)).await
                        },
                        move |result| Message::MergeComplete(result.map(|_| output_clone)),
                    );
                }
                Command::none()
            }
            Message::FileSelected(index) => {
                self.selected_index = Some(index);
                Command::none()
            }
            Message::MergeComplete(result) => {
                self.is_merging = false;
                self.progress_state = None;
                match result {
                    Ok(path) => {
                        self.progress = 1.0;
                        self.last_output = Some(path.clone());
                        self.status = format!(
                            "Merge completed successfully: {}",
                            path.display()
                        );
                    }
                    Err(e) => {
                        self.progress = 0.0;
                        self.last_output = None;
                        self.status = format!("Error: {}", e);
                    }
                }
                Command::none()
            }
            Message::FilesDropped(paths) => {
                let added = self.add_files(paths);
                if added > 0 {
                    self.status = format!("Added {} file(s) via drag-and-drop.", added);
                }
                Command::none()
            }
            Message::ProgressTick => {
                if let Some(progress) = &self.progress_state {
                    let total = progress.total as f32;
                    let current = progress.current.load(Ordering::Relaxed) as f32;
                    if total > 0.0 {
                        self.progress = (current / total).min(1.0);
                    }
                    let last = progress.last_file.lock().clone();
                    if !last.is_empty() {
                        self.status = format!("Processing: {} ({}/{})", last, current as usize, progress.total);
                    }
                }
                Command::none()
            }
            Message::OpenOutputLocation => {
                if let Some(path) = &self.last_output {
                    // Attempt to open location in Explorer on Windows
                    let _ = StdCommand::new("explorer")
                        .arg("/select,")
                        .arg(path)
                        .spawn();
                }
                Command::none()
            }
            Message::KeyPressed(key, modifiers) => {
                use iced::keyboard::key::Named;

                // Don't process shortcuts while merging
                if self.is_merging {
                    return Command::none();
                }

                match key.as_ref() {
                    // Ctrl/Cmd + O: Select Files
                    keyboard::Key::Character(c) if c == "o" && (modifiers.command() || modifiers.control()) => {
                        return self.update(Message::SelectFiles);
                    }
                    // Ctrl/Cmd + M: Merge PDFs
                    keyboard::Key::Character(c) if c == "m" && (modifiers.command() || modifiers.control()) => {
                        if !self.files.is_empty() && self.files.iter().all(|f| f.error.is_none()) {
                            return self.update(Message::MergePdfs);
                        }
                    }
                    // Delete: Remove selected file
                    keyboard::Key::Named(Named::Delete) | keyboard::Key::Named(Named::Backspace) => {
                        if self.selected_index.is_some() {
                            return self.update(Message::RemoveSelected);
                        }
                    }
                    // Arrow Up: Move selection up or move file up (with Ctrl)
                    keyboard::Key::Named(Named::ArrowUp) => {
                        if modifiers.control() || modifiers.command() {
                            return self.update(Message::MoveUp);
                        } else if let Some(idx) = self.selected_index {
                            if idx > 0 {
                                self.selected_index = Some(idx - 1);
                            }
                        } else if !self.files.is_empty() {
                            self.selected_index = Some(self.files.len() - 1);
                        }
                    }
                    // Arrow Down: Move selection down or move file down (with Ctrl)
                    keyboard::Key::Named(Named::ArrowDown) => {
                        if modifiers.control() || modifiers.command() {
                            return self.update(Message::MoveDown);
                        } else if let Some(idx) = self.selected_index {
                            if idx < self.files.len().saturating_sub(1) {
                                self.selected_index = Some(idx + 1);
                            }
                        } else if !self.files.is_empty() {
                            self.selected_index = Some(0);
                        }
                    }
                    // Home: Move to top (with Ctrl) or select first
                    keyboard::Key::Named(Named::Home) => {
                        if modifiers.control() || modifiers.command() {
                            return self.update(Message::MoveTop);
                        } else if !self.files.is_empty() {
                            self.selected_index = Some(0);
                        }
                    }
                    // End: Move to bottom (with Ctrl) or select last
                    keyboard::Key::Named(Named::End) => {
                        if modifiers.control() || modifiers.command() {
                            return self.update(Message::MoveBottom);
                        } else if !self.files.is_empty() {
                            self.selected_index = Some(self.files.len() - 1);
                        }
                    }
                    _ => {}
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let move_up_enabled = self.selected_index.is_some() && !self.is_merging;
        let move_down_enabled = self.selected_index.is_some()
            && self.selected_index
                .map(|i| i < self.files.len().saturating_sub(1))
                .unwrap_or(false)
            && !self.is_merging;
        let move_top_enabled = move_up_enabled;
        let move_bottom_enabled = move_down_enabled;
        let remove_enabled = self.selected_index.is_some() && !self.is_merging;
        let clear_enabled = !self.files.is_empty() && !self.is_merging;
        let merge_enabled = !self.files.is_empty()
            && !self.is_merging
            && self.files.iter().all(|f| f.error.is_none());

        // Primary action button - most prominent
        let select_files_btn = button(
            row![
                text("📁").size(16),
                text("Add Files")
            ]
            .spacing(8)
            .align_items(iced::Alignment::Center)
        )
        .on_press(Message::SelectFiles)
        .padding(12)
        .style(iced::theme::Button::Primary);

        // Reorder controls - grouped together visually
        let move_up_btn = button(
            row![
                text("↑").size(16),
                text("Up")
            ]
            .spacing(6)
            .align_items(iced::Alignment::Center)
        )
        .padding([8, 12])
        .style(if move_up_enabled {
            iced::theme::Button::Secondary
        } else {
            iced::theme::Button::Text
        });
        let move_up_btn: Element<Message> = if move_up_enabled {
            move_up_btn.on_press(Message::MoveUp).into()
        } else {
            move_up_btn.into()
        };

        let move_down_btn = button(
            row![
                text("↓").size(16),
                text("Down")
            ]
            .spacing(6)
            .align_items(iced::Alignment::Center)
        )
        .padding([8, 12])
        .style(if move_down_enabled {
            iced::theme::Button::Secondary
        } else {
            iced::theme::Button::Text
        });
        let move_down_btn: Element<Message> = if move_down_enabled {
            move_down_btn.on_press(Message::MoveDown).into()
        } else {
            move_down_btn.into()
        };

        let move_top_btn = button(text("⇈").size(18))
            .padding([8, 12])
            .style(if move_top_enabled {
                iced::theme::Button::Secondary
            } else {
                iced::theme::Button::Text
            });
        let move_top_btn: Element<Message> = if move_top_enabled {
            move_top_btn.on_press(Message::MoveTop).into()
        } else {
            move_top_btn.into()
        };

        let move_bottom_btn = button(text("⇊").size(18))
            .padding([8, 12])
            .style(if move_bottom_enabled {
                iced::theme::Button::Secondary
            } else {
                iced::theme::Button::Text
            });
        let move_bottom_btn: Element<Message> = if move_bottom_enabled {
            move_bottom_btn.on_press(Message::MoveBottom).into()
        } else {
            move_bottom_btn.into()
        };

        // Reorder group with visual separator
        let reorder_group = container(
            row![
                move_top_btn,
                move_up_btn,
                move_down_btn,
                move_bottom_btn,
            ]
            .spacing(4)
            .align_items(iced::Alignment::Center)
        )
        .padding(6)
        .style(|_theme: &Theme| container::Appearance {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.2, 0.2, 0.2, 0.4))),
            border: iced::Border {
                radius: 6.0.into(),
                ..Default::default()
            },
            ..Default::default()
        });

        // Destructive actions - visually separated
        let remove_btn = button(
            row![
                text("🗑").size(14),
                text("Remove")
            ]
            .spacing(6)
            .align_items(iced::Alignment::Center)
        )
        .padding([8, 12])
        .style(if remove_enabled {
            iced::theme::Button::Destructive
        } else {
            iced::theme::Button::Text
        });
        let remove_btn: Element<Message> = if remove_enabled {
            remove_btn.on_press(Message::RemoveSelected).into()
        } else {
            remove_btn.into()
        };

        let clear_btn = button(
            row![
                text("✕").size(14),
                text("Clear All")
            ]
            .spacing(6)
            .align_items(iced::Alignment::Center)
        )
        .padding([8, 12])
        .style(if clear_enabled {
            iced::theme::Button::Destructive
        } else {
            iced::theme::Button::Text
        });
        let clear_btn: Element<Message> = if clear_enabled {
            clear_btn.on_press(Message::ClearList).into()
        } else {
            clear_btn.into()
        };

        // Primary CTA - larger and prominent
        let merge_btn = button(
            row![
                text("⚡").size(18),
                text("Merge PDFs").size(16)
            ]
            .spacing(8)
            .align_items(iced::Alignment::Center)
        )
        .padding(12)
        .style(if merge_enabled {
            iced::theme::Button::Primary
        } else {
            iced::theme::Button::Text
        });
        let merge_btn: Element<Message> = if merge_enabled {
            merge_btn.on_press(Message::MergePdfs).into()
        } else {
            merge_btn.into()
        };

        // Layout: Primary actions on left, reorder in center, merge on right
        let controls = row![
            // Left: File management
            column![
                select_files_btn,
                row![remove_btn, clear_btn].spacing(6)
            ]
            .spacing(6)
            .width(Length::FillPortion(2)),

            // Center: Reorder controls
            container(reorder_group)
                .width(Length::FillPortion(2))
                .center_x(),

            // Right: Primary action
            container(merge_btn)
                .width(Length::FillPortion(2))
                .center_x(),
        ]
        .spacing(12)
        .align_items(iced::Alignment::Center)
        .width(Length::Fill);

        let file_list: Column<Message> = if self.files.is_empty() {
            Column::new()
                .push(
                    container(
                        column![
                            text("📄").size(48),
                            text("Drag & drop PDF files here")
                                .size(20)
                                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.8, 0.8, 0.8))),
                            text("or click 'Select Files' button above")
                                .size(14)
                                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                            text("Supports multiple PDF files")
                                .size(12)
                                .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                        ]
                        .spacing(10)
                        .align_items(iced::Alignment::Center),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .style(|_theme: &Theme| container::Appearance {
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                            width: 2.0,
                            radius: 10.0.into(),
                        },
                        background: Some(iced::Background::Color(iced::Color::from_rgba(0.2, 0.2, 0.2, 0.3))),
                        ..Default::default()
                    }),
                )
                .width(Length::Fill)
                .height(Length::Fill)
        } else {
            let mut col = Column::new().spacing(8);
            for (idx, entry) in self.files.iter().enumerate() {
                let file_name = entry
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown file");
                let size_label = fs::metadata(&entry.path)
                    .map(|m| format_size(m.len()))
                    .unwrap_or_else(|_| "-".to_string());
                let is_selected = self.selected_index == Some(idx);

                // Number badge with file info
                let number_badge = container(
                    text(format!("{}", idx + 1))
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 1.0, 1.0)))
                )
                .padding(4)
                .style(move |_theme: &Theme| container::Appearance {
                    background: Some(iced::Background::Color(
                        if is_selected {
                            iced::Color::from_rgb(0.3, 0.7, 1.0)
                        } else {
                            iced::Color::from_rgb(0.4, 0.4, 0.4)
                        }
                    )),
                    border: iced::Border {
                        radius: 12.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                let file_info_row = row![
                    number_badge,
                    text("📄").size(16),
                    column![
                        text(file_name).size(14).style(iced::theme::Text::Color(
                            if is_selected {
                                iced::Color::from_rgb(0.3, 0.7, 1.0)
                            } else {
                                iced::Color::from_rgb(0.95, 0.95, 0.95)
                            }
                        )),
                        text(size_label).size(11).style(iced::theme::Text::Color(
                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                        ))
                    ]
                    .spacing(2)
                ]
                .spacing(10)
                .align_items(iced::Alignment::Center);

                let mut entry_col = column![file_info_row];

                if let Some(err) = &entry.error {
                    entry_col = entry_col.push(
                        row![
                            text("⚠").size(14).style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.4, 0.4))),
                            text(err).size(12).style(iced::theme::Text::Color(iced::Color::from_rgb(1.0, 0.4, 0.4)))
                        ]
                        .spacing(6)
                        .align_items(iced::Alignment::Center)
                    );
                }

                let file_container = container(entry_col.spacing(6))
                    .padding(12)
                    .width(Length::Fill)
                    .style(move |_theme: &Theme| container::Appearance {
                        background: Some(iced::Background::Color(
                            if is_selected {
                                iced::Color::from_rgba(0.2, 0.5, 0.8, 0.2)
                            } else {
                                iced::Color::from_rgba(0.15, 0.15, 0.15, 0.5)
                            }
                        )),
                        border: iced::Border {
                            color: if is_selected {
                                iced::Color::from_rgb(0.3, 0.7, 1.0)
                            } else {
                                iced::Color::from_rgba(0.3, 0.3, 0.3, 0.0)
                            },
                            width: if is_selected { 3.0 } else { 0.0 },
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    });

                let file_button = button(file_container)
                    .style(iced::theme::Button::Text)
                    .width(Length::Fill)
                    .on_press(Message::FileSelected(idx));

                col = col.push(file_button);
            }
            col
        };

        let scrollable_list = scrollable(
            container(file_list)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10),
        )
        .height(Length::Fill);

        let (status_icon, status_color) = get_status_style(&self.status, self.is_merging);
        let status_text = row![
            text(status_icon).size(16),
            text(&self.status)
        ]
        .spacing(8)
        .align_items(iced::Alignment::Center);

        let status_container = container(status_text)
            .padding(8)
            .width(Length::Fill)
            .style(move |_theme: &Theme| container::Appearance {
                background: Some(iced::Background::Color(status_color)),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        let progress_widget = if self.is_merging || self.progress > 0.0 {
            let percentage = (self.progress * 100.0) as i32;
            let progress_text = if self.is_merging {
                text(format!("{}%", percentage))
                    .size(12)
                    .horizontal_alignment(Horizontal::Center)
                    .width(Length::Fill)
            } else {
                text("") // Hide percentage when not merging
            };

            column![
                progress_bar(0.0..=1.0, self.progress)
                    .width(Length::Fill)
                    .height(20),
                progress_text
            ]
            .spacing(2)
        } else {
            column![
                progress_bar(0.0..=1.0, 0.0)
                    .width(Length::Fill)
                    .height(20)
            ]
        };

        let open_btn: Element<Message> = if self.last_output.is_some() {
            button("Open Output Location")
                .on_press(Message::OpenOutputLocation)
                .width(Length::Shrink)
                .into()
        } else {
            button("Open Output Location")
                .style(iced::theme::Button::Secondary)
                .width(Length::Shrink)
                .into()
        };

        let status_section = column![
            row![status_container, open_btn].spacing(10).width(Length::Fill).align_items(iced::Alignment::Center),
            progress_widget
        ]
        .spacing(5)
        .width(Length::Fill);

        column![
            container(controls)
                .width(Length::Fill)
                .padding(10),
            container(scrollable_list)
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10),
            container(status_section)
                .width(Length::Fill)
                .padding(10),
        ]
        .spacing(5)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn merge_pdfs_async_with_progress(
    file_paths: Vec<PathBuf>,
    output_path: PathBuf,
    progress: Option<Arc<ProgressState>>,
) -> Result<(), String> {
    let total_files = file_paths.len();

    // Run the merge operation in a blocking task
    tokio::task::spawn_blocking(move || {
        let callback = progress.map(|p| {
            move |current: usize, _total: usize, path: &PathBuf| {
                p.current.store(current, Ordering::Relaxed);
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    *p.last_file.lock() = name.to_string();
                }
            }
        });
        merge_pdfs_with_progress(file_paths, output_path, total_files, callback)
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.1} MB", b / MB)
    } else if b >= KB {
        format!("{:.0} KB", b / KB)
    } else {
        format!("{:.0} B", b)
    }
}

fn get_status_style(status: &str, is_merging: bool) -> (&'static str, iced::Color) {
    if is_merging {
        // Processing state - blue/cyan
        ("⏳", iced::Color::from_rgba(0.2, 0.4, 0.6, 0.3))
    } else if status.starts_with("Error") || status.contains("not found") || status.contains("empty") || status.contains("Not a PDF") {
        // Error state - red
        ("❌", iced::Color::from_rgba(0.6, 0.2, 0.2, 0.3))
    } else if status.starts_with("Merge completed") || status.starts_with("Added") || status.contains("successfully") {
        // Success state - green
        ("✓", iced::Color::from_rgba(0.2, 0.6, 0.3, 0.3))
    } else if status.starts_with("Removed") || status.starts_with("Cleared") || status.starts_with("Moved") || status.contains("duplicates ignored") {
        // Info/action state - yellow/amber
        ("ℹ", iced::Color::from_rgba(0.5, 0.4, 0.2, 0.3))
    } else {
        // Default/ready state - neutral
        ("●", iced::Color::from_rgba(0.3, 0.3, 0.3, 0.3))
    }
}

fn validate_inputs(paths: &[PathBuf]) -> Option<String> {
    for path in paths {
        if !path.exists() {
            return Some(format!("File not found: {}", path.display()));
        }
        if path.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
            return Some(format!("File is empty: {}", path.display()));
        }
        if path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()) != Some("pdf".to_string()) {
            return Some(format!("Not a PDF: {}", path.display()));
        }
    }
    None
}

impl PdfMergerApp {
    fn add_files(&mut self, paths: Vec<PathBuf>) -> usize {
        let mut added = 0;
        for path in paths {
            if self.files.iter().any(|f| f.path == path) {
                continue;
            }
            let error = validate_inputs(&[path.clone()]);
            self.files.push(FileEntry { path, error });
            added += 1;
        }
        added
    }
}
