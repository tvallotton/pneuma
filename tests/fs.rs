#![feature(maybe_uninit_uninit_array, core_io_borrowed_buf)]
use std::io::prelude::*;

use pneuma::fs::{self, File,  OpenOptions};
use std::env;
use std::io::{BorrowedBuf, ErrorKind, SeekFrom};
use std::mem::MaybeUninit;
use std::path::Path;
use std::str;
use std::sync::Arc;

use std::thread;
use std::time::{Duration, Instant, SystemTime};

use rand::RngCore;

#[cfg(target_os = "macos")]
use std::ffi::{c_char, c_int};
#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_dir;
#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_file;
#[cfg(unix)]
use std::os::unix::fs::symlink as symlink_junction;
#[cfg(windows)]
use std::os::windows::fs::{symlink_dir, symlink_file};
#[cfg(windows)]
use std::sys::fs::symlink_junction;
#[cfg(target_os = "macos")]
use std::sys::weak::weak;

use std::path::PathBuf;


pub struct TempDir(PathBuf);

impl TempDir {
    pub fn join(&self, path: &str) -> PathBuf {
        let TempDir(ref p) = *self;
        p.join(path)
    }

    pub fn path(&self) -> &Path {
        let TempDir(ref p) = *self;
        p
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Gee, seeing how we're testing the fs module I sure hope that we
        // at least implement this correctly!
        let TempDir(ref p) = *self;
        let result = std:: fs::remove_dir_all(p);
        // Avoid panicking while panicking as this causes the process to
        // immediately abort, without displaying test results.
        if !thread::panicking() {
            result.unwrap();
            
        }
    }
}

#[track_caller] // for `rand::thread_rng()`
pub fn tmpdir() -> TempDir {
    let p = env::temp_dir();
    let mut r = rand::thread_rng();
    let ret = p.join(&format!("rust-{}", r.next_u32()));
    std::fs::create_dir(&ret).unwrap();
    TempDir(ret)
}

macro_rules! check {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => panic!("{} failed with: {e}", stringify!($e)),
        }
    };
}

#[cfg(windows)]
macro_rules! error {
    ($e:expr, $s:expr) => {
        match $e {
            Ok(_) => panic!("Unexpected success. Should've been: {:?}", $s),
            Err(ref err) => {
                assert!(
                    err.raw_os_error() == Some($s),
                    "`{}` did not have a code of `{}`",
                    err,
                    $s
                )
            }
        }
    };
}

#[cfg(unix)]
macro_rules! error {
    ($e:expr, $s:expr) => {
        error_contains!($e, $s)
    };
}

macro_rules! error_contains {
    ($e:expr, $s:expr) => {
        match $e {
            Ok(_) => panic!("Unexpected success. Should've been: {:?}", $s),
            Err(ref err) => {
                assert!(
                    err.to_string().contains($s),
                    "`{}` did not contain `{}`",
                    err,
                    $s
                )
            }
        }
    };
}

// Several test fail on windows if the user does not have permission to
// create symlinks (the `SeCreateSymbolicLinkPrivilege`). Instead of
// disabling these test on Windows, use this function to test whether we
// have permission, and return otherwise. This way, we still don't run these
// tests most of the time, but at least we do if the user has the right
// permissions.
pub fn got_symlink_permission(tmpdir: &TempDir) -> bool {
    if cfg!(unix) {
        return true;
    }
    let link = tmpdir.join("some_hopefully_unique_link_name");

    match symlink_file(r"nonexisting_target", link) {
        // ERROR_PRIVILEGE_NOT_HELD = 1314
        Err(ref err) if err.raw_os_error() == Some(1314) => false,
        Ok(_) | Err(_) => true,
    }
}

#[cfg(target_os = "macos")]
fn able_to_not_follow_symlinks_while_hard_linking() -> bool {
    weak!(fn linkat(c_int, *const c_char, c_int, *const c_char, c_int) -> c_int);
    linkat.get().is_some()
}

