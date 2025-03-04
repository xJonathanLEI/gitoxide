use std::{cmp::Ordering, io};

pub(crate) const LARGE_OFFSET_THRESHOLD: u64 = 0x7fff_ffff;
pub(crate) const HIGH_BIT: u32 = 0x8000_0000;

use git_features::{
    hash,
    progress::{self, Progress},
};

use crate::index::{util::Count, V2_SIGNATURE};

pub(crate) fn write_to(
    out: impl io::Write,
    entries_sorted_by_oid: Vec<crate::cache::delta::Item<crate::index::write::TreeEntry>>,
    pack_hash: &git_hash::ObjectId,
    kind: crate::index::Version,
    mut progress: impl Progress,
) -> io::Result<git_hash::ObjectId> {
    use io::Write;
    assert_eq!(kind, crate::index::Version::V2, "Can only write V2 packs right now");
    assert!(
        entries_sorted_by_oid.len() <= u32::MAX as usize,
        "a pack cannot have more than u32::MAX objects"
    );

    // Write header
    let mut out = Count::new(std::io::BufWriter::with_capacity(
        8 * 4096,
        hash::Write::new(out, kind.hash()),
    ));
    out.write_all(V2_SIGNATURE)?;
    out.write_all(&(kind as u32).to_be_bytes())?;

    progress.init(Some(4), progress::steps());
    let start = std::time::Instant::now();
    let _info = progress.add_child("writing fan-out table");
    let fan_out = fanout(entries_sorted_by_oid.iter().map(|e| e.data.id.first_byte()));

    for value in fan_out.iter() {
        out.write_all(&value.to_be_bytes())?;
    }

    progress.inc();
    let _info = progress.add_child("writing ids");
    for entry in &entries_sorted_by_oid {
        out.write_all(entry.data.id.as_slice())?;
    }

    progress.inc();
    let _info = progress.add_child("writing crc32");
    for entry in &entries_sorted_by_oid {
        out.write_all(&entry.data.crc32.to_be_bytes())?;
    }

    progress.inc();
    let _info = progress.add_child("writing offsets");
    {
        let mut offsets64 = Vec::<u64>::new();
        for entry in &entries_sorted_by_oid {
            let offset: u32 = if entry.offset > LARGE_OFFSET_THRESHOLD {
                assert!(
                    offsets64.len() < LARGE_OFFSET_THRESHOLD as usize,
                    "Encoding breakdown - way too many 64bit offsets"
                );
                offsets64.push(entry.offset);
                ((offsets64.len() - 1) as u32) | HIGH_BIT
            } else {
                entry.offset as u32
            };
            out.write_all(&offset.to_be_bytes())?;
        }
        for value in offsets64 {
            out.write_all(&value.to_be_bytes())?;
        }
    }

    out.write_all(pack_hash.as_slice())?;

    let bytes_written_without_trailer = out.bytes;
    let mut out = out.inner.into_inner()?;
    let index_hash: git_hash::ObjectId = out.hash.digest().into();
    out.inner.write_all(index_hash.as_slice())?;
    out.inner.flush()?;

    progress.inc();
    progress.show_throughput_with(
        start,
        (bytes_written_without_trailer + 20) as usize,
        progress::bytes().expect("unit always set"),
        progress::MessageLevel::Success,
    );

    Ok(index_hash)
}

pub(crate) fn fanout(iter: impl ExactSizeIterator<Item = u8>) -> [u32; 256] {
    let mut fan_out = [0u32; 256];
    let entries_len = iter.len() as u32;
    let mut iter = iter.enumerate();
    let mut idx_and_entry = iter.next();
    let mut upper_bound = 0;

    for (offset_be, byte) in fan_out.iter_mut().zip(0u8..=255) {
        *offset_be = match idx_and_entry.as_ref() {
            Some((_idx, first_byte)) => match first_byte.cmp(&byte) {
                Ordering::Less => unreachable!("ids should be ordered, and we make sure to keep ahead with them"),
                Ordering::Greater => upper_bound,
                Ordering::Equal => {
                    if byte == 255 {
                        entries_len
                    } else {
                        idx_and_entry = iter.find(|(_, first_byte)| *first_byte != byte);
                        upper_bound = idx_and_entry
                            .as_ref()
                            .map(|(idx, _)| *idx as u32)
                            .unwrap_or(entries_len);
                        upper_bound
                    }
                }
            },
            None => entries_len,
        };
    }

    fan_out
}
