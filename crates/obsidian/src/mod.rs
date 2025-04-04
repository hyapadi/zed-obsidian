mod graph_view;
mod markdown_preview;
mod note_editor;

pub use graph_view::GraphView;
pub use markdown_preview::ObsidianMarkdownPreview;
pub use note_editor::NoteEditor;

use anyhow::Result;
use fs::fs_watcher;
use gpui::{AppContext, Context, Model, ModelContext, Task, View, ViewContext, WindowContext};
use project::Project;
use std::{path::PathBuf, sync::Arc};
use workspace::Workspace;

pub struct ObsidianWorkspace {
    workspace: Model<Workspace>,
    project: Model<Project>,
    graph_view: Option<Model<GraphView>>,
    settings: ObsidianSettings,
    _fs_watcher: Option<Arc<fs_watcher::FsWatcher>>,
}

#[derive(Debug, Default)]
pub struct ObsidianSettings {
    vault_path: PathBuf,
    enable_wiki_links: bool,
    enable_graph_view: bool,
}

impl ObsidianWorkspace {
    pub fn new(
        workspace: Model<Workspace>,
        project: Model<Project>,
        settings: ObsidianSettings,
        cx: &mut ModelContext<Self>,
    ) -> Self {
        // Initialize file system watcher
        let _fs_watcher = fs_watcher::global(|_| {}).ok();
        Self {
            workspace,
            project,
            graph_view: None,
            settings,
            _fs_watcher: None,
        }
    }

    pub fn toggle_graph_view(&mut self, cx: &mut ModelContext<Self>) {
        if self.settings.enable_graph_view {
            if self.graph_view.is_none() {
                // Initialize graph view
                let graph = petgraph::Graph::new();
                let graph_view = GraphView::new(graph);
                self.graph_view = Some(cx.new_model(|_| graph_view));
            }

            if let Some(graph_view) = &self.graph_view {
                self.workspace.update(cx, |workspace, cx| {
                    workspace.add_item(graph_view.clone(), None);
                });
            }
        }
    }

    pub fn open_note(&mut self, path: PathBuf, cx: &mut ModelContext<Self>) -> Task<Result<()>> {
        cx.spawn(|this, mut cx| async move {
            let file = fs::File::open(&path).await?;
            let editor = editor::Editor::new(file, cx.clone());
            let note_editor = NoteEditor::new(editor, path.clone(), &mut cx);
            
            this.update(&mut cx, |this, cx| {
                this.workspace.update(cx, |workspace, cx| {
                    workspace.add_item(note_editor, None);
                });
            })?;
            Ok(())
        })
    }
}