#[cfg(not(target_os = "macos"))]
fn able_to_not_follow_symlinks_while_hard_linking() -> bool {
    return true;
}

#[test]
fn file_test_io_smoke_test() {
    let message = "it's alright. have a good time";
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_rt_io_file_test.txt");
    {
        let mut write_stream = check!(File::create(filename));
        check!(write_stream.write(message.as_bytes()));
    }
    {
        let mut read_stream = check!(File::open(filename));
        let mut read_buf = [0; 1028];
        let read_str = match check!(read_stream.read(&mut read_buf)) {
            0 => panic!("shouldn't happen"),
            n => str::from_utf8(&read_buf[..n]).unwrap().to_string(),
        };
        assert_eq!(read_str, message);
    }
    check!(fs::remove_file(filename));
}

#[test]
fn invalid_path_raises() {
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_that_does_not_exist.txt");
    let result = File::open(filename);

    #[cfg(all(unix, not(target_os = "vxworks")))]
    error!(result, "No such file or directory");
    #[cfg(target_os = "vxworks")]
    error!(result, "no such file or directory");
    #[cfg(windows)]
    error!(result, 2); // ERROR_FILE_NOT_FOUND
}

#[test]
fn file_test_iounlinking_invalid_path_should_raise_condition() {
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_another_file_that_does_not_exist.txt");

    let result = fs::remove_file(filename);

    #[cfg(all(unix, not(target_os = "vxworks")))]
    error!(result, "No such file or directory");
    #[cfg(target_os = "vxworks")]
    error!(result, "no such file or directory");
    #[cfg(windows)]
    error!(result, 2); // ERROR_FILE_NOT_FOUND
}

#[test]
fn file_test_io_non_positional_read() {
    let message: &str = "ten-four";
    let mut read_mem = [0; 8];
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_rt_io_file_test_positional.txt");
    {
        let mut rw_stream = check!(File::create(filename));
        check!(rw_stream.write(message.as_bytes()));
    }
    {
        let mut read_stream = check!(File::open(filename));
        {
            let read_buf = &mut read_mem[0..4];
            check!(read_stream.read(read_buf));
        }
        {
            let read_buf = &mut read_mem[4..8];
            check!(read_stream.read(read_buf));
        }
    }
    check!(fs::remove_file(filename));
    let read_str = str::from_utf8(&read_mem).unwrap();
    assert_eq!(read_str, message);
}

#[test]
fn file_test_io_seek_and_tell_smoke_test() {
    let message = "ten-four";
    let mut read_mem = [0; 4];
    let set_cursor = 4 as u64;
    let tell_pos_pre_read;
    let tell_pos_post_read;
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_rt_io_file_test_seeking.txt");
    {
        let mut rw_stream = check!(File::create(filename));
        check!(rw_stream.write(message.as_bytes()));
    }
    {
        let mut read_stream = check!(File::open(filename));
        check!(read_stream.seek(SeekFrom::Start(set_cursor)));
        tell_pos_pre_read = check!(read_stream.seek(SeekFrom::Current(0)));
        check!(read_stream.read(&mut read_mem));
        tell_pos_post_read = check!(read_stream.seek(SeekFrom::Current(0)));
    }
    check!(fs::remove_file(filename));
    let read_str = str::from_utf8(&read_mem).unwrap();
    assert_eq!(read_str, &message[4..8]);
    assert_eq!(tell_pos_pre_read, set_cursor);
    assert_eq!(tell_pos_post_read, message.len() as u64);
}

