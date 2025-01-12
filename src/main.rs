use std::fs;
use std::env;
use std::os::unix::fs::MetadataExt;
use std::process;
use std::os::unix::fs::FileTypeExt;

const ICON_ERROR:   &'static str = "\u{2753}\u{FE0E}";
const ICON_FILE:    &'static str = "\u{1F5CE}\u{FE0E} ";
const ICON_DIRECTORY:  &'static str = "\u{1F4C1}\u{FE0E}";
const ICON_SYMLINK: &'static str = "\u{1F517}\u{FE0E}";
const ICON_EMPTY_FILE: &'static str = "\u{2B55}\u{FE0E}";
const ICON_SOCKET:  &'static str = "\u{1F50C}\u{FE0E}";
const ICON_PIPE:    &'static str = "\u{1F6B0}\u{FE0E}";
const ICON_TEXT_FILE: &'static str = "\u{1F5D2}\u{FE0E}";
const ICON_CHAR_DEVICE: &'static str = "\u{1F5A8}\u{FE0E}";
const ICON_BLOCK_DEVICE: &'static str = "\u{1F4BF}\u{FE0E}";
const ICON_DISK:    &'static str = "\u{1F5D4}\u{FE0E}";
const ICON_DEV_NULL:  &'static str = "\u{1F6BD}\u{FE0E}";
const ICON_TTY:     &'static str = "\u{1F4BB}\u{FE0E}";

/// A single entry of the listing we will produce.
#[derive(PartialEq, Eq)]
enum ListingEntry {

    Unknown {
        name: String,
        icon: String
    },
    Regular {
        name: String,
        icon: String
    },
    Directory {
        name: String,
        icon: String
    },
    Symlink {
        name: String,
        target: String,
        icon: String
    },
    Pipe {
        name: String,
        icon: String
    },
    Socket {
        name: String,
        icon: String
    },
    CharDevice {
        name: String,
        dev_id: u64,
        icon: String
    },
    BlockDevice {
        name: String,
        dev_id: u64,
        icon: String
    }
}

impl ListingEntry {

    pub fn get_name(&self) -> String {
        match self {
            ListingEntry::Unknown { name, .. } => { name.to_string() }
            ListingEntry::Regular { name, .. } => { name.to_string() }
            ListingEntry::Directory { name, .. } => { name.to_string() }
            ListingEntry::Symlink { name, .. } => { name.to_string() }
            ListingEntry::Pipe { name, .. } => { name.to_string() }
            ListingEntry::Socket { name, .. } => { name.to_string() }
            ListingEntry::CharDevice { name, .. } => { name.to_string() }
            ListingEntry::BlockDevice { name, .. } => { name.to_string() }
        }
    }

    pub fn get_icon(&self) -> String {
        match self {
            ListingEntry::Unknown { icon, .. } => { icon.to_string() }
            ListingEntry::Regular { icon, .. } => { icon.to_string() }
            ListingEntry::Directory { icon, .. } => { icon.to_string() }
            ListingEntry::Symlink { icon, .. } => { icon.to_string() }
            ListingEntry::Pipe { icon, .. } => { icon.to_string() }
            ListingEntry::Socket { icon, .. } => { icon.to_string() }
            ListingEntry::CharDevice { icon, .. } => { icon.to_string() }
            ListingEntry::BlockDevice { icon, .. } => { icon.to_string() }
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            ListingEntry::Directory { .. } => { true }
            _ => { false }
        }
    }

    pub fn new_regular(name: &str) -> ListingEntry {
        ListingEntry::Regular {
            name: name.to_string(),
            icon: ICON_FILE.into()
        }
    }

    pub fn new_dir(name: &str) -> ListingEntry {
        ListingEntry::Directory {
            name: name.to_string(),
            icon: ICON_DIRECTORY.into()
        }
    }

    pub fn new_symlink(name: &str, target: &str) -> ListingEntry {
        ListingEntry::Symlink {
            name: name.to_string(),
            target: target.to_string(),
            icon: ICON_SYMLINK.into()
        }
    }

