use aether_syntax::AnyRSelector;
use aether_syntax::RIdentifier;
use aether_syntax::RStringValue;
use biome_rowan::AstNode;

// Candidates for upstreaming into `aether_syntax`.

pub trait RIdentifierExt {
    /// Return the symbol name, stripping backtick quoting if present.
    ///
    /// Backtick-quoted identifiers like `` `my var` `` are parsed as
    /// `RIdentifier` nodes whose `text_trimmed()` includes the backticks.
    /// The backticks are a quoting mechanism, not part of the symbol name.
    fn name_text(&self) -> String;
}

impl RIdentifierExt for RIdentifier {
    fn name_text(&self) -> String {
        let text = self.syntax().text_trimmed().to_string();
        match text.strip_prefix('`').and_then(|s| s.strip_suffix('`')) {
            Some(inner) => inner.to_string(),
            None => text,
        }
    }
}

pub trait RStringValueExt {
    /// Return the string contents without surrounding quotes.
    fn string_text(&self) -> Option<String>;
}

impl RStringValueExt for RStringValue {
    fn string_text(&self) -> Option<String> {
        // The content token is absent for empty strings like `""`.
        match self.content_token() {
            Some(token) => Some(token.text_trimmed().to_string()),
            None => Some(String::new()),
        }
    }
}

pub trait AnyRSelectorExt {
    /// Uses [RIdentifierExt::name_text()] and [RStringValueExt::string_text()] to report
    /// the name without backticks or quotes
    ///
    /// Returns [None] for [aether_syntax::AnyRSelector::RDotDotI] and
    /// [aether_syntax::AnyRSelector::RDots], which we don't consider standard
    /// identifiers.
    fn identifier_text(&self) -> Option<String>;
}

impl AnyRSelectorExt for AnyRSelector {
    fn identifier_text(&self) -> Option<String> {
        match self {
            AnyRSelector::RIdentifier(node) => Some(node.name_text()),
            AnyRSelector::RStringValue(node) => node.string_text(),
            AnyRSelector::RDots(_) | AnyRSelector::RDotDotI(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use aether_parser::RParserOptions;
    use aether_syntax::AnyRExpression;
    use aether_syntax::AnyRValue;
    use assert_matches::assert_matches;
    use biome_rowan::AstNodeList;

    use super::*;

    fn parse_single_expr(code: &str) -> AnyRExpression {
        let parsed = aether_parser::parse(code, RParserOptions::default());
        parsed.tree().expressions().iter().next().unwrap()
    }

    #[test]
    fn identifier_plain() {
        assert_matches!(parse_single_expr("foo"), AnyRExpression::RIdentifier(ident) => {
            assert_eq!(ident.name_text(), "foo");
        });
    }

    #[test]
    fn identifier_backtick_quoted() {
        assert_matches!(parse_single_expr("`my var`"), AnyRExpression::RIdentifier(ident) => {
            assert_eq!(ident.name_text(), "my var");
        });
    }

    #[test]
    fn string_double_quoted() {
        assert_matches!(parse_single_expr("\"hello\""), AnyRExpression::AnyRValue(AnyRValue::RStringValue(s)) => {
            assert_eq!(s.string_text().unwrap(), "hello");
        });
    }

    #[test]
    fn string_single_quoted() {
        assert_matches!(parse_single_expr("'world'"), AnyRExpression::AnyRValue(AnyRValue::RStringValue(s)) => {
            assert_eq!(s.string_text().unwrap(), "world");
        });
    }
}