#[test]
fn file_test_io_seek_and_write() {
    let initial_msg = "food-is-yummy";
    let overwrite_msg = "-the-bar!!";
    let final_msg = "foo-the-bar!!";
    let seek_idx = 3;
    let mut read_mem = [0; 13];
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_rt_io_file_test_seek_and_write.txt");
    {
        let mut rw_stream = check!(File::create(filename));
        check!(rw_stream.write(initial_msg.as_bytes()));
        check!(rw_stream.seek(SeekFrom::Start(seek_idx)));
        check!(rw_stream.write(overwrite_msg.as_bytes()));
    }
    {
        let mut read_stream = check!(File::open(filename));
        check!(read_stream.read(&mut read_mem));
    }
    check!(fs::remove_file(filename));
    let read_str = str::from_utf8(&read_mem).unwrap();
    assert!(read_str == final_msg);
}

#[test]
fn file_test_io_seek_shakedown() {
    //                   01234567890123
    let initial_msg = "qwer-asdf-zxcv";
    let chunk_one: &str = "qwer";
    let chunk_two: &str = "asdf";
    let chunk_three: &str = "zxcv";
    let mut read_mem = [0; 4];
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_rt_io_file_test_seek_shakedown.txt");
    {
        let mut rw_stream = check!(File::create(filename));
        check!(rw_stream.write(initial_msg.as_bytes()));
    }
    {
        let mut read_stream = check!(File::open(filename));

        check!(read_stream.seek(SeekFrom::End(-4)));
        check!(read_stream.read(&mut read_mem));
        assert_eq!(str::from_utf8(&read_mem).unwrap(), chunk_three);

        check!(read_stream.seek(SeekFrom::Current(-9)));
        check!(read_stream.read(&mut read_mem));
        assert_eq!(str::from_utf8(&read_mem).unwrap(), chunk_two);

        check!(read_stream.seek(SeekFrom::Start(0)));
        check!(read_stream.read(&mut read_mem));
        assert_eq!(str::from_utf8(&read_mem).unwrap(), chunk_one);
    }
    check!(fs::remove_file(filename));
}

#[test]
fn file_test_io_eof() {
    let tmpdir = tmpdir();
    let filename = tmpdir.join("file_rt_io_file_test_eof.txt");
    let mut buf = [0; 256];
    {
        let oo = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .clone();
        let mut rw = check!(oo.open(&filename));
        assert_eq!(check!(rw.read(&mut buf)), 0);
        assert_eq!(check!(rw.read(&mut buf)), 0);
    }
    check!(fs::remove_file(&filename));
}

#[test]
#[cfg(unix)]
fn file_test_io_read_write_at() {
    use std::os::unix::fs::FileExt;

    let tmpdir = tmpdir();
    let filename = tmpdir.join("file_rt_io_file_test_read_write_at.txt");
    let mut buf = [0; 256];
    let write1 = "asdf";
    let write2 = "qwer-";
    let write3 = "-zxcv";
    let content = "qwer-asdf-zxcv";
    {
        let oo = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .clone();
        let mut rw = check!(oo.open(&filename));
        assert_eq!(check!(rw.write_at(write1.as_bytes(), 5)), write1.len());
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 0);
        assert_eq!(check!(rw.read_at(&mut buf, 5)), write1.len());
        assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 0);
        assert_eq!(
            check!(rw.read_at(&mut buf[..write2.len()], 0)),
            write2.len()
        );
        assert_eq!(str::from_utf8(&buf[..write2.len()]), Ok("\0\0\0\0\0"));
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 0);
        assert_eq!(check!(rw.write(write2.as_bytes())), write2.len());
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 5);
        assert_eq!(check!(rw.read(&mut buf)), write1.len());
        assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
        assert_eq!(
            check!(rw.read_at(&mut buf[..write2.len()], 0)),
            write2.len()
        );
        assert_eq!(str::from_utf8(&buf[..write2.len()]), Ok(write2));
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
        assert_eq!(check!(rw.write_at(write3.as_bytes(), 9)), write3.len());
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
    }
    {
        let mut read = check!(File::open(&filename));
        assert_eq!(check!(read.read_at(&mut buf, 0)), content.len());
        assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 0);
        assert_eq!(check!(read.seek(SeekFrom::End(-5))), 9);
        assert_eq!(check!(read.read_at(&mut buf, 0)), content.len());
        assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 9);
        assert_eq!(check!(read.read(&mut buf)), write3.len());
        assert_eq!(str::from_utf8(&buf[..write3.len()]), Ok(write3));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
        assert_eq!(check!(read.read_at(&mut buf, 0)), content.len());
        assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
        assert_eq!(check!(read.read_at(&mut buf, 14)), 0);
        assert_eq!(check!(read.read_at(&mut buf, 15)), 0);
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
    }
    check!(fs::remove_file(&filename));
}

