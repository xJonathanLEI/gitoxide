use super::Tree;
use crate::Repository;

/// An entry within a tree
pub struct EntryRef<'repo, 'a> {
    /// The actual entry ref we are wrapping.
    pub inner: git_object::tree::EntryRef<'a>,

    pub(crate) repo: &'repo Repository,
}

impl<'repo, 'a> EntryRef<'repo, 'a> {
    /// The kind of object to which [`id()`][Self::id()] is pointing.
    pub fn mode(&self) -> git_object::tree::EntryMode {
        self.inner.mode
    }

    /// The name of the file in the parent tree.
    pub fn filename(&self) -> &git_object::bstr::BStr {
        self.inner.filename
    }

    /// Return the entries id, connected to the underlying repository.
    pub fn id(&self) -> crate::Id<'repo> {
        crate::Id::from_id(self.inner.oid, self.repo)
    }

    /// Return the entries id, without repository connection.
    pub fn oid(&self) -> git_hash::ObjectId {
        self.inner.oid.to_owned()
    }
}

impl<'repo, 'a> std::fmt::Display for EntryRef<'repo, 'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:06o} {:>6} {}\t{}",
            self.mode() as u32,
            self.mode().as_str(),
            self.id().shorten_or_id(),
            self.filename()
        )
    }
}

impl<'repo> Tree<'repo> {
    /// Return an iterator over tree entries.
    pub fn iter(&self) -> impl Iterator<Item = Result<EntryRef<'repo, '_>, git_object::decode::Error>> {
        let repo = self.repo;
        git_object::TreeRefIter::from_bytes(&self.data).map(move |e| e.map(|entry| EntryRef { inner: entry, repo }))
    }
}
