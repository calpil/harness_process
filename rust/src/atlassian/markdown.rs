//! Markdown -> `storage` de Confluence (decision OBS-10).
//!
//! Subconjunto ACOTADO a proposito: titulos, parrafos, listas, tablas, bloques
//! de codigo, enlaces, `codigo en linea` y **negrita**. Lo que no entra en ese
//! subconjunto sobrevive como texto plano — nunca se pierde — y cada pagina
//! publicada lleva el enlace al archivo del repo, que es la fuente de verdad.

/// Escapa lo que rompe XHTML.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Convierte los marcadores en linea sobre texto YA escapado.
fn inline(text: &str) -> String {
    let mut out = escape(text);
    out = replace_pairs(&out, "**", "<strong>", "</strong>");
    out = replace_pairs(&out, "`", "<code>", "</code>");
    out = links(&out);
    out
}

/// Reemplaza pares de delimitadores (`**texto**`, `` `texto` ``). Un
/// delimitador impar se deja como esta.
fn replace_pairs(text: &str, delim: &str, open: &str, close: &str) -> String {
    let parts: Vec<&str> = text.split(delim).collect();
    if parts.len() < 3 {
        return text.to_string();
    }
    let mut out = String::new();
    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            out.push_str(part);
            continue;
        }
        // Dentro de un par cerrado: i impar abre, i par cierra.
        if i % 2 == 1 {
            if i + 1 < parts.len() {
                out.push_str(open);
                out.push_str(part);
            } else {
                // Delimitador sin cierre: se restituye literal.
                out.push_str(delim);
                out.push_str(part);
            }
        } else {
            out.push_str(close);
            out.push_str(part);
        }
    }
    out
}

/// `[texto](url)` -> `<a href="url">texto</a>`.
fn links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' {
            if let Some((label, url, next)) = parse_link(&chars, i) {
                out.push_str(&format!("<a href=\"{url}\">{label}</a>"));
                i = next;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

fn parse_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    let close = chars[start..].iter().position(|c| *c == ']')? + start;
    if chars.get(close + 1) != Some(&'(') {
        return None;
    }
    let end = chars[close + 2..].iter().position(|c| *c == ')')? + close + 2;
    let label: String = chars[start + 1..close].iter().collect();
    let url: String = chars[close + 2..end].iter().collect();
    if url.contains(' ') {
        return None;
    }
    Some((label, url, end + 1))
}

/// Celdas de una fila de tabla markdown.
fn row_cells(line: &str) -> Vec<String> {
    line.trim()
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().to_string())
        .collect()
}

fn is_separator_row(line: &str) -> bool {
    let t = line.trim();
    t.starts_with('|')
        && t.chars()
            .all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t'))
        && t.contains('-')
}

/// Markdown -> storage. `source_note` es el enlace al archivo del repo que se
/// agrega al pie como fuente de verdad.
pub fn to_storage(markdown: &str, source_note: Option<&str>) -> String {
    let mut out = String::new();
    let lines: Vec<&str> = markdown.lines().collect();
    let mut i = 0;
    let mut list: Option<&'static str> = None;

    // Cierra la lista abierta, si hay.
    fn close_list(out: &mut String, list: &mut Option<&'static str>) {
        if let Some(tag) = list.take() {
            out.push_str(&format!("</{tag}>"));
        }
    }

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        // Bloque de codigo cercado.
        if let Some(lang) = trimmed.strip_prefix("```") {
            close_list(&mut out, &mut list);
            let mut body = String::new();
            i += 1;
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                body.push_str(lines[i]);
                body.push('\n');
                i += 1;
            }
            i += 1; // cierre
            let lang = lang.trim();
            let lang_param = if lang.is_empty() { "text" } else { lang };
            out.push_str(&format!(
                "<ac:structured-macro ac:name=\"code\"><ac:parameter ac:name=\"language\">{}</ac:parameter><ac:plain-text-body><![CDATA[{}]]></ac:plain-text-body></ac:structured-macro>",
                escape(lang_param),
                // CDATA no admite la secuencia de cierre dentro del cuerpo.
                body.replace("]]>", "]] >")
            ));
            continue;
        }

        // Tabla.
        if trimmed.starts_with('|') && i + 1 < lines.len() && is_separator_row(lines[i + 1]) {
            close_list(&mut out, &mut list);
            let headers = row_cells(trimmed);
            out.push_str("<table><tbody><tr>");
            for h in &headers {
                out.push_str(&format!("<th>{}</th>", inline(h)));
            }
            out.push_str("</tr>");
            i += 2;
            while i < lines.len() && lines[i].trim().starts_with('|') {
                out.push_str("<tr>");
                for c in row_cells(lines[i]) {
                    out.push_str(&format!("<td>{}</td>", inline(&c)));
                }
                out.push_str("</tr>");
                i += 1;
            }
            out.push_str("</tbody></table>");
            continue;
        }

        // Titulos.
        if let Some(rest) = trimmed.strip_prefix('#') {
            let level = 1 + rest.chars().take_while(|c| *c == '#').count();
            let title = rest.trim_start_matches('#').trim();
            if level <= 6 && !title.is_empty() {
                close_list(&mut out, &mut list);
                out.push_str(&format!("<h{level}>{}</h{level}>", inline(title)));
                i += 1;
                continue;
            }
        }

        // Listas.
        let bullet = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "));
        if let Some(item) = bullet {
            if list != Some("ul") {
                close_list(&mut out, &mut list);
                out.push_str("<ul>");
                list = Some("ul");
            }
            out.push_str(&format!("<li>{}</li>", inline(item)));
            i += 1;
            continue;
        }
        if let Some(item) = ordered_item(trimmed) {
            if list != Some("ol") {
                close_list(&mut out, &mut list);
                out.push_str("<ol>");
                list = Some("ol");
            }
            out.push_str(&format!("<li>{}</li>", inline(item)));
            i += 1;
            continue;
        }

        // Linea en blanco: cierra listas.
        if trimmed.is_empty() {
            close_list(&mut out, &mut list);
            i += 1;
            continue;
        }

        // Parrafo: junta lineas contiguas.
        close_list(&mut out, &mut list);
        let mut para = String::new();
        while i < lines.len() {
            let l = lines[i].trim();
            if l.is_empty()
                || l.starts_with('#')
                || l.starts_with("- ")
                || l.starts_with("* ")
                || l.starts_with("```")
                || l.starts_with('|')
                || ordered_item(l).is_some()
            {
                break;
            }
            if !para.is_empty() {
                para.push(' ');
            }
            para.push_str(l);
            i += 1;
        }
        if !para.is_empty() {
            out.push_str(&format!("<p>{}</p>", inline(&para)));
        }
    }
    close_list(&mut out, &mut list);

    if let Some(note) = source_note {
        out.push_str(&format!(
            "<hr/><p><em>{}</em></p>",
            inline(note)
        ));
    }
    out
}

