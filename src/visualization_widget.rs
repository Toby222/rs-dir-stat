use druid::{im::Vector, Color, Data, LifeCycle, Rect, RenderContext, Size, Widget};

use crate::file_system::FileNode;

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub(crate) struct VisualizationWidget {
    width: f64,
    files: Option<(Vector<FileNode>, u64)>,
}

impl Data for VisualizationWidget {
    fn same(&self, other: &Self) -> bool {
        self.width == other.width && self.files == other.files
    }
}

impl Widget<crate::AppState> for VisualizationWidget {
    fn event(
        &mut self,
        _ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut crate::AppState,
        _env: &druid::Env,
    ) {
        if let druid::Event::MouseDown(event) = event {
            let Some((files, total_size)) = &self.files else {
                tracing::debug!("clicked at x: {}, but don't have any files", event.pos.x);
                return;
            };
            tracing::debug!("clicked at x: {}", event.pos.x);
            let target_size = *total_size as f64 * (event.pos.x / self.width);

            let mut size_so_far = 0;
            let Some(file) = files.iter()
                .skip_while(|&file| {
                    size_so_far += file.size();
                    (size_so_far as f64) < target_size
                })
                .next() else {
                    tracing::warn!("clicked on empty space");
                    data.selected_file = None;
                    return;
                };

            assert!(
                matches!(file, FileNode::File { .. }),
                "Folders shouldn't be clickable"
            );

            tracing::debug!("clicked: {} ({} B)", file.path().display(), file.size());
            data.selected_file = Some(file.clone());
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        _data: &crate::AppState,
        _env: &druid::Env,
    ) {
        if let LifeCycle::Size(size) = event {
            self.width = size.width;
        }
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        _old_data: &crate::AppState,
        data: &crate::AppState,
        _env: &druid::Env,
    ) {
        self.files = match &data.all_files {
            None => None,
            Some(files) => Some((
                files.clone().as_vector(),
                files
                    .clone()
                    .as_vector()
                    .iter()
                    .map(|node| node.size())
                    .sum::<u64>(),
            )),
        };
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        _data: &crate::AppState,
        _env: &druid::Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &crate::AppState, _env: &druid::Env) {
        tracing::debug!("Painting");
        let size = ctx.size();
        ctx.fill(Rect::new(0.0, 0.0, size.width, size.height), &Color::BLACK);

        let Some(files) = &self.files else {
            return;
        };

        let total_filesize: f64 = files.1 as f64;
        tracing::debug!("total filesize is {}", total_filesize);
        let mut done = 0f64;
        for node in files.0.iter() {
            assert!(
                matches!(node, FileNode::File { .. }),
                "VisualizationWidget can only draw Files, not directories"
            );
            let percentage = node.size() as f64 / total_filesize;
            tracing::debug!(
                "Drawing `{}` which makes up {}% of width",
                node.path().display(),
                percentage
            );

            let file_rect = Rect::new(
                size.width * done,
                0.0,
                size.width * (done + percentage),
                size.height,
            );
            // Blue to green (possibly less red/blue for blue light filter)
            // let stroke_color = Color::rgb(0.0, done, 1.0 - done);
            // Red to blue (Bi theme)
            let stroke_color = Color::rgb(1.0 - done, 0.0, done);
            // Greyscale
            // let stroke_color = Color::rgb(done, done, done);
            ctx.fill(file_rect.inset(-1.0), &stroke_color);
            if let Some(selected) = &data.selected_file {
                if node == selected {
                    let contrasting_color = get_contrasting_color(stroke_color);
                    tracing::debug!("contrasting color: {:?}", contrasting_color);
                    ctx.paint_with_z_index(1, move |ctx| {
                        ctx.stroke(file_rect, &contrasting_color, 2.0)
                    });
                }
            }
            done += percentage;
        }
        tracing::debug!("Done painting");
    }
}

fn get_contrasting_color(color: Color) -> Color {
    let (red, green, blue, _) = color.as_rgba();
    // Calculate the relative luminance of the color
    let luminance = 0.2126 * red + 0.7152 * green + 0.0722 * blue;

    // Handle the case of grey shades separately
    if red == green && green == blue {
        let contrasting_lightness = if luminance > 0.5 { 0.0 } else { 1.0 };
        return Color::rgb(
            contrasting_lightness,
            contrasting_lightness,
            contrasting_lightness,
        );
    }

    // Determine the contrasting color by inverting the RGB values and adjusting the lightness
    let contrasting_red = 1.0 - red;
    let contrasting_green = 1.0 - green;
    let contrasting_blue = 1.0 - blue;

    Color::rgb(contrasting_red, contrasting_green, contrasting_blue)
}
