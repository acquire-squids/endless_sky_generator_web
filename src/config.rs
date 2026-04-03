use std::collections::HashMap;

#[macro_export]
macro_rules! __parse_config {
    (
        $source:expr => $settings_ty:ty;
        $($option:ident => { $($t:tt)+ })+
    ) => {
        $crate::config::parse($source).and_then(|config| Some(<$settings_ty>::new(
            $(
                match config.options().get(stringify!($option)) {
                    Some($option) => $crate::config::config_option!(
                        $option as $($t)+
                    ),
                    None => {
                        eprintln!("{} was never set and is required!", stringify!($option));
                        None
                    },
                }?,
            )+
        )))
    };
}

#[macro_export]
macro_rules! __config_option {
    (
        $option:ident as int of $int_ty:ty
            $(where $predicate:expr)?
            => $body:expr
    ) => {
        match $option {
            Value::DecimalInteger(lexeme) => match lexeme.parse::<$int_ty>() {
                Ok($option) $(if $predicate)? => Some($body),
                Ok(_) => {
                    eprintln!("{}'s value does not match its constraints!", stringify!($option));
                    None
                }
                Err(error) => {
                    eprintln!("{error}");
                    eprintln!("{} is not a valid {}!", stringify!($option), stringify!($int_ty));
                    None
                }
            }
            _ => {
                eprintln!("{} should be an int of type {}, but it is not!", stringify!($option), stringify!($int_ty));
                None
            }
        }
    };

    (
        $option:ident as float of $float_ty:ty
            $(where $predicate:expr)?
            => $body:expr
    ) => {
        match $option {
            Value::Float(lexeme) => match lexeme.parse::<$float_ty>() {
                Ok($option) $(if $predicate)? => Some($body),
                Ok(_) => {
                    eprintln!("{}'s value does not match its constraints!", stringify!($option));
                    None
                }
                Err(error) => {
                    eprintln!("{error}");
                    eprintln!("{} is not a valid {}!", stringify!($option), stringify!($float_ty));
                    None
                }
            }
            _ => {
                eprintln!("{} should be an float of type {}, but it is not!", stringify!($option), stringify!($float_ty));
                None
            }
        }
    };

    (
        $option:ident as string
            $(where $predicate:expr)?
            => $body:expr
    ) => {
        match $option {
            Value::String($option) $(if $predicate)? => Some($body),
            Value::String(_) => {
                eprintln!("{}'s value does not match its constraints!", stringify!($option));
                None
            }
            _ => {
                eprintln!("{} should be a string, but it is not!", stringify!($option));
                None
            }
        }
    };

    (
        $option:ident as bool
            $(where $predicate:expr)?
            => $body:expr
    ) => {
        match $option {
            Value::Boolean($option) $(if $predicate)? => Some($body),
            Value::Boolean(_) => {
                eprintln!("{}'s value does not match its constraints!", stringify!($option));
                None
            }
            _ => {
                eprintln!("{} should be a boolean, but it is not!", stringify!($option));
                None
            }
        }
    };

    (
        $option:ident as list
            $(where $predicate:expr)?
            => $body:expr
    ) => {
        match $option {
            Value::List($option) $(if $predicate)? => Some($body),
            Value::List(_) => {
                eprintln!("{}'s value does not match its constraints!", stringify!($option));
                None
            }
            _ => {
                eprintln!("{} should be a list, but it is not!", stringify!($option));
                None
            }
        }
    };
}

pub use __config_option as config_option;

pub use __parse_config as parse_config;

#[must_use]
pub fn parse(source: &str) -> Option<Config<'_>> {
    let mut config = Config {
        options: HashMap::new(),
    };

    let tokens = tokenize(source);

    let mut at = 0;

    while at < tokens.len() {
        if let Some(name) = tokens.get(at) {
            at += 1;

            if matches!(name.kind, TokenKind::String)
                && let Some(token) = tokens.get(at)
                && matches!(token.kind, TokenKind::Equal)
            {
                at += 1;

                match value(&tokens, &mut at) {
                    Some(value) => {
                        config.options.insert(name.lexeme, value);
                    }
                    None => return None,
                }
            } else {
                eprintln!("Invalid config!");

                let values = "any of: a decimal integer, a string, a list, a 64-bit float, \"true\", \"false\"";

                eprintln!("Expected a string, followed by an equal sign, followed by {values}");

                eprintln!("Lists look \"(like this)\" and can contain {values}");

                return None;
            }
        }
    }

    Some(config)
}

#[must_use]
pub fn key_value_list<'a, 'b>(list: &'b Value<'a>) -> Option<HashMap<&'a str, &'b Value<'a>>> {
    if let Value::List(values) = list {
        let mut table = HashMap::new();

        for value in values {
            if let Value::List(pair) = value
                && pair.len() == 2
                && let Some(Value::String(key)) = pair.first()
                && let Some(value) = pair.get(1)
            {
                table.insert(*key, value);
            }
        }

        Some(table)
    } else {
        None
    }
}

