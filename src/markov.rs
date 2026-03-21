#![allow(dead_code)]
use crate::wandom::{XoShiRo256SS, weighted_choice::WeightedChoice};

use std::collections::HashMap;

pub struct MarkovChain<'a> {
    data: HashMap<MarkovNext<&'a str>, MarkovWeights<'a>>,
    order: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkovNext<T> {
    Start,
    Some(T),
    End,
}

pub type MarkovWeights<'a> = Vec<(MarkovNext<&'a str>, u64)>;

impl<'a> MarkovChain<'a> {
    pub fn new(texts: &[&'a str], order: usize) -> Self {
        let mut data = HashMap::new();

        for text in texts {
            let mut i = text
                .char_indices()
                .take(order)
                .last()
                .map_or(0, |(i, ch)| i + ch.len_utf8());

            let mut started = false;
            let mut ended = false;

            while !ended
                && let Some((p, _)) = text
                    .get(..i)
                    .and_then(|t| t.char_indices().rev().take(order).last())
                && let Some(prefix) = text.get(p..i)
            {
                let (prefix, next) = if started {
                    text.get(i..)
                        .and_then(|t| t.chars().next())
                        .and_then(|next| {
                            let start = i;
                            i += next.len_utf8();
                            text.get(start..i)
                        })
                        .map_or((MarkovNext::Some(prefix), MarkovNext::End), |next| {
                            (MarkovNext::Some(prefix), MarkovNext::Some(next))
                        })
                } else {
                    started = true;
                    (MarkovNext::Start, MarkovNext::Some(prefix))
                };

                if next == MarkovNext::End {
                    ended = true;
                }

                data.entry(prefix)
                    .and_modify(|weighted: &mut MarkovWeights<'a>| {
                        if let Some((_, weight)) =
                            weighted.iter_mut().find(|(ch, _)| match (ch, next) {
                                (MarkovNext::Some(ch), MarkovNext::Some(next)) if *ch == next => {
                                    true
                                }
                                (MarkovNext::End, MarkovNext::End) => true,
                                (_, _) => false,
                            })
                        {
                            *weight += 1;
                        } else {
                            weighted.push((next, 1));
                        }
                    })
                    .or_insert_with(|| vec![(next, 1)]);
            }
        }

        Self { data, order }
    }

    pub fn one<F>(&self, rng: &mut XoShiRo256SS, mut early_exit: F) -> String
    where
        F: FnMut(&str) -> bool,
    {
        let mut buffer = self
            .data
            .get(&MarkovNext::Start)
            .map_or_else(String::new, |b| {
                if let Some(MarkovNext::Some(b)) = b.choose_with_rng(rng) {
                    b.to_string()
                } else {
                    String::new()
                }
            });

        'append: loop {
            if early_exit(buffer.as_str()) {
                break 'append;
            }

            match self.data.get(&MarkovNext::Some(
                buffer
                    .char_indices()
                    .rev()
                    .take(self.order)
                    .last()
                    .map_or("", |(i, _)| &buffer[i..]),
            )) {
                Some(weighted) => match weighted.choose_with_rng(rng) {
                    Some(MarkovNext::Start) => {
                        unreachable!("I think this is unreachable but I need to contemplate more")
                    }
                    Some(MarkovNext::Some(ch)) => buffer.push_str(ch),
                    Some(MarkovNext::End) | None => break 'append,
                },
                None => break 'append,
            }
        }

        buffer
    }
}
