use std::fs;
use std::env;
use std::process;
use std::os::unix::fs::FileTypeExt;

/// File types, as reported in our listing.
#[derive(PartialEq)]
enum ListingFileType {
    Unknown,
    Regular,
    Directory,
    Symlink,
    Pipe,
    Socket,
    CharDevice,
    BlockDevice
}

/// A single entry of the listing we will produce.
struct ListingEntry {
    pub name: String,
    pub file_type: ListingFileType,
    pub link_target: String
}

impl ListingEntry {

    pub fn new(name: &str, file_type: ListingFileType) -> ListingEntry {
        ListingEntry {
            name: name.to_string(),
            file_type,
            link_target: "".to_string()
        }
    }

    pub fn new_symlink(name: &str, link_target: &str) -> ListingEntry {
        ListingEntry {
            name: name.to_string(),
            file_type: ListingFileType::Symlink,
            link_target: link_target.to_string()
        }
    }

    pub fn from_dentry(dentry: &fs::DirEntry) -> ListingEntry {

        // get the file name; this may fail, in which case
        // we print "???" to at least show that there is something
        let name = dentry.file_name().into_string();
        if name.is_err() {
            return ListingEntry::new("???", ListingFileType::Unknown);
        }
        let name = name.unwrap();

        // identify file type; this can also fail, in which case
        // we print the name and unknown type
        let dentry_file_type = dentry.file_type();
        if dentry_file_type.is_err() {
            return ListingEntry::new(&name, ListingFileType::Unknown);
        }
        let dentry_file_type = dentry_file_type.unwrap();

        if dentry_file_type.is_dir() {
            ListingEntry::new(&name, ListingFileType::Directory)
        }
        else if dentry_file_type.is_symlink() {
            let result = fs::read_link(dentry.path());
            match result {
                Err(_) => { ListingEntry::new_symlink(&name, "???") }
                Ok(target) => {
                    match target.to_str() {
                        Some(target) => {
                            ListingEntry::new_symlink(&name, target)
                        }
                        None => {
                            ListingEntry::new_symlink(&name, "???")
                        }
                    }
                }
            }
        }
        else if dentry_file_type.is_fifo() {
            ListingEntry::new(&name, ListingFileType::Pipe)
        }
        else if dentry_file_type.is_char_device() {
            ListingEntry::new(&name, ListingFileType::CharDevice)
        }
        else if dentry_file_type.is_block_device() {
            ListingEntry::new(&name, ListingFileType::BlockDevice)
        }
        else if dentry_file_type.is_socket() {
            ListingEntry::new(&name, ListingFileType::Socket)
        }
        else {
            ListingEntry::new(&name, ListingFileType::Regular)
        }
    }

    /// Returns a string containing Unicode icon for the file.
    pub fn get_icon(&self) -> &'static str {
        match self.file_type {
            ListingFileType::Unknown => { "\u{274E}\u{FE0E}" },
            ListingFileType::Regular => { "\u{1F5CE}\u{FE0E} " },
            ListingFileType::Directory => { "\u{1F4C1}\u{FE0E}" },
            ListingFileType::Symlink => { "\u{1F517}\u{FE0E}" },
            ListingFileType::Pipe => { "\u{1F6B0}\u{FE0E}" },
            ListingFileType::Socket => { "\u{1F50C}\u{FE0E}" },
            ListingFileType::CharDevice => { "\u{1F5A8}\u{FE0E}" },
            ListingFileType::BlockDevice => { "\u{1F4BF}\u{FE0E}" }
        }
    }
}

impl PartialEq for ListingEntry {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name &&
            self.file_type == other.file_type
    }
}

impl PartialOrd for ListingEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Eq for ListingEntry {
}

impl Ord for ListingEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // use the argument as the target dir; if none, use current dir
    let mut query = ".";
    if args.len() > 1 { query = &args[1]; }

    // open directory stream
    let rd = fs::read_dir(query).unwrap_or_else(|err| {
        eprintln!("Could not open '{query}': {err}");
        process::exit(1)
    });

    // build the list of files to show
    let mut listing = Vec::<ListingEntry>::new();
    for d in rd {
        if let Ok(dentry) = d {
            listing.push(ListingEntry::from_dentry(&dentry));
        } else {

            // if the query fails, add at least the "???" entry
            // to show that something was detected
            listing.push(ListingEntry::new("???", ListingFileType::Unknown));
        }
    }

    listing.sort();

    // show directories first
    for l in &listing {
        if l.file_type == ListingFileType::Directory {
            println!("{} {}", l.get_icon(), l.name);
        }
    }

    // then other files
    for l in &listing {
        if l.file_type != ListingFileType::Directory {
            if l.file_type == ListingFileType::Symlink {
                println!("{} {} -> {}", l.get_icon(), l.name, l.link_target);
            } else {
                println!("{} {}", l.get_icon(), l.name);
            }
        }
    }

    Ok(())
}

fn might_be_dir(dentry: &fs::DirEntry) -> bool
{
    match dentry.file_type() {
        Ok(file_type) => { file_type.is_dir() },
        Err(_) => { false }
    }
}

fn print_dir_entry(dentry: &fs::DirEntry)
{
    const ICON_ERROR:   &'static str = "\u{2753}\u{FE0E}";
    const ICON_FILE:    &'static str = "\u{1F5CE}\u{FE0E} ";
    const ICON_FOLDER:  &'static str = "\u{1F4C1}\u{FE0E}";
    const ICON_CHAIN:   &'static str = "\u{1F517}\u{FE0E}";
    const ICON_EMPTY_FILE: &'static str = "\u{2B55}\u{FE0E}";
    const ICON_SOCKET:  &'static str = "\u{1F50C}\u{FE0E}";
    const ICON_PIPE:    &'static str = "\u{1F6B0}\u{FE0E}";
    const ICON_TEXT_FILE: &'static str = "\u{1F5D2}\u{FE0E}";

    let name = dentry.file_name().into_string();
    if name.is_err() {
        println!("{} (error reading name)", ICON_ERROR);
        return;
    }
    let name = name.unwrap();

    let mut icon:&str = ICON_FILE;

    let file_type = dentry.file_type();
    if file_type.is_err() {
        println!("{} {}", ICON_ERROR, name);
        return;
    }
    let file_type = file_type.unwrap();

    if file_type.is_dir() {
        icon = ICON_FOLDER;
    }
    if file_type.is_symlink() {
        icon = ICON_CHAIN;
    };
    if file_type.is_socket() {
        icon = ICON_SOCKET;
    }
    if file_type.is_fifo() {
        icon = ICON_PIPE;
    }

    if file_type.is_file() {
        if name.ends_with(".txt") {
            icon = ICON_TEXT_FILE;
        }
        else {
            match dentry.metadata() {
                Ok(metadata) => {
                    if metadata.len() == 0 {
                        icon = ICON_EMPTY_FILE;
                    }
                },
                Err(_) => {}
            }
        }
    }

    println!("{} {}", icon, name);
}
