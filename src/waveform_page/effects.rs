use iced::widget::text_editor::Content;

use super::{ast::Statement, sources::DataSource, WaveformPage};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub enum EditEffects {
    // selection interval is stored in page
    MouseEdit,
    Delete, // delete selection
    Transfer {
        // transfer data from other source
        from: usize,                    // index of source in main app array
        from_selection: (usize, usize), // copied data replaces the current page selection
    },
    Paste(Vec<i16>),
    Loop(usize), // repeates selected pattern directly after it
    #[default]
    Unset,
}

impl EditEffects {
    pub fn data(&self, parent: &WaveformPage) -> Vec<i16> {
        match self {
            EditEffects::MouseEdit => parent.affected_data.clone(),
            EditEffects::Delete => Vec::new(),
            EditEffects::Transfer { .. } => {
                unimplemented!("need to figure out the inter-node stuff")
            }
            EditEffects::Paste(x) => x.clone(),
            EditEffects::Loop(x) => {
                let sample = parent.data[parent.selection.0..parent.selection.1].to_vec();
                let len = sample.len() * x;
                sample.into_iter().cycle().take(len).collect()
            }
            EditEffects::Unset => parent.data[parent.selection.0..parent.selection.1].to_vec(),
        }
    }
}
