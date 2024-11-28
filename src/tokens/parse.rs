use super::{DialogueBoxToken, TextColor, TextCommand, TextEffect, TextSection};
use winnow::{
    combinator::peek,
    error::{ContextError, ParseError},
    stream::Stream,
    token::{any, take_while},
    PResult, Parser,
};

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
    parse_tokens.parse(&input)
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
                '[' => parse_command.map(TokenGroup::Bare).parse_next(input)?,
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
        let input = "Hello, {[world](wave)}!";

        let output = parse_groups(input).unwrap();

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
    take_while(0.., |c| !['[', '{', '}'].contains(&c)).parse_next(input)
}

fn parse_command(input: &mut &str) -> PResult<DialogueBoxToken> {
    '['.parse_next(input)?;
    let args: Result<&str, winnow::error::ErrMode<winnow::error::ContextError>> =
        take_while(1.., |c| c != ']').parse_next(input);
    ']'.parse_next(input)?;
    '('.parse_next(input)?;
    let cmd = take_while(1.., |c| c != ')').parse_next(input)?;
    ')'.parse_next(input)?;

    Ok(DialogueBoxToken::parse_command(
        match args {
            Ok(args) => Some(args),
            Err(_) => None,
        },
        cmd,
    ))
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
                let color = if let Some(color) = &section.color {
                    quote! { #color }
                } else {
                    quote! { None }
                };
                let effects = &section.effects;

                tokens.append_all(quote! {
                    bevy_bits::DialogueBoxToken::Section(
                        bevy_bits::tokens::TextSection {
                            text: std::borrow::Cow::Borrowed(#text),
                            color: #color,
                            effects: std::borrow::Cow::Borrowed(&[#(#effects),*])
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
            },
            DialogueBoxToken::Sequence(seq) => {
                 tokens.append_all(
                    quote! { bevy_bits::DialogueBoxToken::Sequence(std::borrow::Cow::Borrowed(&[#(#seq),*])) },
                );
            },
        }
    }
}