/// `1. texto` -> `texto`.
fn ordered_item(line: &str) -> Option<&str> {
    let digits: String = line.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    line[digits.len()..].strip_prefix(". ")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn should_convert_headings_and_paragraphs() {
        let html = to_storage("# Titulo\n\nUn parrafo\nque sigue.\n", None);
        assert!(html.contains("<h1>Titulo</h1>"));
        assert!(html.contains("<p>Un parrafo que sigue.</p>"));
    }

    #[test]
    fn should_convert_lists() {
        let html = to_storage("- uno\n- dos\n\n1. primero\n2. segundo\n", None);
        assert!(html.contains("<ul><li>uno</li><li>dos</li></ul>"));
        assert!(html.contains("<ol><li>primero</li><li>segundo</li></ol>"));
    }

    #[test]
    fn should_convert_code_blocks_with_language() {
        let html = to_storage("```rust\nlet x = 1 < 2;\n```\n", None);
        assert!(html.contains("ac:name=\"code\""));
        assert!(html.contains("<ac:parameter ac:name=\"language\">rust</ac:parameter>"));
        // Dentro de CDATA el codigo viaja literal (sin escapar).
        assert!(html.contains("let x = 1 < 2;"));
    }

    #[test]
    fn should_convert_tables() {
        let md = "| A | B |\n| --- | --- |\n| 1 | 2 |\n";
        let html = to_storage(md, None);
        assert!(html.contains("<th>A</th><th>B</th>"));
        assert!(html.contains("<td>1</td><td>2</td>"));
    }

    #[test]
    fn should_escape_html_and_convert_inline_marks() {
        let html = to_storage("Un <script> con **negrita**, `codigo` y [link](https://x.cl).\n", None);
        assert!(html.contains("&lt;script&gt;"), "el HTML crudo se escapa");
        assert!(html.contains("<strong>negrita</strong>"));
        assert!(html.contains("<code>codigo</code>"));
        assert!(html.contains("<a href=\"https://x.cl\">link</a>"));
    }

    #[test]
    fn should_keep_unpaired_delimiters_literal() {
        let html = to_storage("El precio es 5 ** 2 y nada mas\n", None);
        assert!(html.contains("5 ** 2"), "un delimitador impar no abre nada");
    }

    #[test]
    fn should_append_source_note() {
        let html = to_storage("# T\n", Some("Fuente: docs/prd/PRD-master.md"));
        assert!(html.contains("<hr/>"));
        assert!(html.contains("Fuente: docs/prd/PRD-master.md"));
    }

    #[test]
    fn should_not_break_cdata_with_closing_sequence() {
        let html = to_storage("```\ntexto ]]> raro\n```\n", None);
        assert!(!html.contains("]]> raro"));
        assert!(html.contains("]] > raro"));
    }
}