#[test]
#[cfg(windows)]
fn file_test_io_seek_read_write() {
    use std::os::windows::fs::FileExt;

    let tmpdir = tmpdir();
    let filename = tmpdir.join("file_rt_io_file_test_seek_read_write.txt");
    let mut buf = [0; 256];
    let write1 = "asdf";
    let write2 = "qwer-";
    let write3 = "-zxcv";
    let content = "qwer-asdf-zxcv";
    {
        let oo = OpenOptions::new()
            .create_new(true)
            .write(true)
            .read(true)
            .clone();
        let mut rw = check!(oo.open(&filename));
        assert_eq!(check!(rw.seek_write(write1.as_bytes(), 5)), write1.len());
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
        assert_eq!(check!(rw.seek_read(&mut buf, 5)), write1.len());
        assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
        assert_eq!(check!(rw.seek(SeekFrom::Start(0))), 0);
        assert_eq!(check!(rw.write(write2.as_bytes())), write2.len());
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 5);
        assert_eq!(check!(rw.read(&mut buf)), write1.len());
        assert_eq!(str::from_utf8(&buf[..write1.len()]), Ok(write1));
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 9);
        assert_eq!(
            check!(rw.seek_read(&mut buf[..write2.len()], 0)),
            write2.len()
        );
        assert_eq!(str::from_utf8(&buf[..write2.len()]), Ok(write2));
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 5);
        assert_eq!(check!(rw.seek_write(write3.as_bytes(), 9)), write3.len());
        assert_eq!(check!(rw.seek(SeekFrom::Current(0))), 14);
    }
    {
        let mut read = check!(File::open(&filename));
        assert_eq!(check!(read.seek_read(&mut buf, 0)), content.len());
        assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
        assert_eq!(check!(read.seek(SeekFrom::End(-5))), 9);
        assert_eq!(check!(read.seek_read(&mut buf, 0)), content.len());
        assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
        assert_eq!(check!(read.seek(SeekFrom::End(-5))), 9);
        assert_eq!(check!(read.read(&mut buf)), write3.len());
        assert_eq!(str::from_utf8(&buf[..write3.len()]), Ok(write3));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
        assert_eq!(check!(read.seek_read(&mut buf, 0)), content.len());
        assert_eq!(str::from_utf8(&buf[..content.len()]), Ok(content));
        assert_eq!(check!(read.seek(SeekFrom::Current(0))), 14);
        assert_eq!(check!(read.seek_read(&mut buf, 14)), 0);
        assert_eq!(check!(read.seek_read(&mut buf, 15)), 0);
    }
    check!(fs::remove_file(&filename));
}


#[test]
fn file_test_stat_is_correct_on_is_file() {
    let tmpdir = tmpdir();
    let filename = &tmpdir.join("file_stat_correct_on_is_file.txt");
    {
        let mut opts = OpenOptions::new();
        let mut fs = check!(opts.read(true).write(true).create(true).open(filename));
        let msg = "hw";
        fs.write(msg.as_bytes()).unwrap();

        let fstat_res = check!(fs.metadata());
        assert!(fstat_res.is_file());
    }
    let stat_res_fn = check!(fs::metadata(filename));
    assert!(stat_res_fn.is_file());
    let stat_res_meth = check!(filename.metadata());
    assert!(stat_res_meth.is_file());
    check!(fs::remove_file(filename));
}


