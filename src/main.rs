#![windows_subsystem = "windows"]

mod file_system;
mod visualization_widget;

use std::path::PathBuf;

use druid::widget::{Button, CrossAxisAlignment, Flex, FlexParams, Label, TextBox};
use druid::{AppLauncher, Data, Lens, UnitPoint, Widget, WidgetExt, WindowDesc};
use file_system::{traverse_files_parallel, FileNode};
use visualization_widget::VisualizationWidget;

#[derive(Debug, Clone, Lens)]
struct AppState {
    folder: String,
    selected_file: Option<FileNode>,
    all_files: Option<FileNode>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            folder: "/home/toby/repos/chris/public".into(),
            selected_file: Default::default(),
            all_files: Default::default(),
        }
    }
}

impl Data for AppState {
    fn same(&self, other: &Self) -> bool {
        self.folder == other.folder
            && self.selected_file == other.selected_file
            && self.all_files == other.all_files
    }
}

fn main_widget() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(
                    TextBox::new()
                        .with_placeholder("Folder path...")
                        .lens(AppState::folder)
                        .fix_width(200.0),
                )
                .with_flex_child(
                    Label::dynamic(|state: &AppState, _env| match &state.selected_file {
                        Some(file) => file.path().display().to_string(),
                        None => String::default(),
                    })
                    .expand_width(),
                    1.0,
                )
                .with_child(
                    Button::new("Traverse folder")
                        .on_click(|_ctx, state: &mut AppState, _env| {
                            tracing::debug!("Clicky clicky! {}", &state.folder);
                            state.all_files =
                                traverse_files_parallel(&PathBuf::from(&state.folder));
                            match &state.all_files {
                                Some(files) => tracing::debug!("Found these files: {:?}", files),
                                None => tracing::debug!("Found no files"),
                            }
                        })
                        .align_horizontal(UnitPoint::LEFT),
                ),
        )
        .with_flex_child(
            VisualizationWidget::default(),
            FlexParams::new(1.0, CrossAxisAlignment::Fill),
        )
        .main_axis_alignment(druid::widget::MainAxisAlignment::Start)
}

pub fn main() {
    let window = WindowDesc::new(main_widget()).title(String::from("rs-dir-stat"));
    AppLauncher::with_window(window)
        .log_to_console()
        .launch(AppState::default())
        .expect("launch failed");
}
