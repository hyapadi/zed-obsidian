use anyhow::Result;
use editor::{Editor, EditorElement, EditorStyle};
use gpui::{div, AnyElement, Context, Element, IntoElement, Model, Render, View, ViewContext, WindowContext};
use language::Language;
use markdown::Markdown;
use std::{path::PathBuf, sync::Arc};
use crate::markdown_preview::ObsidianMarkdownPreview;

pub struct NoteEditor {
    editor: Model<Editor>,
    markdown: Option<Model<Markdown>>,
    path: PathBuf,
    preview_mode: bool,
    preview: Option<Model<ObsidianMarkdownPreview>>,
}

impl NoteEditor {
    pub fn new(editor: Model<Editor>, path: PathBuf, cx: &mut ViewContext<Self>) -> Self {
        let preview = cx.new_model(|cx| ObsidianMarkdownPreview::new(path.clone(), cx));
        Self {
            editor,
            markdown: None,
            path,
            preview_mode: false,
            preview: Some(preview),
        }
    }

    pub fn toggle_preview(&mut self, cx: &mut ViewContext<Self>) {
        self.preview_mode = !self.preview_mode;
        if self.preview_mode && self.markdown.is_none() {
            self.update_preview(cx);
        }
        cx.notify();
    }

    fn update_preview(&mut self, cx: &mut ViewContext<Self>) {
        let text = self.editor.read(cx).text();
        let markdown = Markdown::new(
            text.into(),
            markdown::MarkdownStyle::default(),
            None,
            None,
            cx,
        );
        self.markdown = Some(cx.new_model(|_| markdown));
    }

    pub fn handle_wiki_link(&mut self, link: &str, cx: &mut ViewContext<Self>) -> Result<()> {
        // Extract note name from [[note-name]] format
        let note_name = link.trim_start_matches("[[").trim_end_matches("]]");
        
        // Resolve the note path relative to the vault
        let note_path = self.path.parent().unwrap().join(format!("{}.md", note_name));
        
        // Create the note if it doesn't exist
        if !note_path.exists() {
            std::fs::write(&note_path, "")?;
        }
        
        // Open the note
        cx.emit(EditorEvent::OpenPath(note_path));
        Ok(())
    }
    
    pub fn sync_preview(&mut self, cx: &mut ViewContext<Self>) -> Task<Result<()>> {
        if let Some(preview) = &self.preview {
            preview.update(cx, |preview, cx| preview.update_preview(cx))
        } else {
            Task::ready(Ok(()))
        }
    }
}

impl Render for NoteEditor {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        if self.preview_mode {
            if let Some(markdown) = &self.markdown {
                div().size_full().child(markdown.clone())
            } else {
                div().size_full().child("Loading preview...")
            }
        } else {
            div().size_full().child(
                EditorElement::new(
                    &self.editor,
                    EditorStyle {
                        background: Some(gpui::rgb(0x1a1a1a)),
                        ..Default::default()
                    },
                )
                .into_any_element(),
            )
        }
    }
}