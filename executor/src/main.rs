use std::borrow::Cow;
use std::collections::HashSet;
use std::convert::identity;

use hash_rings::maglev::Ring;
use neuromancer::{
    base::*, errors::*, executor::administrative_server::*, executor::*, Checksummable,
    DefaultHasher,
};
use tonic::{
    transport::server::{Router, Unimplemented},
    Request, Response, Status,
};

struct Executor {
    librarians: KnownLibrarians,
}

struct KnownLibrarians {
    set: HashSet<String, DefaultHasher>,
    // the consistent hash ring used for looking up what librarians
    // job identifiers can be requested from and submitted
    lookup: Ring<'static, &'static str>,
}

pub struct Server {
    router: Router<AdministrativeServer<Executor>, Unimplemented>,
    addr: String,
}

impl KnownLibrarians {
    fn new() -> Self {
        Self {
            set: HashSet::default(),
            lookup: Ring::new(vec![]),
        }
    }

    /// Insert
    fn modify_membership(&mut self, librarians: &[&str]) {
        let new_membership_list: HashSet<String, DefaultHasher> =
            librarians.iter().map(|member| member.to_string()).collect();
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

#[tonic::async_trait]
impl Administrative for Executor {
    async fn librarian_membership_change(
        &self,
        request: Request<LibrarianMembershipChangeRequest>,
    ) -> Result<Response<()>, Status> {
        Ok(Response::new(()))
    }
}

fn main() -> Result<()> {
    Ok(())
}
