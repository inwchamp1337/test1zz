use spider::html::node::{Node, NodeRef};
use spider::html::{ElementRef, Html};

#[derive(Clone, Copy)]
enum RenderMode {
    Block,
    Inline,
}

pub fn html_to_markdown(url: &str, html: &str) -> String {
    let document = Html::parse_document(html);
    let mut buffer = String::new();

    if let Some(root) = document.root_element() {
        render_children(root, RenderMode::Block, &mut buffer);
    }

    let trimmed = buffer.trim();
    if trimmed.is_empty() {
        format!("# {}\n", url)
    } else {
        format!("{}\n", trimmed)
    }
}

fn render_children(element: ElementRef, mode: RenderMode, buffer: &mut String) {
    for child in element.children() {
        render_node(child, mode, buffer);
    }
}

fn render_node(node: NodeRef, mode: RenderMode, buffer: &mut String) {
    match node.value() {
        Node::Element(_) => {
            if let Some(element) = ElementRef::wrap(node.clone()) {
                render_element(element, buffer);
            }
        }
        Node::Text(text) => {
            let mut chunk = text.text.replace('\r', " ").replace('\n', " ");
            if mode == RenderMode::Inline {
                chunk = chunk.trim().to_string();
            } else {
                chunk = chunk.trim_matches('\n').to_string();
            }
            if chunk.is_empty() {
                return;
            }
            if matches!(mode, RenderMode::Inline) && !buffer.ends_with([' ', '\n']) {
                buffer.push(' ');
            }
            buffer.push_str(&chunk);
        }
        _ => {}
    }
}

fn render_element(element: ElementRef, buffer: &mut String) {
    let name = element.value().name();
    match name {
        "h1" => render_heading(element, buffer, "#"),
        "h2" => render_heading(element, buffer, "##"),
        "p" => render_paragraph(element, buffer),
        "strong" => render_enclosed(element, buffer, "**"),
        "em" => render_enclosed(element, buffer, "*"),
        "a" => render_link(element, buffer),
        "img" => render_image(element, buffer),
        "ul" => render_unordered_list(element, buffer),
        "ol" => render_ordered_list(element, buffer),
        "li" => render_list_item(element, buffer, "- "),
        "br" => buffer.push_str("  \n"),
        "blockquote" => render_blockquote(element, buffer),
        _ => render_children(element, RenderMode::Inline, buffer),
    }
}

fn render_heading(element: ElementRef, buffer: &mut String, marker: &str) {
    let mut inner = String::new();
    render_children(element, RenderMode::Inline, &mut inner);
    let text = inner.trim();
    if text.is_empty() {
        return;
    }
    buffer.push_str(marker);
    buffer.push(' ');
    buffer.push_str(text);
    buffer.push_str("\n\n");
}

fn render_paragraph(element: ElementRef, buffer: &mut String) {
    let mut inner = String::new();
    render_children(element, RenderMode::Inline, &mut inner);
    let text = inner.trim();
    if text.is_empty() {
        return;
    }
    buffer.push_str(text);
    buffer.push_str("\n\n");
}

fn render_enclosed(element: ElementRef, buffer: &mut String, mark: &str) {
    buffer.push_str(mark);
    render_children(element, RenderMode::Inline, buffer);
    buffer.push_str(mark);
}

fn render_link(element: ElementRef, buffer: &mut String) {
    let mut inner = String::new();
    render_children(element, RenderMode::Inline, &mut inner);
    let text = inner.trim();
    if text.is_empty() {
        return;
    }
    let href = element.value().attr("href").unwrap_or("#");
    buffer.push('[');
    buffer.push_str(text);
    buffer.push_str("](");
    buffer.push_str(href);
    buffer.push(')');
}

fn render_image(element: ElementRef, buffer: &mut String) {
    let alt = element.value().attr("alt").unwrap_or_default();
    let src = element.value().attr("src").unwrap_or_default();
    if src.is_empty() {
        return;
    }
    buffer.push_str("![");
    buffer.push_str(alt);
    buffer.push_str("](");
    buffer.push_str(src);
    buffer.push(')');
}

fn render_unordered_list(element: ElementRef, buffer: &mut String) {
    for child in element.children() {
        if let Some(li) = ElementRef::wrap(child) {
            render_list_item(li, buffer, "- ");
        }
    }
    buffer.push('\n');
}

fn render_ordered_list(element: ElementRef, buffer: &mut String) {
    let mut index = 1;
    for child in element.children() {
        if let Some(li) = ElementRef::wrap(child) {
            let prefix = format!("{}. ", index);
            render_list_item(li, buffer, &prefix);
            index += 1;
        }
    }
    buffer.push('\n');
}

fn render_list_item(element: ElementRef, buffer: &mut String, prefix: &str) {
    let mut inner = String::new();
    render_children(element, RenderMode::Inline, &mut inner);
    let text = inner.trim();
    if text.is_empty() {
        return;
    }
    buffer.push_str(prefix);
    buffer.push_str(text);
    buffer.push('\n');
}

fn render_blockquote(element: ElementRef, buffer: &mut String) {
    let mut inner = String::new();
    render_children(element, RenderMode::Inline, &mut inner);
    let text = inner.trim();
    if text.is_empty() {
        return;
    }
    for line in text.lines() {
        buffer.push_str("> ");
        buffer.push_str(line.trim());
        buffer.push('\n');
    }
    buffer.push('\n');
}