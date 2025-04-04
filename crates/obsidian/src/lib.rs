use anyhow::Result;
use editor::Editor;
use fs::File;
use gpui::{actions, AppContext, Context, Model, ModelContext, Task, View, ViewContext, WindowContext};
use language::Language;
use markdown::Markdown;
use petgraph::graph::{Graph, NodeIndex};
use project::Project;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use workspace::Workspace;

#[derive(Debug)]
pub struct ObsidianApp {
    workspace: Model<Workspace>,
    project: Model<Project>,
    note_graph: Graph<String, ()>,
    note_indices: HashMap<PathBuf, NodeIndex>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ObsidianSettings {
    vault_path: PathBuf,
    enable_wiki_links: bool,
    enable_graph_view: bool,
}

impl ObsidianApp {
    pub fn new(workspace: Model<Workspace>, project: Model<Project>, cx: &mut ModelContext<Self>) -> Self {
        Self {
            workspace,
            project,
            note_graph: Graph::new(),
            note_indices: HashMap::new(),
        }
    }

    pub fn open_note(&mut self, path: PathBuf, cx: &mut ModelContext<Self>) -> Task<Result<()>> {
        cx.spawn(|this, mut cx| async move {
            let file = File::open(&path).await?;
            let editor = Editor::new(file, cx.clone());
            this.update(&mut cx, |this, _| {
                this.workspace.update(&mut cx, |workspace, _| {
                    workspace.add_item(editor, None);
                });
            })?;
            Ok(())
        })
    }

    pub fn update_note_graph(&mut self, note_path: PathBuf, content: String) {
        // Parse content for wiki-links and update graph
        let wiki_links = self.extract_wiki_links(&content);
        let source_idx = self.get_or_create_node(&note_path);

        for link in wiki_links {
            let target_idx = self.get_or_create_node(&link);
            self.note_graph.add_edge(source_idx, target_idx, ());
        }
    }

    fn extract_wiki_links(&self, content: &str) -> Vec<PathBuf> {
        // Extract wiki-links using regex or custom parser
        // Format: [[note-name]]
        vec![] // TODO: Implement wiki-link extraction
    }

    fn get_or_create_node(&mut self, path: &PathBuf) -> NodeIndex {
        if let Some(idx) = self.note_indices.get(path) {
            *idx
        } else {
            let idx = self.note_graph.add_node(path.to_string_lossy().into_owned());
            self.note_indices.insert(path.clone(), idx);
            idx
        }
    }
}

actions!(obsidian, [OpenNote, UpdateGraph]);