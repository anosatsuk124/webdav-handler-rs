//! Contains the structs and traits that define a "filesystem" backend.
//!
use std;
use std::time::{SystemTime,UNIX_EPOCH};
use std::io::{Read,Write,Seek};
use std::fmt::Debug;

use webpath::WebPath;
use hyper::status::StatusCode;

macro_rules! notimplemented {
    ($method:expr) => {
        Err(FsError::NotImplemented)
    }
}

/// Errors generated by a filesystem implementation.
///
/// These are more result-codes than errors, really.
#[derive(Debug,Clone,Copy,PartialEq)]
pub enum FsError {
    NotImplemented,
    GeneralFailure,
    Exists,
    NotFound,
    Forbidden,
    InsufficientStorage,
    LoopDetected,
    PathTooLong,
    TooLarge,
    IsRemote,
}
/// The Result type.
pub type FsResult<T> = std::result::Result<T, FsError>;

/// A webdav "property".
#[derive(Debug,Clone)]
pub struct DavProp {
    pub name:       String,
    pub prefix:     Option<String>,
    pub namespace:  Option<String>,
    pub xml:        Option<Vec<u8>>,
}

/// The trait that defines a filesystem.
///
/// The BoxCloneFs trait is a helper trait that is automatically implemented
/// so that Box\<DavFileSystem\>.clone() works.
pub trait DavFileSystem : Debug + Sync + Send + BoxCloneFs {
    /// Open a file.
    fn open(&self, path: &WebPath, options: OpenOptions) -> FsResult<Box<DavFile>>;
    /// Perform read_dir.
    fn read_dir(&self, path: &WebPath) -> FsResult<Box< DavReadDir<Item=Box<DavDirEntry>> >>;

    /// Return the metadata of a file or directory.
    fn metadata(&self, path: &WebPath) -> FsResult<Box<DavMetaData>>;

    /// Return the metadata of a file, directory or symbolic link.
    ///
    /// Differs from metadata() that if the path is a symbolic link,
    /// it return the metadata for the link itself, not for the thing
    /// it points to.
    ///
    /// Has a default implementation that punts to metadata().
    #[allow(unused_variables)]
    fn symlink_metadata(&self, path: &WebPath) -> FsResult<Box<DavMetaData>> {
        self.metadata(path)
    }

    /// Create a directory.
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn create_dir(&self, path: &WebPath) -> FsResult<()> {
        notimplemented!("create_dir")
    }

    /// Remove a directory.
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn remove_dir(&self, path: &WebPath) -> FsResult<()> {
        notimplemented!("remove_dir")
    }

    /// Remove a file.
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn remove_file(&self, path: &WebPath) -> FsResult<()> {
        notimplemented!("remove_file")
    }

    /// Rename a file or directory.
    ///
    /// Source and destination must be the same type (file/dir).
    /// If the destination already exists and is a file, it
    /// should be replaced. If it is a directory it should give
    /// an error.
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn rename(&self, from: &WebPath, to: &WebPath) -> FsResult<()> {
        notimplemented!("rename")
    }

    /// Copy a file
    ///
    /// Should also copy the DAV properties, if properties
    /// are implemented.
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn copy(&self, from: &WebPath, to: &WebPath) -> FsResult<()> {
        notimplemented!("copy")
    }

    /// Set the access time of a file / directory.
    ///
    /// Default: notimplemented.
    #[doc(hidden)]
    #[allow(unused_variables)]
    fn set_accessed(&self, path: &WebPath, tm: SystemTime) -> FsResult<()> {
        notimplemented!("set_accessed")
    }

    /// Set the modified time of a file / directory.
    ///
    /// Default: notimplemented.
    #[doc(hidden)]
    #[allow(unused_variables)]
    fn set_modified(&self, path: &WebPath, tm: SystemTime) -> FsResult<()> {
        notimplemented!("set_accessed")
    }

    /// Indicator that tells if this filesystem driver supports DAV properties.
    ///
    /// Has a default "false" implementation.
    #[allow(unused_variables)]
    fn have_props(&self, path: &WebPath) -> bool {
        false
    }

    /// Patch the DAV properties of a node (add/remove props)
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn patch_props(&self, path: &WebPath, set: Vec<DavProp>, remove: Vec<DavProp>) -> FsResult<Vec<(StatusCode, DavProp)>> {
        notimplemented!("patch_props")
    }

    /// List/get the DAV properties of a node.
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn get_props(&self, path: &WebPath, do_content: bool) -> FsResult<Vec<DavProp>> {
        notimplemented!("get_props")
    }

    /// Get one specific named property of a node.
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn get_prop(&self, path: &WebPath, prop: DavProp) -> FsResult<Vec<u8>> {
        notimplemented!("get_prop`")
    }

    /// Get quota of this filesystem (used/total space).
    ///
    /// The first value returned is the amount of space used,
    /// the second optional value is the total amount of space
    /// (used + available).
    ///
    /// Has a default "notimplemented" implementation.
    #[allow(unused_variables)]
    fn get_quota(&self) -> FsResult<(u64, Option<u64>)> {
        notimplemented!("get_quota`")
    }
}

// BoxClone trait.
#[doc(hidden)]
pub trait BoxCloneFs {
    fn box_clone(&self) -> Box<DavFileSystem>;
}

