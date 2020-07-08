use std::collections::HashSet;

use conhash::{ConsistentHash, Node};
use crossbeam_utils::sync::ShardedLock;
use smol_str::SmolStr;

use neuromancer::DefaultHasher;

pub(crate) struct Executor {
    pub(crate) librarians: ShardedLock<KnownLibrarians>,
}

pub(crate) trait ToLibrarian {
    fn to_librarian(&self) -> Librarian;
}

#[derive(Clone, Default, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Librarian {
    address: SmolStr,
}

pub(crate) struct KnownLibrarians {
    set: HashSet<Librarian, DefaultHasher>,
    ring: ConsistentHash<Librarian>,
}

impl Executor {
    pub(crate) fn new() -> Self {
        Self {
            librarians: ShardedLock::new(KnownLibrarians::new()),
        }
    }
}

impl KnownLibrarians {
    pub(crate) fn new() -> Self {
        let ring = ConsistentHash::new();
        Self {
            set: HashSet::default(),
            ring,
        }
    }

    /// Accepts the new list of librarians and returns the librarians that were removed.
    #[must_use = "You must remap the values that have been assigned to the deleted servers"]
    pub(crate) fn modify_membership(&mut self, librarians: &[Librarian]) -> Vec<Librarian> {
        let mut removed = Vec::new();
        let mut added = Vec::new();
        let new_membership_list: HashSet<Librarian, _> = librarians.into_iter().cloned().collect();
        if new_membership_list.is_superset(&self.set) {
            for member in new_membership_list.difference(&self.set) {
                added.push(member.clone());
            }
        } else {
            for member in new_membership_list.symmetric_difference(&self.set) {
                if self.set.contains(member) {
                    removed.push(member.clone());
                } else {
                    // the member was added
                    added.push(member.clone());
                }
            }
        }
        for deleted in &removed {
            self.ring.remove(deleted);
            self.set.remove(deleted);
        }
        for new in added {
            self.ring.add(&new, 1);
            self.set.insert(new);
        }
        removed
    }
}

impl Librarian {
    pub fn new(s: impl AsRef<str>) -> Self {
        let address = SmolStr::new(s);
        Self { address }
    }
}

impl Node for Librarian {
    fn name(&self) -> String {
        self.address.to_string()
    }
}

impl<T> ToLibrarian for T
where
    T: AsRef<str>,
{
    fn to_librarian(&self) -> Librarian {
        Librarian::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn superset_modify_membership() {
        let input: Vec<Librarian> = ["foo", "bar"]
            .iter()
            .map(<_ as ToLibrarian>::to_librarian)
            .collect();
        let mut known = KnownLibrarians::new();

        let removed = known.modify_membership(&input);

        assert_eq!(removed, Vec::new());
        assert_eq!(known.set, input.into_iter().collect());
        assert_eq!(known.ring.len(), 2);
    }

    #[test]
    fn added_and_deleted_modify_membership() {
        let baseline: Vec<Librarian> = ["foo", "bar"]
            .iter()
            .map(<_ as ToLibrarian>::to_librarian)
            .collect();
        let request: Vec<Librarian> = ["foo", "baz"]
            .iter()
            .map(<_ as ToLibrarian>::to_librarian)
            .collect();
        let mut known = KnownLibrarians::new();

        let first_removed = known.modify_membership(&baseline);
        let removed = known.modify_membership(&request);

        let removed_predicate = vec![Librarian::new("bar")];
        assert_eq!(first_removed, Vec::new());
        assert_eq!(removed_predicate, removed);
        assert_eq!(known.set, request.into_iter().collect());
        assert_eq!(known.ring.len(), 2);
    }
}
