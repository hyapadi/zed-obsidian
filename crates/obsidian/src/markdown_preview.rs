use anyhow::Result;
use gpui::{Context, Model, ModelContext, Task, View, ViewContext};
use markdown_preview::markdown_preview_view::{MarkdownPreviewView, MarkdownPreviewMode};
use std::path::PathBuf;

pub struct ObsidianMarkdownPreview {
    preview: Model<MarkdownPreviewView>,
    path: PathBuf,
}

impl ObsidianMarkdownPreview {
    pub fn new(path: PathBuf, cx: &mut ModelContext<Self>) -> Self {
        let preview = cx.new_model(|_| MarkdownPreviewView::new());
        Self { preview, path }
    }

    pub fn update_preview(&mut self, cx: &mut ModelContext<Self>) -> Task<Result<()>> {
        cx.spawn(|this, mut cx| async move {
            // Update the preview with the latest content
            this.update(&mut cx, |this, cx| {
                this.preview.update(cx, |preview, cx| {
                    preview.set_mode(MarkdownPreviewMode::Default);
                });
            })?;
            Ok(())
        })
    }
}