#[test]
fn file_test_fileinfo_check_exists_before_and_after_file_creation() {
    let tmpdir = tmpdir();
    let file = &tmpdir.join("fileinfo_check_exists_b_and_a.txt");
    check!(check!(File::create(file)).write(b"foo"));
    assert!(file.exists());
    check!(fs::remove_file(file));
    assert!(!file.exists());
}

#[test]
fn file_create_new_already_exists_error() {
    let tmpdir = tmpdir();
    let file = &tmpdir.join("file_create_new_error_exists");
    check!(fs::File::create(file));
    let e = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file)
        .unwrap_err();
    assert_eq!(e.kind(), ErrorKind::AlreadyExists);
}


#[test]
fn symlinks_work() {
    let tmpdir = tmpdir();
    if !got_symlink_permission(&tmpdir) {
        return;
    };

    let input = tmpdir.join("in.txt");
    let out = tmpdir.join("out.txt");

    check!(check!(File::create(&input)).write("foobar".as_bytes()));
    check!(symlink_file(&input, &out));
    assert!(check!(out.symlink_metadata()).file_type().is_symlink());
    assert_eq!(
        check!(fs::metadata(&out)).len(),
        check!(fs::metadata(&input)).len()
    );
    let mut v = Vec::new();
    check!(check!(File::open(&out)).read_to_end(&mut v));
    assert_eq!(v, b"foobar".to_vec());
}


#[test]
fn sync_doesnt_kill_anything() {
    let tmpdir = tmpdir();
    let path = tmpdir.join("in.txt");

    let mut file = check!(File::create(&path));
    check!(file.sync_all());
    check!(file.sync_data());
    check!(file.write(b"foo"));
    check!(file.sync_all());
    check!(file.sync_data());
}


