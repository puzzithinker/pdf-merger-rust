#![windows_subsystem = "windows"]

use iced::alignment::Horizontal;
use iced::widget::{button, column, container, progress_bar, row, scrollable, text, Column};
use iced::{executor, Application, Command, Element, Length, Settings, Subscription, Theme};
use iced::window;
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

        // File drop handling
        let drop_sub = event::listen_with(|event, _status| match event {
            event::Event::Window(_, window::Event::FileDropped(path)) => {
                Some(Message::FilesDropped(vec![path]))
            }
            _ => None,
        });
        subs.push(drop_sub);

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

        let file_list: Column<Message> = if self.files.is_empty() {
            Column::new().push(
                text("No files selected. Click 'Select Files' to add PDFs.")
                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
            )
        } else {
            let mut col = Column::new().spacing(2);
            for (idx, entry) in self.files.iter().enumerate() {
                let file_name = entry
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown file");
                let size_label = fs::metadata(&entry.path)
                    .map(|m| format_size(m.len()))
                    .unwrap_or_else(|_| "-".to_string());
                let label = format!("#{} {} ({})", idx + 1, file_name, size_label);
                let is_selected = self.selected_index == Some(idx);

                let mut entry_col = column![
                    text(label).style(if is_selected {
                        iced::theme::Text::Color(iced::Color::from_rgb(0.2, 0.6, 1.0))
                    } else {
                        iced::theme::Text::Color(iced::Color::from_rgb(0.9, 0.9, 0.9))
                    })
                ];

                if let Some(err) = &entry.error {
                    entry_col = entry_col.push(
                        text(err).style(iced::theme::Text::Color(iced::Color::from_rgb(
                            0.9, 0.3, 0.3,
                        ))),
                    );
                }

                let file_button = button(entry_col.spacing(2))
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
