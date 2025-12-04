#![windows_subsystem = "windows"]

use iced::alignment::Horizontal;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use rfd::FileDialog;
use std::fs;
use std::path::PathBuf;
use std::process::Command as StdCommand;
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
}

struct PdfMergerApp {
    file_paths: Vec<PathBuf>,
    selected_index: Option<usize>,
    status: String,
    progress: f32,
    is_merging: bool,
    last_output: Option<PathBuf>,
}

impl Application for PdfMergerApp {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                file_paths: Vec::new(),
                selected_index: None,
                status: "Ready.".to_string(),
                progress: 0.0,
                is_merging: false,
                last_output: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("PDF Merger")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::SelectFiles => {
                if let Some(paths) = FileDialog::new()
                    .add_filter("PDF Files", &["pdf"])
                    .set_title("Select PDF files")
                    .pick_files()
                {
                    let mut added = 0;
                    for path in paths {
                        if !self.file_paths.contains(&path) {
                            self.file_paths.push(path);
                            added += 1;
                        }
                    }
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
                        self.file_paths.swap(idx, idx - 1);
                        self.selected_index = Some(idx - 1);
                        self.status = format!(
                            "Moved '{}' up.",
                            self.file_paths[idx - 1]
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
                    if idx < self.file_paths.len().saturating_sub(1) {
                        self.file_paths.swap(idx, idx + 1);
                        self.selected_index = Some(idx + 1);
                        self.status = format!(
                            "Moved '{}' down.",
                            self.file_paths[idx + 1]
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
                    if idx > 0 && idx < self.file_paths.len() {
                        let path = self.file_paths.remove(idx);
                        self.file_paths.insert(0, path);
                        self.selected_index = Some(0);
                        self.status = "Moved file to top.".to_string();
                    }
                }
                Command::none()
            }
            Message::MoveBottom => {
                if let Some(idx) = self.selected_index {
                    if idx < self.file_paths.len().saturating_sub(1) {
                        let path = self.file_paths.remove(idx);
                        self.file_paths.push(path);
                        self.selected_index = Some(self.file_paths.len().saturating_sub(1));
                        self.status = "Moved file to bottom.".to_string();
                    }
                }
                Command::none()
            }
            Message::RemoveSelected => {
                if let Some(idx) = self.selected_index {
                    if idx < self.file_paths.len() {
                        let removed_name = self.file_paths[idx]
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("file")
                            .to_string();
                        self.file_paths.remove(idx);
                        self.selected_index = if idx > 0 && idx <= self.file_paths.len() {
                            Some(idx - 1)
                        } else if !self.file_paths.is_empty() && idx < self.file_paths.len() {
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
                self.file_paths.clear();
                self.selected_index = None;
                self.status = "Cleared file list.".to_string();
                Command::none()
            }
            Message::MergePdfs => {
                if self.file_paths.is_empty() {
                    self.status = "Please select at least one PDF file to merge.".to_string();
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

                    let files = self.file_paths.clone();
                    let output_clone = output_path.clone();
                    return Command::perform(
                        async move {
                            merge_pdfs_async_with_progress(files, output_path).await
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let move_up_enabled = self.selected_index.is_some() && !self.is_merging;
        let move_down_enabled = self.selected_index.is_some()
            && self.selected_index
                .map(|i| i < self.file_paths.len().saturating_sub(1))
                .unwrap_or(false)
            && !self.is_merging;
        let move_top_enabled = move_up_enabled;
        let move_bottom_enabled = move_down_enabled;
        let remove_enabled = self.selected_index.is_some() && !self.is_merging;
        let clear_enabled = !self.file_paths.is_empty() && !self.is_merging;
        let merge_enabled = !self.file_paths.is_empty() && !self.is_merging;

        let move_up_btn: Element<Message> = if move_up_enabled {
            button("Move Up")
                .on_press(Message::MoveUp)
                .width(Length::Fill)
                .into()
        } else {
            button("Move Up")
                .width(Length::Fill)
                .style(iced::theme::Button::Secondary)
                .into()
        };

        let move_down_btn: Element<Message> = if move_down_enabled {
            button("Move Down")
                .on_press(Message::MoveDown)
                .width(Length::Fill)
                .into()
        } else {
            button("Move Down")
                .width(Length::Fill)
                .style(iced::theme::Button::Secondary)
                .into()
        };

        let move_top_btn: Element<Message> = if move_top_enabled {
            button("Top")
                .on_press(Message::MoveTop)
                .width(Length::Fill)
                .into()
        } else {
            button("Top")
                .width(Length::Fill)
                .style(iced::theme::Button::Secondary)
                .into()
        };

        let move_bottom_btn: Element<Message> = if move_bottom_enabled {
            button("Bottom")
                .on_press(Message::MoveBottom)
                .width(Length::Fill)
                .into()
        } else {
            button("Bottom")
                .width(Length::Fill)
                .style(iced::theme::Button::Secondary)
                .into()
        };

        let remove_btn: Element<Message> = if remove_enabled {
            button("Remove Selected")
                .on_press(Message::RemoveSelected)
                .width(Length::Fill)
                .into()
        } else {
            button("Remove Selected")
                .width(Length::Fill)
                .style(iced::theme::Button::Secondary)
                .into()
        };

        let clear_btn: Element<Message> = if clear_enabled {
            button("Clear List")
                .on_press(Message::ClearList)
                .width(Length::Fill)
                .into()
        } else {
            button("Clear List")
                .width(Length::Fill)
                .style(iced::theme::Button::Secondary)
                .into()
        };

        let merge_btn: Element<Message> = if merge_enabled {
            button("Merge PDFs")
                .on_press(Message::MergePdfs)
                .width(Length::Fill)
                .into()
        } else {
            button("Merge PDFs")
                .width(Length::Fill)
                .style(iced::theme::Button::Secondary)
                .into()
        };

        let controls = column![
            row![
                button("Select Files")
                    .on_press(Message::SelectFiles)
                    .width(Length::Fill),
                move_up_btn,
                move_down_btn,
                move_top_btn,
                move_bottom_btn,
            ]
            .spacing(5)
            .width(Length::Fill),
            row![remove_btn, clear_btn, merge_btn]
                .spacing(5)
                .width(Length::Fill),
        ]
        .spacing(5)
        .width(Length::Fill);

        let file_list: Column<Message> = if self.file_paths.is_empty() {
            Column::new().push(
                text("No files selected. Click 'Select Files' to add PDFs.")
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
            )
        } else {
            let mut col = Column::new().spacing(2);
            for (idx, path) in self.file_paths.iter().enumerate() {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown file");
                let size_label = fs::metadata(path)
                    .map(|m| format_size(m.len()))
                    .unwrap_or_else(|_| "-".to_string());
                let label = format!("#{} {} ({})", idx + 1, file_name, size_label);
                let is_selected = self.selected_index == Some(idx);

                let file_text = if is_selected {
                    text(label)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.6, 1.0)))
                } else {
                    text(label)
                };

                let file_button = button(file_text)
                    .style(if is_selected {
                        iced::theme::Button::Primary
                    } else {
                        iced::theme::Button::Secondary
                    })
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

        let status_text = text(&self.status)
            .horizontal_alignment(Horizontal::Left)
            .width(Length::Fill);

        let progress = progress_bar(0.0..=1.0, self.progress)
            .width(Length::Fill)
            .height(20);

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
            row![status_text, open_btn].spacing(10).width(Length::Fill),
            progress
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
) -> Result<(), String> {
    let total_files = file_paths.len();
    
    // Run the merge operation in a blocking task
    // Progress updates are shown through status messages
    tokio::task::spawn_blocking(move || {
        merge_pdfs_with_progress(file_paths, output_path, total_files)
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
