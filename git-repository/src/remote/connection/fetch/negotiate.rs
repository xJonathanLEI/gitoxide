/// The way the negotiation is performed
#[derive(Copy, Clone)]
pub(crate) enum Algorithm {
    /// Our very own implementation that probably should be replaced by one of the known algorithms soon.
    Naive,
}

/// The error returned during negotiation.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("We were unable to figure out what objects the server should send after {rounds} round(s)")]
    NegotiationFailed { rounds: usize },
}

/// Negotiate one round with `algo` by looking at `ref_map` and adjust `arguments` to contain the haves and wants.
/// If this is not the first round, the `previous_response` is set with the last recorded server response.
/// Returns `true` if the negotiation is done from our side so the server won't keep asking.
pub(crate) fn one_round(
    algo: Algorithm,
    round: usize,
    repo: &crate::Repository,
    ref_map: &crate::remote::fetch::RefMap,
    arguments: &mut git_protocol::fetch::Arguments,
    _previous_response: Option<&git_protocol::fetch::Response>,
) -> Result<bool, Error> {
    match algo {
        Algorithm::Naive => {
            assert_eq!(round, 1, "Naive always finishes after the first round, and claims.");
            let mut has_missing_tracking_branch = false;
            for mapping in &ref_map.mappings {
                let have_id = mapping.local.as_ref().and_then(|name| {
                    repo.find_reference(name)
                        .ok()
                        .and_then(|r| r.target().try_id().map(ToOwned::to_owned))
                });
                match have_id {
                    Some(have_id) if mapping.remote.as_id() != have_id => {
                        arguments.want(mapping.remote.as_id());
                        arguments.have(have_id);
                    }
                    Some(_) => {}
                    None => {
                        arguments.want(mapping.remote.as_id());
                        has_missing_tracking_branch = true;
                    }
                }
            }

            if has_missing_tracking_branch {
                if let Ok(Some(r)) = repo.head_ref() {
                    if let Some(id) = r.target().try_id() {
                        arguments.have(id);
                    }
                }
            }
            Ok(true)
        }
    }
}
