use std::borrow::Cow;

use super::{DialogueBoxToken, TextColor, TextCommand, TextEffect, TextSection};
use winnow::{
    ascii::float,
    combinator::{alt, delimited, opt, peek, terminated},
    error::{ContextError, ParseError},
    token::{any, take_till, take_while},
    PResult, Parser,
};

const _EXAMPLES: &str = "<1.2> But you're a `big FLOWER|red`[wave]!";

fn parse_speed(input: &mut &str) -> PResult<f32> {
    delimited('<', float, '>').parse_next(input)
}

fn parse_pause(input: &mut &str) -> PResult<f32> {
    delimited('[', float, ']').parse_next(input)
}

fn parse_effect(input: &mut &str) -> PResult<TextEffect> {
    alt(("wave".map(|_| TextEffect::Wave),)).parse_next(input)
}

fn parse_color(input: &mut &str) -> PResult<TextColor> {
    alt((
        "red".map(|_| TextColor::Red),
        "green".map(|_| TextColor::Green),
        "blue".map(|_| TextColor::Blue),
    ))
    .parse_next(input)
}

fn parse_ticks(input: &mut &str) -> PResult<TextSection> {
    '`'.parse_next(input)?;
    let text = take_till(0.., ['|', '`']).parse_next(input)?;

    let color = match any.parse_next(input)? {
        '|' => {
            let color = terminated(parse_color, '`').parse_next(input)?;
            Some(color)
        }
        _ => None,
    };

    let effect = opt(delimited('[', parse_effect, ']')).parse_next(input)?;

    Ok(TextSection {
        text: Cow::Owned(text.into()),
        color,
        effect,
    })
}

#[cfg(feature = "proc-macro")]
use quote::{quote, TokenStreamExt};

#[derive(Debug)]
pub enum TokenGroup {
    Bare(DialogueBoxToken),
    Group(Vec<TokenGroup>),
}

impl TokenGroup {
    /// Returns whether a slice of [TokenGroup] is all [TokenGroup::Bare].
    pub fn bare(items: &[TokenGroup]) -> bool {
        items.iter().all(|i| matches!(i, TokenGroup::Bare(_)))
    }
}

pub fn parse_groups(input: &str) -> Result<Vec<TokenGroup>, ParseError<&str, ContextError>> {
    let mut tokens = parse_tokens.parse(&input)?;
    process_nodes(&mut tokens, false);

    Ok(tokens)
}

fn parse_tokens(input: &mut &str) -> PResult<Vec<TokenGroup>> {
    let mut result = Vec::new();

    while let Ok(text) = parse_normal(input) {
        let token = DialogueBoxToken::Section(TextSection::from(text.to_owned()));

        if !text.is_empty() {
            result.push(TokenGroup::Bare(token));
        }

        if let Some(tok) = peek(any::<_, ()>).parse_next(input).ok() {
            let item = match tok {
                '<' => {
                    let speed = parse_speed(input)?;
                    TokenGroup::Bare(DialogueBoxToken::Command(TextCommand::Speed(speed)))
                }
                '[' => {
                    let speed = parse_pause(input)?;
                    TokenGroup::Bare(DialogueBoxToken::Command(TextCommand::Pause(speed)))
                }
                '`' => parse_ticks
                    .map(|s| TokenGroup::Bare(DialogueBoxToken::Section(s)))
                    .parse_next(input)?,
                '{' => {
                    any.parse_next(input)?;
                    parse_tokens.map(TokenGroup::Group).parse_next(input)?
                }
                _ => {
                    any.parse_next(input)?;
                    break;
                }
            };

            result.push(item);
        } else {
            break;
        }
    }

    Ok(result)
}

fn de_duplicate_spaces(text: &str, prev_was_space: bool) -> (String, bool) {
    let mut result = String::with_capacity(text.len());
    let mut last_char_was_space = prev_was_space;
    for c in text.chars() {
        if c.is_whitespace() {
            if !last_char_was_space {
                result.push(' ');
                last_char_was_space = true;
            }
        } else {
            result.push(c);
            last_char_was_space = false;
        }
    }
    (result, last_char_was_space)
}

