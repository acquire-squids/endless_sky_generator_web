pub mod chaos;
pub mod full_map;
pub mod system_shuffler;

use crate::import_from_javascript;
use crate::zippy::Zip;

use endless_sky_rw::{
    self, Data, DataFolder, Node, NodeIndex, SourceIndex, Span, Token, TokenKind,
};

use std::{error::Error, io, path::PathBuf};

fn zip_root_nodes<P: Into<PathBuf>>(
    archive: &mut Zip,
    path: P,
    data: &Data,
    root_nodes: &[(SourceIndex, NodeIndex)],
) -> Result<(), Box<dyn Error>> {
    let path = P::into(path);

    let mut text = String::new();

    if data.write_root_nodes(&mut text, root_nodes).is_err() {
        return Err(Box::new(io::Error::other(format!(
            "Failed to write `{}` to string :(",
            path.display()
        ))));
    }

    archive.write_file(path, text.trim().as_bytes())?;

    Ok(())
}

fn read_upload(paths: Vec<String>, sources: Vec<String>) -> Result<DataFolder, Box<dyn Error>> {
    match endless_sky_rw::read_upload(paths, sources) {
        Some((data_folder, errors)) => {
            if !errors.is_empty() {
                let error_string = String::from_utf8(errors)?;

                import_from_javascript::error(error_string.as_str());
            }

            Ok(data_folder)
        }
        None => {
            Err(Box::new(
                io::Error::other(
                    "ERROR: Somehow, everything went wrong while reading the data folder. You're on your own.".to_owned()
                )
            ))
        }
    }
}

fn copy_node(
    data: &Data,
    (source_index, node_index): (SourceIndex, NodeIndex),
    output_data: &mut Data,
    output_source: SourceIndex,
    disallowed_children: &[&str],
) -> Option<NodeIndex> {
    let tokens = data.get_tokens(node_index)?;

    if tokens.is_empty() {
        return None;
    }

    let output_node = output_data.insert_node(Node::Some { tokens: vec![] });

    for token in tokens {
        if let Some(lexeme) = data.get_lexeme(source_index, *token)
            && let Some((span_start, span_end)) = output_data.push_source(output_source, lexeme)
        {
            output_data.push_token(
                output_node,
                Token::new(TokenKind::Symbol, Span::new(span_start, span_end)),
            );
        }
    }

    if let Some(children) = data.get_children(node_index) {
        for child in children {
            if let Some(lexeme) = data
                .get_tokens(*child)
                .and_then(|tokens| tokens.first())
                .and_then(|t| data.get_lexeme(source_index, *t))
                && !disallowed_children.contains(&lexeme)
                && let Some(output_child) = copy_node(
                    data,
                    (source_index, *child),
                    output_data,
                    output_source,
                    disallowed_children,
                )
            {
                output_data.push_child(output_node, output_child);
            }
        }
    }

    Some(output_node)
}