fn value<'a>(tokens: &[Token<'a>], at: &mut usize) -> Option<Value<'a>> {
    if let Some(token) = tokens.get(*at) {
        *at += 1;

        match token.kind {
            TokenKind::DecimalInteger => Some(Value::DecimalInteger(token.lexeme)),
            TokenKind::Float => Some(Value::Float(token.lexeme)),
            TokenKind::String => Some(Value::String(token.lexeme)),
            TokenKind::True => Some(Value::Boolean(true)),
            TokenKind::False => Some(Value::Boolean(false)),
            TokenKind::CloseParenthesis => {
                eprintln!("Invalid config!");

                eprintln!("A list was ended before it started!");

                None
            }
            TokenKind::Equal => {
                eprintln!("Invalid config!");

                eprintln!("An equal sign was present where a value should have been!");

                None
            }
            TokenKind::OpenParenthesis => {
                let mut values = vec![];

                while let Some(token) = tokens.get(*at) {
                    if matches!(token.kind, TokenKind::CloseParenthesis) {
                        *at += 1;
                        break;
                    }

                    values.push(value(tokens, at)?);
                }

                Some(Value::List(values))
            }
        }
    } else {
        eprintln!("Invalid config!");
        eprintln!("There should have been a value, but there wasn't!");

        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config<'a> {
    options: HashMap<&'a str, Value<'a>>,
}

impl<'a> Config<'a> {
    #[must_use]
    pub const fn options(&self) -> &HashMap<&'a str, Value<'a>> {
        &self.options
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    DecimalInteger(&'a str),
    Float(&'a str),
    String(&'a str),
    Boolean(bool),
    List(Vec<Self>),
}

impl From<bool> for Value<'_> {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<&[Self]> for Value<'_> {
    fn from(value: &[Self]) -> Self {
        Self::List(value.to_vec())
    }
}

impl From<Vec<Self>> for Value<'_> {
    fn from(value: Vec<Self>) -> Self {
        Self::List(value)
    }
}

#[must_use]
pub fn tokenize(source: &str) -> Vec<Token<'_>> {
    let mut tokens = vec![];
    let mut byte_offset = 0;

    while let Some(ch) = source.char_at(byte_offset) {
        let start = byte_offset;

        byte_offset += ch.len_utf8();

        match ch {
            '(' => tokens.push(Token {
                lexeme: &source[start..byte_offset],
                kind: TokenKind::OpenParenthesis,
            }),
            ')' => tokens.push(Token {
                lexeme: &source[start..byte_offset],
                kind: TokenKind::CloseParenthesis,
            }),
            '=' => tokens.push(Token {
                lexeme: &source[start..byte_offset],
                kind: TokenKind::Equal,
            }),
            _ if ch.is_whitespace() => {}
            _ if ch.is_ascii_digit() || ch == '-' => {
                while let Some(n) = source.char_at(byte_offset) {
                    if !n.is_ascii_digit() {
                        break;
                    }

                    byte_offset += n.len_utf8();
                }

                if source.char_at(byte_offset) == Some('.') {
                    byte_offset += '.'.len_utf8();

                    while let Some(n) = source.char_at(byte_offset) {
                        if !n.is_ascii_digit() {
                            break;
                        }

                        byte_offset += n.len_utf8();
                    }

                    tokens.push(Token {
                        lexeme: &source[start..byte_offset],
                        kind: TokenKind::Float,
                    });
                } else {
                    tokens.push(Token {
                        lexeme: &source[start..byte_offset],
                        kind: TokenKind::DecimalInteger,
                    });
                }
            }
            '"' | '`' => {
                let start = byte_offset;

                while let Some(n) = source.char_at(byte_offset) {
                    if n == ch {
                        break;
                    }

                    byte_offset += n.len_utf8();
                }

                tokens.push(Token {
                    lexeme: &source[start..byte_offset],
                    kind: TokenKind::String,
                });

                byte_offset += ch.len_utf8();
            }
            _ => {
                while let Some(n) = source.char_at(byte_offset) {
                    if n.is_whitespace() {
                        break;
                    }

                    byte_offset += n.len_utf8();
                }

                tokens.push(Token {
                    lexeme: &source[start..byte_offset],
                    kind: match &source[start..byte_offset] {
                        "true" => TokenKind::True,
                        "false" => TokenKind::False,
                        _ => TokenKind::String,
                    },
                });
            }
        }
    }

    tokens
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token<'a> {
    pub lexeme: &'a str,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    DecimalInteger,
    Float,
    String,
    True,
    False,
    OpenParenthesis,
    CloseParenthesis,
    Equal,
}

pub trait CharAt {
    fn char_at(&self, byte_offset: usize) -> Option<char>;
}

impl<T> CharAt for T
where
    T: AsRef<str>,
{
    fn char_at(&self, byte_offset: usize) -> Option<char> {
        self.as_ref()
            .get(byte_offset..)
            .and_then(|text| text.chars().next())
    }
}