#[test]
fn open_flavors() {
    use pneuma::fs::OpenOptions as OO;
    fn c<T: Clone>(t: &T) -> T {
        t.clone()
    }

    let tmpdir = tmpdir();

    let mut r = OO::new();
    r.read(true);
    let mut w = OO::new();
    w.write(true);
    let mut rw = OO::new();
    rw.read(true).write(true);
    let mut a = OO::new();
    a.append(true);
    let mut ra = OO::new();
    ra.read(true).append(true);

    #[cfg(windows)]
    let invalid_options = 87; // ERROR_INVALID_PARAMETER
    #[cfg(all(unix, not(target_os = "vxworks")))]
    let invalid_options = "Invalid argument";
    #[cfg(target_os = "vxworks")]
    let invalid_options = "invalid argument";

    // Test various combinations of creation modes and access modes.
    //
    // Allowed:
    // creation mode           | read  | write | read-write | append | read-append |
    // :-----------------------|:-----:|:-----:|:----------:|:------:|:-----------:|
    // not set (open existing) |   X   |   X   |     X      |   X    |      X      |
    // create                  |       |   X   |     X      |   X    |      X      |
    // truncate                |       |   X   |     X      |        |             |
    // create and truncate     |       |   X   |     X      |        |             |
    // create_new              |       |   X   |     X      |   X    |      X      |
    //
    // tested in reverse order, so 'create_new' creates the file, and 'open existing' opens it.

    // write-only
    check!(c(&w).create_new(true).open(&tmpdir.join("a")));
    check!(c(&w).create(true).truncate(true).open(&tmpdir.join("a")));
    check!(c(&w).truncate(true).open(&tmpdir.join("a")));
    check!(c(&w).create(true).open(&tmpdir.join("a")));
    check!(c(&w).open(&tmpdir.join("a")));

    // read-only
    error!(
        c(&r).create_new(true).open(&tmpdir.join("b")),
        invalid_options
    );
    error!(
        c(&r).create(true).truncate(true).open(&tmpdir.join("b")),
        invalid_options
    );
    error!(
        c(&r).truncate(true).open(&tmpdir.join("b")),
        invalid_options
    );
    error!(c(&r).create(true).open(&tmpdir.join("b")), invalid_options);
    check!(c(&r).open(&tmpdir.join("a"))); // try opening the file created with write_only

    // read-write
    check!(c(&rw).create_new(true).open(&tmpdir.join("c")));
    check!(c(&rw).create(true).truncate(true).open(&tmpdir.join("c")));
    check!(c(&rw).truncate(true).open(&tmpdir.join("c")));
    check!(c(&rw).create(true).open(&tmpdir.join("c")));
    check!(c(&rw).open(&tmpdir.join("c")));

    // append
    check!(c(&a).create_new(true).open(&tmpdir.join("d")));
    error!(
        c(&a).create(true).truncate(true).open(&tmpdir.join("d")),
        invalid_options
    );
    error!(
        c(&a).truncate(true).open(&tmpdir.join("d")),
        invalid_options
    );
    check!(c(&a).create(true).open(&tmpdir.join("d")));
    check!(c(&a).open(&tmpdir.join("d")));

    // read-append
    check!(c(&ra).create_new(true).open(&tmpdir.join("e")));
    error!(
        c(&ra).create(true).truncate(true).open(&tmpdir.join("e")),
        invalid_options
    );
    error!(
        c(&ra).truncate(true).open(&tmpdir.join("e")),
        invalid_options
    );
    check!(c(&ra).create(true).open(&tmpdir.join("e")));
    check!(c(&ra).open(&tmpdir.join("e")));

    // Test opening a file without setting an access mode
    let mut blank = OO::new();
    error!(blank.create(true).open(&tmpdir.join("f")), invalid_options);

    // Test write works
    check!(check!(File::create(&tmpdir.join("h"))).write("foobar".as_bytes()));

    // Test write fails for read-only
    check!(r.open(&tmpdir.join("h")));
    {
        let mut f = check!(r.open(&tmpdir.join("h")));
        assert!(f.write("wut".as_bytes()).is_err());
    }

    // Test write overwrites
    {
        let mut f = check!(c(&w).open(&tmpdir.join("h")));
        check!(f.write("baz".as_bytes()));
    }
    {
        let mut f = check!(c(&r).open(&tmpdir.join("h")));
        let mut b = vec![0; 6];
        check!(f.read(&mut b));
        assert_eq!(b, "bazbar".as_bytes());
    }

    // Test truncate works
    {
        let mut f = check!(c(&w).truncate(true).open(&tmpdir.join("h")));
        check!(f.write("foo".as_bytes()));
    }
    assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 3);

    // Test append works
    assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 3);
    {
        let mut f = check!(c(&a).open(&tmpdir.join("h")));
        check!(f.write("bar".as_bytes()));
    }
    assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 6);

    // Test .append(true) equals .write(true).append(true)
    {
        let mut f = check!(c(&w).append(true).open(&tmpdir.join("h")));
        check!(f.write("baz".as_bytes()));
    }
    assert_eq!(check!(fs::metadata(&tmpdir.join("h"))).len(), 9);
}

#[test]
fn _assert_send_sync() {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<OpenOptions>();
}

#[test]
fn binary_file() {
    let mut bytes = [0; 1024];
    rand::thread_rng().fill_bytes(&mut bytes);

    let tmpdir = tmpdir();

    check!(check!(File::create(&tmpdir.join("test"))).write(&bytes));
    let mut v = Vec::new();
    check!(check!(File::open(&tmpdir.join("test"))).read_to_end(&mut v));
    assert!(v == &bytes[..]);
}

#[test]
fn file_open_not_found() {
    let res = File::open("/path/that/does/not/exist");
    assert_eq!(res.err().unwrap().kind(), ErrorKind::NotFound);
}

