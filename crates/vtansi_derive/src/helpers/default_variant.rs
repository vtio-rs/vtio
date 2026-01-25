//! Logic for finding and representing default variants in enums.
//!
//! This module provides utilities for handling the `#[vtansi(default)]`
//! attribute, which allows enum variants to serve as fallback values when
//! parsing fails.

use syn::{DataEnum, Fields, Ident};

use super::variant_props::HasVariantProperties;

/// Represent a variant marked with `#[vtansi(default)]`.
///
/// Default variants can either be unit variants (which return a constant
/// value when parsing fails) or single-field tuple variants (which capture
/// the unparsed value).
pub enum DefaultVariant {
    /// A unit variant that returns a constant value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     #[vtansi(default)]
    ///     Unknown,
    /// }
    /// ```
    Unit(Ident),
    /// A tuple variant with one field that captures the unrecognized value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// enum Color {
    ///     Red,
    ///     Green,
    ///     #[vtansi(default)]
    ///     Custom(String),
    /// }
    /// ```
    Capturing(Ident),
}

/// Find the variant marked with `#[vtansi(default)]`, if any.
///
/// This function scans all variants in the enum and identifies the one
/// marked with the `#[vtansi(default)]` attribute. It validates that the
/// default variant is either a unit variant or a single-field tuple
/// variant.
///
/// # Errors
///
/// Return an error if:
/// - Multiple variants are marked with `#[vtansi(default)]`
/// - The default variant is not a unit or single-field tuple variant
pub fn find_default_variant(
    data: &DataEnum,
) -> syn::Result<Option<DefaultVariant>> {
    let mut default_variant = None;
    let mut default_ident: Option<&Ident> = None;

    for variant in &data.variants {
        let props = variant.get_variant_properties()?;

        if props.default.is_none() {
            continue;
        }

        // Check for duplicate default variants
        if let Some(first_ident) = default_ident {
            return Err(syn::Error::new_spanned(
                variant,
                format!(
                    "Only one variant can be marked with #[vtansi(default)]. \
                     First default variant was '{first_ident}'"
                ),
            ));
        }
        default_ident = Some(&variant.ident);

        // Determine if it's a capturing variant
        let dv = match &variant.fields {
            Fields::Unit => DefaultVariant::Unit(variant.ident.clone()),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                DefaultVariant::Capturing(variant.ident.clone())
            }
            _ => {
                return Err(syn::Error::new_spanned(
                    variant,
                    "Default variant must be either a unit variant or a \
                     tuple variant with exactly one field",
                ));
            }
        };

        default_variant = Some(dv);
    }

    Ok(default_variant)
}