// generic Clone, calls implementation-specific box_clone().
impl Clone for Box<DavFileSystem> {
    fn clone(&self) -> Box<DavFileSystem> {
        self.box_clone()
    }
}

// implementation-specific clone.
#[doc(hidden)]
impl<FS: Clone + DavFileSystem + 'static> BoxCloneFs for FS {
    fn box_clone(&self) -> Box<DavFileSystem> {
        Box::new((*self).clone())
    }
}

/// Iterator, returned by read_dir(), that generates DavDirEntries.
///
/// Often you'll end up creating an empty imp DavReadDir, plus an
/// impl Iterator.
pub trait DavReadDir : Iterator<Item=Box<DavDirEntry>> + Debug {
}

/// One directory entry (or child node).
pub trait DavDirEntry: Debug {
    /// name of the entry.
    fn name(&self) -> Vec<u8>;

    /// metadata of the entry.
    fn metadata(&self) -> FsResult<Box<DavMetaData>>;

    /// Default implementation of is_dir just returns `self.metadata()?.is_dir()`.
    /// Implementations can override this if their metadata() method is
    /// expensive and there is a cheaper way to provide the same info
    /// (e.g. dirent.d_type in unix filesystems).
    fn is_dir(&self) -> FsResult<bool> { Ok(self.metadata()?.is_dir()) }


    /// Likewise. Default: `!is_dir()`.
    fn is_file(&self) -> FsResult<bool> { Ok(self.metadata()?.is_file()) }

    /// Likewise. Default: `false`.
    fn is_symlink(&self) -> FsResult<bool> { Ok(self.metadata()?.is_symlink()) }
}

/// A DavFile should be readable/writeable/seekable, and be able
/// to return its metadata.
pub trait DavFile: Read + Write + Seek + Debug {
    fn metadata(&self) -> FsResult<Box<DavMetaData>>;
}

/// Not much more than type, length, and some timestamps.
///
/// The BoxCloneMd trait is a helper trait that is automatically implemented
/// so that Box\<DavMetaData\>.clone() works.
pub trait DavMetaData : Debug + BoxCloneMd {

    fn len(&self) -> u64;
    fn modified(&self) -> FsResult<SystemTime>;
	fn is_dir(&self) -> bool;

    /// Simplistic implementation of etag()
    ///
    /// Returns a simple etag that basically is "\<length\>-\<timestamp_in_ms\>"
    /// with the numbers in hex. Enough for most implementations.
    fn etag(&self) -> String {
		if let Ok(t) = self.modified() {
            if let Ok(t) = t.duration_since(UNIX_EPOCH) {
			    // apache style etag.
			    return format!("{:x}-{:x}", self.len(),
				    t.as_secs() * 1000000 + t.subsec_nanos() as u64 / 1000);
            }
		}
		format!("{:x}", self.len())
	}

    /// Default implementation for is_file() is !self.is_dir()
	fn is_file(&self) -> bool {
		!self.is_dir()
	}

    /// Default implementation for is_symlink() is "false".
	fn is_symlink(&self) -> bool {
		false
	}

    /// Last access time (default: notimplemented)
    fn accessed(&self) -> FsResult<SystemTime> {
        notimplemented!("access time")
    }

    /// Creation time (default: notimplemented)
    fn created(&self) -> FsResult<SystemTime> {
        notimplemented!("creation time")
    }

    /// Inode change time (ctime) (default: notimplemented)
    fn status_changed(&self) -> FsResult<SystemTime> {
        notimplemented!("status change time")
    }

    /// Is file executable (unix: has "x" mode bit) (default: notimplemented)
    fn executable(&self) -> FsResult<bool> {
        notimplemented!("executable")
    }
}

// generic Clone, calls implementation-specific box_clone().
impl Clone for Box<DavMetaData> {
    fn clone(&self) -> Box<DavMetaData> {
        self.box_clone()
    }
}

// BoxCloneMd trait.
#[doc(hidden)]
pub trait BoxCloneMd {
    fn box_clone(&self) -> Box<DavMetaData>;
}

// implementation-specific clone.
#[doc(hidden)]
impl<MD: Clone + DavMetaData + 'static> BoxCloneMd for MD {
    fn box_clone(&self) -> Box<DavMetaData> {
        Box::new((*self).clone())
    }
}

/// OpenOptions for open().
#[derive(Debug,Clone,Copy)]
pub struct OpenOptions {
    /// open for reading
    pub read: bool,
    /// open for writing
    pub write: bool,
    /// open in write-append mode
    pub append: bool,
    /// truncate file first when writing
    pub truncate: bool,
    /// create file if it doesn't exist
    pub create: bool,
    /// must create new file, fail if it already exists.
    pub create_new: bool,
}

impl OpenOptions {
    #[allow(dead_code)]
    pub(crate) fn new() -> OpenOptions {
        OpenOptions{
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub(crate) fn read() -> OpenOptions {
        OpenOptions{
            read: true,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }

    pub(crate) fn write() -> OpenOptions {
        OpenOptions{
            read: false,
            write: true,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
        }
    }
}

impl std::error::Error for FsError {
    fn description(&self) -> &str {
        "DavFileSystem error"
    }
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl std::fmt::Display for FsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

