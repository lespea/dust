extern crate ansi_term;

use self::ansi_term::Colour::Fixed;
use self::ansi_term::Style;
use std::collections::HashSet;
use utils::{ensure_end_slash, strip_end_slash};

static UNITS: [char; 4] = ['T', 'G', 'M', 'K'];

pub fn draw_it(
    permissions: bool,
    short_paths: bool,
    depth: Option<u64>,
    base_dirs: HashSet<String>,
    to_display: Vec<(String, u64)>,
) {
    if !permissions {
        eprintln!("Did not have permissions for all directories");
    }
    let mut found = HashSet::new();

    for &(ref k, _) in to_display.iter() {
        if base_dirs.contains(k) {
            display_node(&k, &mut found, &to_display, true, short_paths, depth, "─┴");
        }
    }
}

fn get_size(nodes: &[(String, u64)], node_to_print: &str) -> Option<u64> {
    for &(ref k, ref v) in nodes.iter() {
        if *k == *node_to_print {
            return Some(*v);
        }
    }
    None
}

fn display_node(
    node_to_print: &str,
    found: &mut HashSet<String>,
    to_display: &[(String, u64)],
    is_biggest: bool,
    short_paths: bool,
    depth: Option<u64>,
    indentation_str: &str,
) {
    if found.contains(node_to_print) {
        return;
    }
    found.insert(node_to_print.to_string());

    let new_depth = match depth {
        None => None,
        Some(0) => return,
        Some(d) => Some(d - 1),
    };
    match get_size(to_display, node_to_print) {
        None => println!("Can not find path: {}", node_to_print),
        Some(size) => {
            let new_indent = clean_indentation_string(indentation_str.into());

            let ntp_with_slash = strip_end_slash(node_to_print);

            // Annoying edge case for when run on root directory
            let num_slashes = if ntp_with_slash == "/" {
                1
            } else {
                ntp_with_slash.matches('/').count() + 1
            };
            let mut num_siblings = count_siblings(to_display, num_slashes - 1, node_to_print);
            let start_sib = num_siblings;

            let mut new_is_biggest = true;
            for &(ref k, _) in to_display.iter() {
                let temp = String::from(ensure_end_slash(node_to_print));
                if k.starts_with(temp.as_str()) && k.matches('/').count() == num_slashes {
                    num_siblings -= 1;
                    let has_children = has_children(to_display, new_depth, k, num_slashes);
                    display_node(
                        k,
                        found,
                        to_display,
                        num_siblings!=start_sib-1,
                        short_paths,
                        new_depth,
                        &*(new_indent.to_string() + get_tree_chars(num_siblings!=start_sib-1, has_children)),
                    );
                    new_is_biggest = false;
                }
            };
            print_this_node(node_to_print, size, is_biggest, short_paths, &*indentation_str);
        }
    }
}

fn clean_indentation_string(s: String) -> String {
    let mut is :String = s;
    is = is.replace("┬─┴", "  ");
    is = is.replace("┬──", "  ");
    is = is.replace("├──", "│ ");
    is = is.replace("├─┴", "│ ");
    is = is.replace("─┴", " ");
    is
}

fn count_siblings(to_display: &[(String, u64)], num_slashes: usize, ntp: &str) -> u64 {
    to_display.iter().fold(0, |a, b| {
        if b.0.starts_with(ntp) && b.0.as_str().matches('/').count() == num_slashes + 1 {
            a + 1
        } else {
            a
        }
    })
}

fn has_children(
    to_display: &[(String, u64)],
    new_depth: Option<u64>,
    ntp: &str,
    num_slashes: usize,
) -> bool {
    if new_depth.is_none() || new_depth.unwrap() != 1 {
        for &(ref k2, _) in to_display.iter() {
            let ntp_with_slash = String::from(ntp.to_owned() + "/");
            if k2.starts_with(ntp_with_slash.as_str()) && k2.matches('/').count() == num_slashes + 1
            {
                return true;
            }
        }
    }
    false
}

fn get_tree_chars(has_smaller_siblings: bool, has_children: bool) -> &'static str {
    if !has_smaller_siblings {
        if has_children {
            "┬─┴"
        } else {
            "┬──"
        }
    } else if has_children {
        "├─┴"
    } else {
        "├──"
    }
}

fn print_this_node(
    node_name: &str,
    size: u64,
    is_biggest: bool,
    short_paths: bool,
    indentation: &str,
) {
    let pretty_size = format!("{:>5}", human_readable_number(size),);
    println!(
        "{}",
        format_string(
            node_name,
            is_biggest,
            short_paths,
            pretty_size.as_ref(),
            indentation
        )
    )
}

pub fn format_string(
    dir_name: &str,
    is_biggest: bool,
    short_paths: bool,
    size: &str,
    indentation: &str,
) -> String {
    let printable_name = {
        if short_paths {
            dir_name.split('/').last().unwrap_or(dir_name)
        } else {
            dir_name
        }
    };
    format!(
        "{} {} {}",
        if is_biggest {
            Fixed(196).paint(size)
        } else {
            Style::new().paint(size)
        },
        indentation,
        printable_name,
    )
}

fn human_readable_number(size: u64) -> String {
    for (i, u) in UNITS.iter().enumerate() {
        let marker = 1024u64.pow((UNITS.len() - i) as u32);
        if size >= marker {
            if size / marker < 10 {
                return format!("{:.1}{}", (size as f32 / marker as f32), u);
            } else {
                return format!("{}{}", (size / marker), u);
            }
        }
    }
    return format!("{}B", size);
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_human_readable_number() {
        assert_eq!(human_readable_number(1), "1B");
        assert_eq!(human_readable_number(956), "956B");
        assert_eq!(human_readable_number(1004), "1004B");
        assert_eq!(human_readable_number(1024), "1.0K");
        assert_eq!(human_readable_number(1536), "1.5K");
        assert_eq!(human_readable_number(1024 * 512), "512K");
        assert_eq!(human_readable_number(1024 * 1024), "1.0M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 - 1), "1023M");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 20), "20G");
        assert_eq!(human_readable_number(1024 * 1024 * 1024 * 1024), "1.0T");
    }
}