fn process_nodes(nodes: &mut Vec<TokenGroup>, mut prev_was_space: bool) {
    let mut i = 0;
    while i < nodes.len() {
        match &mut nodes[i] {
            TokenGroup::Bare(content) => {
                match content {
                    DialogueBoxToken::Section(text_section) => {
                        let (new_text, last_char_was_space) =
                            de_duplicate_spaces(&text_section.text, prev_was_space);
                        if new_text.is_empty() {
                            // Remove the node if the text is empty
                            nodes.remove(i);
                            continue;
                        } else {
                            text_section.text = new_text.into();
                            prev_was_space = last_char_was_space;
                        }
                    }
                    _ => {}
                }
            }
            TokenGroup::Group(group_nodes) => {
                process_nodes(group_nodes, prev_was_space);
                if group_nodes.is_empty() {
                    // Remove the group if it's empty
                    nodes.remove(i);
                    continue;
                } else {
                    // Reset prev_was_space after processing a group
                    prev_was_space = false;
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "Hello, world!";

        let mut output = parse_groups(input).unwrap();
        let first = output.remove(0);

        assert!(matches!(first, TokenGroup::Bare(_)));
    }

    #[test]
    fn test_nested() {
        let input = "Hello, <0.5> {`world|red`[wave]}!";

        let mut output = parse_groups(input).unwrap();

        process_nodes(&mut output, false);

        panic!("{:#?}", output);

        assert!(matches!(output.as_slice(),
                &[
                    TokenGroup::Bare(_),
                    TokenGroup::Group(ref n),
                    TokenGroup::Bare(_),
                ] if n.len() == 1
        ));
    }
}

fn parse_normal<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_while(0.., |c| !['[', '<', '`', '{', '}'].contains(&c)).parse_next(input)
}

#[cfg(feature = "proc-macro")]
impl quote::ToTokens for &'_ super::TextEffect {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(quote! { bevy_bits::TextEffect:: });
        tokens.append_all(match self {
            TextEffect::Wave => quote! { Wave },
        });
    }
}

#[cfg(feature = "proc-macro")]
impl quote::ToTokens for &'_ super::TextColor {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TextColor::Red => tokens.append_all(quote! { bevy_bits::TextColor::Red }),
            TextColor::Green => tokens.append_all(quote! { bevy_bits::TextColor::Green }),
            TextColor::Blue => tokens.append_all(quote! { bevy_bits::TextColor::Blue }),
        }
    }
}

#[cfg(feature = "proc-macro")]
impl quote::ToTokens for DialogueBoxToken {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            DialogueBoxToken::Section(section) => {
                let text = &section.text;

                let color = match &section.color {
                    Some(c) => quote! { Some(#c) },
                    None => quote! { None },
                };

                let effect = match &section.effect {
                    Some(e) => quote! { Some(#e) },
                    None => quote! { None },
                };

                tokens.append_all(quote! {
                    bevy_bits::DialogueBoxToken::Section(
                        bevy_bits::tokens::TextSection {
                            text: std::borrow::Cow::Borrowed(#text),
                            color: #color,
                            effect: #effect,
                        }
                    )
                });
            }
            DialogueBoxToken::Command(cmd) => match &cmd {
                TextCommand::Clear => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::Clear) },
                ),
                TextCommand::AwaitClear => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::AwaitClear) },
                ),
                TextCommand::ClearAfter(dur) => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::ClearAfter(#dur)) },
                ),
                TextCommand::Speed(speed) => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::Speed(#speed)) },
                ),
                TextCommand::Pause(pause) => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::Pause(#pause)) },
                ),
                TextCommand::Delete(num) => tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Command(bevy_bits::tokens::TextCommand::Delete(#num)) },
                ),
            },
            DialogueBoxToken::Sequence(seq) => {
                 tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Sequence(std::borrow::Cow::Borrowed(&[#(#seq),*])) },
                );
            },
        }
    }
}