    pub fn new_unknown(name: &str) -> ListingEntry {
        ListingEntry::Unknown {
            name: name.to_string(),
            icon: ICON_ERROR.into()
        }
    }

    pub fn new_pipe(name: &str) -> ListingEntry {
        ListingEntry::Pipe {
            name: name.to_string(),
            icon: ICON_PIPE.into()
        }
    }

    pub fn new_char_device(name: &str, dev_id: u64) -> ListingEntry {
        let mut icon = ICON_CHAR_DEVICE;

        // give some specific devices their own icons
        let dev_major = (dev_id & 0x000000000000ff00) >> 8;
        let dev_minor = dev_id & 0x00000000000000ff;
        if dev_major == 1 && dev_minor == 3 {   // /dev/null
            icon = ICON_DEV_NULL;
        }
        else if dev_major == 4 {                // oldschool ttys
            icon = ICON_TTY;
        }
        else if dev_major == 5 && (dev_minor == 0 || dev_minor == 1) {      // /dev/tty, /dev/console
            icon = ICON_TTY;
        }
        else if dev_major == 241 {              // disks
            icon = ICON_DISK;
        }

        ListingEntry::CharDevice {
            name: name.to_string(),
            dev_id: dev_id,
            icon: icon.into()
        }
    }

    pub fn new_block_device(name: &str, dev_id: u64) -> ListingEntry {
        ListingEntry::BlockDevice {
            name: name.to_string(),
            dev_id: dev_id,
            icon: ICON_BLOCK_DEVICE.into()
        }
    }

    pub fn new_socket(name: &str) -> ListingEntry {
        ListingEntry::Socket {
            name: name.to_string(),
            icon: ICON_SOCKET.into()
        }
    }

    pub fn from_dentry(dentry: &fs::DirEntry) -> ListingEntry {

        // get the file name; this may fail, in which case
        // we print "???" to at least show that there is something
        let name = dentry.file_name().into_string();
        if name.is_err() {
            return ListingEntry::new_unknown("???");
        }
        let name = name.unwrap();

        // identify file type; this can also fail, in which case
        // we print the name and unknown type
        let dentry_file_type = dentry.file_type();
        if dentry_file_type.is_err() {
            return ListingEntry::new_unknown(&name);
        }
        let dentry_file_type = dentry_file_type.unwrap();

        if dentry_file_type.is_dir() {
            ListingEntry::new_dir(&name)
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
            ListingEntry::new_pipe(&name)
        }
        else if dentry_file_type.is_char_device() {
            let result = dentry.metadata();
            match result {
                Err(_) => { ListingEntry::new_char_device(&name, 0) }
                Ok(metadata) => {
                    let dev_id = metadata.rdev();
                    ListingEntry::new_char_device(&name, dev_id)
                }
            }
        }
        else if dentry_file_type.is_block_device() {
            let result = dentry.metadata();
            match result {
                Err(_) => { ListingEntry::new_block_device(&name, 0) }
                Ok(metadata) => {
                    let dev_id = metadata.rdev();
                    ListingEntry::new_block_device(&name, dev_id)
                }
            }
        }
        else if dentry_file_type.is_socket() {
            ListingEntry::new_socket(&name)
        }
        else {
            ListingEntry::new_regular(&name)
        }
    }
}

impl PartialOrd for ListingEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.get_name().partial_cmp(&other.get_name())
    }
}

impl Ord for ListingEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.get_name().cmp(&other.get_name())
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
            listing.push(ListingEntry::new_unknown("???"));
        }
    }

    listing.sort();

    // show directories first
    for l in &listing {
        if l.is_directory() {
            println!("{} {}", l.get_icon(), l.get_name());
        }
    }

    // then other files
    for l in &listing {
        match l {
            ListingEntry::Directory {..} => { },
            ListingEntry::Symlink { name, icon, target } => {
                println!("{} {} -> {}", icon, name, target);
            }
            _ => {
                println!("{} {}", l.get_icon(), l.get_name());
            }
        }
    }

    Ok(())
}
