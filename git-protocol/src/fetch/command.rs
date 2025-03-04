/// The kind of command to invoke on the server side.
#[derive(PartialEq, Eq, Debug, Hash, Ord, PartialOrd, Clone, Copy)]
pub enum Command {
    /// List references.
    LsRefs,
    /// Fetch a pack.
    Fetch,
}

/// A key value pair of values known at compile time.
pub type Feature = (&'static str, Option<&'static str>);

impl Command {
    /// Produce the name of the command as known by the server side.
    pub fn as_str(&self) -> &'static str {
        match self {
            Command::LsRefs => "ls-refs",
            Command::Fetch => "fetch",
        }
    }
}

#[cfg(any(test, feature = "async-client", feature = "blocking-client"))]
mod with_io {
    use bstr::{BString, ByteSlice};
    use git_transport::client::Capabilities;

    use crate::fetch::{agent, command::Feature, Command};

    impl Command {
        /// Only V2
        fn all_argument_prefixes(&self) -> &'static [&'static str] {
            match self {
                Command::LsRefs => &["symrefs", "peel", "ref-prefix "],
                Command::Fetch => &[
                    "want ", // hex oid
                    "have ", // hex oid
                    "done",
                    "thin-pack",
                    "no-progress",
                    "include-tag",
                    "ofs-delta",
                    // Shallow feature/capability
                    "shallow ", // hex oid
                    "deepen ",  // commit depth
                    "deepen-relative",
                    "deepen-since ", // time-stamp
                    "deepen-not ",   // rev
                    // filter feature/capability
                    "filter ", // filter-spec
                    // ref-in-want feature
                    "want-ref ", // ref path
                    // sideband-all feature
                    "sideband-all",
                    // packfile-uris feature
                    "packfile-uris ", // protocols
                    // wait-for-done feature
                    "wait-for-done",
                ],
            }
        }

        fn all_features(&self, version: git_transport::Protocol) -> &'static [&'static str] {
            match self {
                Command::LsRefs => &[],
                Command::Fetch => match version {
                    git_transport::Protocol::V1 => &[
                        "multi_ack",
                        "thin-pack",
                        "side-band",
                        "side-band-64k",
                        "ofs-delta",
                        "shallow",
                        "deepen-since",
                        "deepen-not",
                        "deepen-relative",
                        "no-progress",
                        "include-tag",
                        "multi_ack_detailed",
                        "allow-tip-sha1-in-want",
                        "allow-reachable-sha1-in-want",
                        "no-done",
                        "filter",
                    ],
                    git_transport::Protocol::V2 => &[
                        "shallow",
                        "filter",
                        "ref-in-want",
                        "sideband-all",
                        "packfile-uris",
                        "wait-for-done",
                    ],
                },
            }
        }

        /// Compute initial arguments based on the given `features`. They are typically provided by the `default_features(…)` method.
        /// Only useful for V2
        pub(crate) fn initial_arguments(&self, features: &[Feature]) -> Vec<BString> {
            match self {
                Command::Fetch => ["thin-pack", "include-tag", "ofs-delta"]
                    .iter()
                    .map(|s| s.as_bytes().as_bstr().to_owned())
                    .chain(
                        ["sideband-all", "packfile-uris"]
                            .iter()
                            .filter(|f| features.iter().any(|(sf, _)| sf == *f))
                            .map(|f| f.as_bytes().as_bstr().to_owned()),
                    )
                    .collect(),
                Command::LsRefs => vec![b"symrefs".as_bstr().to_owned(), b"peel".as_bstr().to_owned()],
            }
        }

        /// Turns on all modern features for V1 and all supported features for V2, returning them as a vector of features.
        /// Note that this is the basis for any fetch operation as these features fulfil basic requirements and reasonably up-to-date servers.
        pub fn default_features(
            &self,
            version: git_transport::Protocol,
            server_capabilities: &Capabilities,
        ) -> Vec<Feature> {
            match self {
                Command::Fetch => match version {
                    git_transport::Protocol::V1 => {
                        let has_multi_ack_detailed = server_capabilities.contains("multi_ack_detailed");
                        let has_sideband_64k = server_capabilities.contains("side-band-64k");
                        self.all_features(version)
                            .iter()
                            .copied()
                            .filter(|feature| match *feature {
                                "side-band" if has_sideband_64k => false,
                                "multi_ack" if has_multi_ack_detailed => false,
                                "include-tag" | "no-progress" => false,
                                feature => server_capabilities.contains(feature),
                            })
                            .map(|s| (s, None))
                            .chain(Some(agent()))
                            .collect()
                    }
                    git_transport::Protocol::V2 => {
                        let supported_features: Vec<_> = server_capabilities
                            .iter()
                            .find_map(|c| {
                                if c.name() == Command::Fetch.as_str().as_bytes().as_bstr() {
                                    c.values().map(|v| v.map(|f| f.to_owned()).collect())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        self.all_features(version)
                            .iter()
                            .copied()
                            .filter(|feature| supported_features.iter().any(|supported| supported == feature))
                            .map(|s| (s, None))
                            .chain(Some(agent()))
                            .collect()
                    }
                },
                Command::LsRefs => vec![agent()],
            }
        }
        /// Panics if the given arguments and features don't match what's statically known. It's considered a bug in the delegate.
        pub(crate) fn validate_argument_prefixes_or_panic(
            &self,
            version: git_transport::Protocol,
            server: &Capabilities,
            arguments: &[BString],
            features: &[Feature],
        ) {
            let allowed = self.all_argument_prefixes();
            for arg in arguments {
                if allowed.iter().any(|allowed| arg.starts_with(allowed.as_bytes())) {
                    continue;
                }
                panic!("{}: argument {} is not known or allowed", self.as_str(), arg);
            }
            match version {
                git_transport::Protocol::V1 => {
                    for (feature, _) in features {
                        if server
                            .iter()
                            .any(|c| feature.starts_with(c.name().to_str_lossy().as_ref()))
                        {
                            continue;
                        }
                        panic!("{}: capability {} is not supported", self.as_str(), feature);
                    }
                }
                git_transport::Protocol::V2 => {
                    let allowed = server
                        .iter()
                        .find_map(|c| {
                            if c.name() == self.as_str().as_bytes().as_bstr() {
                                c.values().map(|v| v.map(|f| f.to_string()).collect::<Vec<_>>())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    for (feature, _) in features {
                        if allowed.iter().any(|allowed| feature == allowed) {
                            continue;
                        }
                        match *feature {
                            "agent" => {}
                            _ => panic!("{}: V2 feature/capability {} is not supported", self.as_str(), feature),
                        }
                    }
                }
            }
        }
    }
}
