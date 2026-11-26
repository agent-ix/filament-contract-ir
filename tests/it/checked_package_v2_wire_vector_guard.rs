// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! A budgeted read pays for a vector before requesting it only when the member
//! decodes through an `exact_` entry: serde's own `Vec` visitor allocates
//! before the decoder sees the first element. This scan lists every derived
//! `Deserialize` member of the checked-package sources that holds a vector or
//! owned slice without one, so a new member cannot join them unnoticed.
//! Hand-written `Deserialize` implementations and type aliases are not seen.

use ix_trace_rs::trace;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};

/// Whether the type owns a vector or a slice anywhere in it.
#[derive(Default)]
struct HoldsVector(bool);

impl<'ast> Visit<'ast> for HoldsVector {
    fn visit_path_segment(&mut self, segment: &'ast syn::PathSegment) {
        self.0 |= segment.ident == "Vec";
        visit::visit_path_segment(self, segment);
    }
    fn visit_type_slice(&mut self, _: &'ast syn::TypeSlice) {
        self.0 = true;
    }
    // A borrowed slice allocates nothing.
    fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {}
}

fn derives_deserialize(attributes: &[syn::Attribute]) -> bool {
    let mut derives = false;
    for attribute in attributes.iter().filter(|a| a.path().is_ident("derive")) {
        attribute
            .parse_nested_meta(|meta| {
                derives |= meta
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "Deserialize");
                Ok(())
            })
            .expect("a derive list");
    }
    derives
}

/// Whether the field is skipped, or decodes through an `exact_` entry.
fn pays_first(attributes: &[syn::Attribute]) -> bool {
    let mut pays = false;
    for attribute in attributes.iter().filter(|a| a.path().is_ident("serde")) {
        attribute
            .parse_nested_meta(|meta| {
                pays |= meta.path.is_ident("skip") || meta.path.is_ident("skip_deserializing");
                if meta.input.peek(syn::Token![=]) {
                    let value: syn::Expr = meta.value()?.parse()?;
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(path),
                        ..
                    }) = value
                    {
                        pays |= meta.path.is_ident("deserialize_with")
                            && path
                                .value()
                                .rsplit("::")
                                .next()
                                .is_some_and(|entry| entry.starts_with("exact_"));
                    }
                }
                Ok(())
            })
            .expect("a serde attribute");
    }
    pays
}

fn test_only(attributes: &[syn::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && attribute
                .parse_args::<syn::Ident>()
                .is_ok_and(|condition| condition == "test")
    })
}

struct Scan {
    file: String,
    found: BTreeSet<String>,
}

impl Scan {
    fn fields(&mut self, owner: &str, fields: &syn::Fields) {
        for (position, field) in fields.iter().enumerate() {
            let mut holds = HoldsVector::default();
            holds.visit_type(&field.ty);
            if holds.0 && !pays_first(&field.attrs) {
                let name = field
                    .ident
                    .as_ref()
                    .map_or_else(|| position.to_string(), ToString::to_string);
                self.found.insert(format!("{} {owner}.{name}", self.file));
            }
        }
    }
}

impl<'ast> Visit<'ast> for Scan {
    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        if !test_only(&item.attrs) {
            visit::visit_item_mod(self, item);
        }
    }
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        if derives_deserialize(&item.attrs) && !test_only(&item.attrs) {
            self.fields(&item.ident.to_string(), &item.fields);
        }
    }
    fn visit_item_enum(&mut self, item: &'ast syn::ItemEnum) {
        if derives_deserialize(&item.attrs) && !test_only(&item.attrs) {
            for variant in &item.variants {
                let owner = format!("{}::{}", item.ident, variant.ident);
                self.fields(&owner, &variant.fields);
            }
        }
    }
}

fn rust_files(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).expect("a source directory") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            rust_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

fn unpaid_vector_members() -> BTreeSet<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("crates/filament-contract-model/src/checked_package");
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    let mut found = BTreeSet::new();
    for path in files {
        let text = std::fs::read_to_string(&path).expect("a source file");
        let mut scan = Scan {
            file: path
                .strip_prefix(&root)
                .expect("under the root")
                .to_string_lossy()
                .into_owned(),
            found: BTreeSet::new(),
        };
        scan.visit_file(&syn::parse_file(&text).expect("a Rust source file"));
        found.append(&mut scan.found);
    }
    found
}

