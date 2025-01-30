use super::Source;
use crate::Key;

pub fn _and_predicate_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let temp_position = position;
    let (valid, _position) = func(parent, source, temp_position);
    if !valid {
        return (false, temp_position);
    }
    (true, temp_position)
}

pub fn _and_predicate(
    func: &impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + '_ {
    move |parent: Key, source: &Source, position: u32| {
        _and_predicate_kernel(parent, source, position, func)
    }
}

pub fn _not_predicate_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let temp_position = position;
    let (valid, _position) = func(parent, source, temp_position);
    if !valid {
        return (true, temp_position);
    }
    (false, temp_position)
}

pub fn _not_predicate(
    func: &impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + '_ {
    move |parent: Key, source: &Source, position: u32| {
        _not_predicate_kernel(parent, source, position, func)
    }
}

pub fn _one_or_more_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let mut temp_position = position;
    let (valid, position) = func(parent, source, temp_position);
    if !valid {
        return (false, temp_position);
    }
    temp_position = position;
    loop {
        let (valid, position) = func(parent, source, temp_position);
        if !valid {
            break;
        }
        temp_position = position;
    }
    (true, temp_position)
}

pub fn _one_or_more(
    func: &impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + '_ {
    move |parent: Key, source: &Source, position: u32| {
        _one_or_more_kernel(parent, source, position, func)
    }
}

pub fn _optional_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let temp_position = position;
    let (valid, position) = func(parent, source, temp_position);
    if !valid {
        return (true, temp_position);
    }
    (true, position)
}

pub fn _optional(
    func: &impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + '_ {
    move |parent: Key, source: &Source, position: u32| {
        _optional_kernel(parent, source, position, func)
    }
}

pub fn _ordered_choice_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func_lhs: impl Fn(Key, &Source, u32) -> (bool, u32),
    func_rhs: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let temp_position = position;
    let (valid, position) = func_lhs(parent, source, position);
    if valid {
        return (true, position);
    }
    let position = temp_position;
    let (valid, position) = func_rhs(parent, source, position);

    if valid {
        (true, position)
    } else {
        (false, temp_position)
    }
}

pub fn _ordered_choice<'a>(
    func_lhs: &'a impl Fn(Key, &Source, u32) -> (bool, u32),
    func_rhs: &'a impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + 'a {
    move |parent: Key, source: &Source, position: u32| {
        _ordered_choice_kernel(parent, source, position, func_lhs, func_rhs)
    }
}

pub fn _sequence_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func_lhs: impl Fn(Key, &Source, u32) -> (bool, u32),
    func_rhs: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let temp_position = position;

    let (lhs_bool, position) = func_lhs(parent, source, position);

    if lhs_bool {
        let (rhs_bool, position) = func_rhs(parent, source, position);
        if rhs_bool {
            return (true, position);
        }
    }
    (false, temp_position)
}

pub fn _sequence<'a>(
    func_lhs: &'a impl Fn(Key, &Source, u32) -> (bool, u32),
    func_rhs: &'a impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + 'a {
    move |parent: Key, source: &Source, position: u32| {
        _sequence_kernel(parent, source, position, func_lhs, func_rhs)
    }
}

pub fn _subexpression_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let temp_position = position;
    let (valid, position) = func(parent, source, temp_position);
    if !valid {
        return (false, temp_position);
    }
    (true, position)
}

pub fn _subexpression(
    func: &impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + '_ {
    move |parent: Key, source: &Source, position: u32| {
        _subexpression_kernel(parent, source, position, func)
    }
}

pub fn _terminal_kernel(source: &Source, position: u32, chr: u8) -> (bool, u32) {
    let char = source.get_char(position);
    //let char = char.expect("\nThis error only happens with invalid grammar. E.g **. This should be disallowed at generation time!!\n");
    if char == Some(chr) {
        (true, position + 1)
    } else {
        (false, position)
    }
}

pub fn _terminal(chr: u8) -> impl Fn(Key, &Source, u32) -> (bool, u32) {
    move |_parent: Key, source: &Source, position: u32| _terminal_kernel(source, position, chr)
}

pub fn _zero_or_more_kernel(
    parent: Key,
    source: &Source,
    position: u32,
    func: impl Fn(Key, &Source, u32) -> (bool, u32),
) -> (bool, u32) {
    let mut temp_position = position;
    loop {
        let (valid, position) = func(parent, source, temp_position);
        if !valid {
            break;
        }
        temp_position = position;
    }
    // Always true but may consume zero positions
    (true, temp_position)
}

pub fn _zero_or_more(
    func: &impl Fn(Key, &Source, u32) -> (bool, u32),
) -> impl Fn(Key, &Source, u32) -> (bool, u32) + '_ {
    move |parent: Key, source: &Source, position: u32| {
        _zero_or_more_kernel(parent, source, position, func)
    }
}

// Optimizations

fn _string_terminal_kernel_ascii_opt(source: &Source, position: u32, data: &[u8]) -> (bool, u32) {
    let mut end_position = position;
    for char in data {
        let s = match source.get_char(end_position) {
            None => return (false, position),
            Some(val) => val,
        };
        if *char == s {
            end_position += 1;
        } else {
            return (false, position);
        }
    }
    (true, end_position)
}

pub fn _string_terminal_opt_ascii(data: &[u8]) -> impl Fn(Key, &Source, u32) -> (bool, u32) + '_ {
    move |_parent: Key, source: &Source, position: u32| {
        _string_terminal_kernel_ascii_opt(source, position, data)
    }
}

fn _ordered_choice_match_range_kernel(
    source: &Source,
    position: u32,
    start: u32,
    end: u32,
) -> (bool, u32) {
    let chr = source.get_char(position);
    match chr {
        Some(value) => {
            if (value as u32 >= start) && (value as u32 <= end) {
                (true, position + 1)
            } else {
                (false, position)
            }
        }
        None => (false, position),
    }
}

pub fn _ordered_choice_match_range(
    start: u32,
    end: u32,
) -> impl Fn(Key, &Source, u32) -> (bool, u32) {
    move |_parent: Key, source: &Source, position: u32| {
        _ordered_choice_match_range_kernel(source, position, start, end)
    }
}
