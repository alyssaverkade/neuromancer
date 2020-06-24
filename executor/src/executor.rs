use std::collections::HashSet;

use crossbeam::sync::ShardedLock;

use neuromancer::DefaultHasher;

#[derive(Default)]
pub(crate) struct Executor {
    pub(crate) librarians: ShardedLock<KnownLibrarians>,
}

#[derive(Default)]
pub(crate) struct KnownLibrarians {
    set: HashSet<String, DefaultHasher>,
}

impl KnownLibrarians {
    pub(crate) fn modify_membership(&mut self, librarians: &[String]) {
        let new_membership_list: HashSet<String, _> = librarians.iter().cloned().collect();
        if new_membership_list.is_superset(&self.set) {
            // (new_membership_list.difference(&self.set), vec![])
        } else {
            for member in new_membership_list.symmetric_difference(&self.set) {
                if self.set.contains(member) {
                    // the member was removed
                } else { // the member was added
                }
            }
        }
    }
}