/// The members that decode as a plain vector: caller evidence, contract-package
/// records, the operation catalog (read without a limit), the node-ID, term and
/// refinement types the budgeted read decodes by hand, and the tagged enums serde
/// buffers (`DomainKey`, `RequirementExtentWire`), whose buffering is itself
/// uncharged. A vector member the budgeted decoder reads through serde belongs on
/// an `exact_` entry, not in this list.
const PLAIN_VECTOR_MEMBERS: &[&str] = &[
    "evidence.rs CheckedExtensionGrammarSchema.productions",
    "evidence.rs CheckedExtensionNodeSchema.clauses",
    "evidence.rs CheckedExtensionProduction.clauses",
    "evidence.rs CheckedExtensionTypedNodeSchema.nodes",
    "shared.rs WireNodeId.path",
    "v2/contract_package.rs ContractNodeSourcesV1.occurrences",
    "v2/contract_package.rs ContractPackageWireV1.feature_accounting",
    "v2/contract_package.rs ContractPackageWireV1.source_keys",
    "v2/contract_package.rs ContractRefusalSourcesV1.occurrences",
    "v2/contract_package.rs ContractRequestSourcesV1.occurrences",
    "v2/contract_package.rs ContractSemanticGraphV1.nodes",
    "v2/contract_package.rs ContractSourceMapV1.nodes",
    "v2/contract_package.rs ContractSourceMapV1.refusals",
    "v2/contract_package.rs ContractSourceMapV1.requested_items",
    "v2/model_members.rs SourceOperationFrame.creates",
    "v2/model_members.rs SourceOperationFrame.deletes",
    "v2/model_members.rs SourceOperationFrame.deletes_parameters",
    "v2/model_members.rs SourceOperationFrame.modifies",
    "v2/model_members.rs SourceOperationFrame.modifies_receiver",
    "v2/operation_catalog.rs Wire.operands",
    "v2/operation_catalog_index.rs OperationCatalogWire.constraint_kinds",
    "v2/operation_catalog_index.rs OperationCatalogWire.families",
    "v2/operation_catalog_index.rs OperationCatalogWire.groups",
    "v2/operation_catalog_index.rs OperationCatalogWire.law_roles",
    "v2/operation_catalog_index.rs OperationCatalogWire.leaf_sources",
    "v2/operation_catalog_index.rs OperationCatalogWire.member_kinds",
    "v2/operation_catalog_index.rs OperationCatalogWire.modes",
    "v2/operation_catalog_index.rs OperationCatalogWire.operations",
    "v2/operation_catalog_index.rs OperationCatalogWire.profile_law_roles",
    "v2/operation_catalog_index.rs OperationCatalogWire.result_forms",
    "v2/operation_catalog_index.rs OperationCatalogWire.type_pinned_modes",
    "v2/refinement.rs RefinementAbstractClass::Declared.binder_types",
    "v2/refinement.rs RefinementClassCatalog.concrete",
    "v2/refinement.rs RefinementClassCatalog.r#abstract",
    "v2/refinement.rs RefinementConcreteClass::Declared.binder_types",
    "v2/refinement.rs RefinementHistory.updates",
    "v2/refinement.rs RefinementObject.fields",
    "v2/refinement.rs RefinementStepTarget::Operation.arguments",
    "v2/refinement.rs RefinementStepTarget::ProtocolStep.arguments",
    "v2/requirement_records.rs DomainKey::Node.path",
    "v2/requirement_records.rs RequirementExtentWire::Unbounded.domains",
    "v2/typed_terms.rs CheckedTerm::AbstractionRelation.frames",
    "v2/typed_terms.rs CheckedTerm::AbstractionRelation.objects",
    "v2/typed_terms.rs CheckedTerm::AbstractionRelation.populations",
    "v2/typed_terms.rs CheckedTerm::Aggregate.members",
    "v2/typed_terms.rs CheckedTerm::Application.arguments",
    "v2/typed_terms.rs CheckedTerm::ComponentComposition.components",
    "v2/typed_terms.rs CheckedTerm::ComponentComposition.environment",
    "v2/typed_terms.rs CheckedTerm::ComponentComposition.guarantees",
    "v2/typed_terms.rs CheckedTerm::ComponentComposition.liveness_dependencies",
    "v2/typed_terms.rs CheckedTerm::ComponentComposition.relies",
    "v2/typed_terms.rs CheckedTerm::ComponentComposition.supplies",
    "v2/typed_terms.rs CheckedTerm::ExtensionForm.clauses",
    "v2/typed_terms.rs CheckedTerm::Frame.creates",
    "v2/typed_terms.rs CheckedTerm::Frame.deletes",
    "v2/typed_terms.rs CheckedTerm::Frame.deletes_scoped",
    "v2/typed_terms.rs CheckedTerm::Frame.modifies",
    "v2/typed_terms.rs CheckedTerm::ImplementationGuarantee.environment",
    "v2/typed_terms.rs CheckedTerm::ImplementationGuarantee.initial",
    "v2/typed_terms.rs CheckedTerm::ImplementationGuarantee.progress",
    "v2/typed_terms.rs CheckedTerm::Refinement.abstract_internal",
    "v2/typed_terms.rs CheckedTerm::Refinement.assume",
    "v2/typed_terms.rs CheckedTerm::Refinement.ensure",
    "v2/typed_terms.rs CheckedTerm::Refinement.histories",
    "v2/typed_terms.rs CheckedTerm::Refinement.objects",
    "v2/typed_terms.rs CheckedTerm::Refinement.populations",
    "v2/typed_terms.rs CheckedTerm::Refinement.steps",
];

#[trace("TC-472", "FR-038-AC-499")]
#[test]
fn no_new_wire_member_decodes_as_a_plain_vector() {
    let expected: BTreeSet<String> = PLAIN_VECTOR_MEMBERS
        .iter()
        .map(|member| (*member).to_owned())
        .collect();
    let found = unpaid_vector_members();
    let added: Vec<_> = found.difference(&expected).collect();
    let gone: Vec<_> = expected.difference(&found).collect();
    assert!(
        added.is_empty() && gone.is_empty(),
        "decode a budgeted vector member through an `exact_` entry; added: {added:#?}, \
         no longer present: {gone:#?}"
    );
}
