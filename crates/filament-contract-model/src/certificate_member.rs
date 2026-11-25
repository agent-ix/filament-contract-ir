// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Certificate member names for the malformed certificate rejection locus (FR-070).

macro_rules! certificate_members {
    ($( $variant:ident => $spelling:literal, )+) => {
        /// A member of a probability or zone certificate named by a malformed-certificate rejection.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum CertificateMember {
            $( #[doc = concat!("The `", $spelling, "` member.")] $variant, )+
        }

        impl CertificateMember {
            /// Every member in the published schema sequence.
            pub const ALL: &'static [Self] = &[$( Self::$variant, )+];

            /// The exact member name in the backend-provider certificate schema.
            pub fn as_str(self) -> &'static str {
                match self {
                    $( Self::$variant => $spelling, )+
                }
            }
        }
    };
}

certificate_members! {
    Identity => "identity",
    NodeId => "node_id",
    Model => "model",
    Ordinal => "ordinal",
    Objective => "objective",
    Kind => "kind",
    Values => "values",
    Ranking => "ranking",
    Policy => "policy",
    Components => "components",
    Scale => "scale",
    Bounds => "bounds",
    Nodes => "nodes",
    Edges => "edges",
    Reasons => "reasons",
    Divergence => "divergence",
    NonZeno => "non_zeno",
    Initials => "initials",
    Admitted => "admitted",
    Refinement => "refinement",
    RefinementNode => "refinement_node",
    ConcretePackage => "concrete_package",
    AbstractPackage => "abstract_package",
    Positions => "positions",
    Taken => "taken",
    Locked => "locked",
    Start => "start",
    Nested => "nested",
    TaskProduct => "task_product",
    AbstractStates => "abstract_states",
    CandidateBlocks => "candidate_blocks",
    Automaton => "automaton",
    Bias => "bias",
    Bound => "bound",
    Choice => "choice",
    Clock => "clock",
    Concrete => "concrete",
    Conditional => "conditional",
    Constraint => "constraint",
    Denominator => "denominator",
    EfNodes => "ef_nodes",
    Form => "form",
    Gain => "gain",
    History => "history",
    Horizon => "horizon",
    Lower => "lower",
    Mode => "mode",
    Node => "node",
    Numerator => "numerator",
    Part => "part",
    Post => "post",
    Rank => "rank",
    Reset => "reset",
    Set => "set",
    State => "state",
    StateGraph => "state_graph",
    Strict => "strict",
    Target => "target",
    Transition => "transition",
    Transitions => "transitions",
    Type => "type",
    Upper => "upper",
    Value => "value",
    Zone => "zone",
}

#[cfg(test)]
mod tests {
    use super::CertificateMember;
    use ix_trace_rs::trace;

    /// FR-070-AC-4: members named by the conformance-checked schema appear once in `ALL`,
    /// each spelled distinctly and without a path, index or alias.
    #[trace("FR-070-AC-4")]
    #[test]
    fn members_appear_once_with_distinct_plain_spellings() {
        for member in CertificateMember::ALL {
            assert_eq!(
                CertificateMember::ALL
                    .iter()
                    .filter(|m| *m == member)
                    .count(),
                1
            );
            let name = member.as_str();
            assert_eq!(
                CertificateMember::ALL
                    .iter()
                    .filter(|m| m.as_str() == name)
                    .count(),
                1,
                "{name} is spelled once"
            );
            assert!(name.chars().all(|c| c.is_ascii_lowercase() || c == '_'));
        }
    }

    /// FR-418-AC-22: `candidate_blocks` is a certificate member of the `positions` entry, listed
    /// after `abstract_states`; a wrong length or presence refuses `malformed` at `positions`.
    #[trace("FR-418-AC-22")]
    #[test]
    fn candidate_blocks_is_a_member_after_abstract_states() {
        let names: Vec<&str> = CertificateMember::ALL.iter().map(|m| m.as_str()).collect();
        let at = names
            .iter()
            .position(|n| *n == "abstract_states")
            .expect("abstract_states");
        assert_eq!(
            CertificateMember::CandidateBlocks.as_str(),
            "candidate_blocks"
        );
        assert_eq!(names[at + 1], "candidate_blocks");
    }

    /// FR-070-AC-1: horizon and part use their exact schema spellings.
    #[trace("FR-070-AC-1")]
    #[test]
    fn horizon_and_part_have_schema_spellings() {
        for (member, spelling) in [
            (CertificateMember::Horizon, "horizon"),
            (CertificateMember::Part, "part"),
        ] {
            assert!(CertificateMember::ALL.contains(&member));
            assert_eq!(member.as_str(), spelling);
        }
    }
}
