pub mod chaos;
pub mod full_map;
pub mod system_shuffler;

use crate::zippy::Zip;

use endless_sky_rw::{self, Data, Node, NodeIndex, SourceIndex, Span, Token, TokenKind};

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

fn copy_node(
    data: &Data,
    (source_index, node_index): (SourceIndex, NodeIndex),
    output_data: &mut Data,
    output_source: SourceIndex,
    disallowed_children: &[&str],
) -> Option<NodeIndex> {
    copy_node_allow_or_deny(
        data,
        (source_index, node_index),
        output_data,
        output_source,
        disallowed_children,
        false,
    )
}

#[allow(dead_code)]
fn copy_node_allow(
    data: &Data,
    (source_index, node_index): (SourceIndex, NodeIndex),
    output_data: &mut Data,
    output_source: SourceIndex,
    allowed_children: &[&str],
) -> Option<NodeIndex> {
    copy_node_allow_or_deny(
        data,
        (source_index, node_index),
        output_data,
        output_source,
        allowed_children,
        true,
    )
}

fn copy_node_allow_or_deny(
    data: &Data,
    (source_index, node_index): (SourceIndex, NodeIndex),
    output_data: &mut Data,
    output_source: SourceIndex,
    child_list: &[&str],
    allow: bool,
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
                && ((allow && child_list.contains(&lexeme))
                    || (!allow && !child_list.contains(&lexeme)))
                && let Some(output_child) = copy_node_allow_or_deny(
                    data,
                    (source_index, *child),
                    output_data,
                    output_source,
                    child_list,
                    allow,
                )
            {
                output_data.push_child(output_node, output_child);
            }
        }
    }

    Some(output_node)
}
