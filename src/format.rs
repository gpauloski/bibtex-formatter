use crate::parse::{Tag, Entry};


fn format_tag(tag: &Tag) -> String {
    format!("{} = {{{}}}", tag.name, tag.content)
}


pub fn format_entry(entry: &Entry) -> String {
    let mut fmt = String::new();

    if entry.tags.is_empty() {
        return format!("@{}{{{}}}", entry.kind, entry.key)
    }

    fmt.push_str(&format!("@{}{{{},\n", entry.kind, entry.key));

    for tag in &entry.tags {
        let tag = format_tag(&tag);
        fmt.push_str(&format!("    {},\n", &tag));
    }

    fmt.push('}');

    fmt
}


pub fn format_entries(entries: &Vec<Entry>) -> Vec<String> {
    entries.iter().map(format_entry).collect()
}


pub fn print_entries(entries: &Vec<Entry>) {
    let formatted = format_entries(entries);

    for (i, fmt) in formatted.iter().enumerate() {
        if i != 0 {
            println!("");
        }
        println!("{}", fmt);
    }
}
