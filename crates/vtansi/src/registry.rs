use linkme::distributed_slice;
use std::collections::HashMap;
use std::fmt::{Debug, Write};
use std::sync::LazyLock;

use crate::byte_trie::{ByteTrie, ByteTrieBuilder, ByteTrieCursor};

pub use crate::byte_trie::Answer;

use crate::AnsiControlFunctionKind;
use crate::{AnsiEvent, ParseError};

pub type AnsiEmitFn<'c> = dyn for<'a, 'b> FnMut(&'b dyn AnsiEvent<'a>) + 'c;

#[derive(Default, Clone, Copy)]
pub struct AnsiEventData<'a> {
    params: Option<&'a [&'a [u8]]>,
    /// Static params consumed during trie matching.
    /// Used by fields with `#[vtansi(locate = "static_params")]`.
    static_params: Option<&'a [&'a [u8]]>,
    data: Option<&'a [u8]>,
    finalbyte: Option<&'a [u8]>,
}

impl<'a> AnsiEventData<'a> {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    #[inline]
    pub fn new_with_params(params: &'a [&'a [u8]]) -> Self {
        Self {
            params: Some(params),
            static_params: None,
            data: None,
            finalbyte: None,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_params(&self, params: &'a [&'a [u8]]) -> Self {
        Self {
            params: Some(params),
            static_params: self.static_params,
            data: self.data,
            finalbyte: self.finalbyte,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_static_params(&self, static_params: &'a [&'a [u8]]) -> Self {
        Self {
            params: self.params,
            static_params: Some(static_params),
            data: self.data,
            finalbyte: self.finalbyte,
        }
    }

    #[must_use]
    #[inline]
    pub fn new_with_data(data: &'a [u8]) -> Self {
        Self {
            params: None,
            static_params: None,
            data: Some(data),
            finalbyte: None,
        }
    }

    #[must_use]
    #[inline]
    pub fn with_data(&self, data: &'a [u8]) -> Self {
        Self {
            params: self.params,
            static_params: self.static_params,
            data: Some(data),
            finalbyte: self.finalbyte,
        }
    }

    #[must_use]
    #[inline]
    pub fn new_with_finalbyte(finalbyte: &'a [u8]) -> Self {
        Self {
            params: None,
            static_params: None,
            data: None,
            finalbyte: Some(finalbyte),
        }
    }

    #[must_use]
    #[inline]
    pub fn with_finalbyte(&self, finalbyte: &'a [u8]) -> Self {
        Self {
            params: self.params,
            static_params: self.static_params,
            data: self.data,
            finalbyte: Some(finalbyte),
        }
    }

    #[must_use]
    #[inline]
    pub fn iter_params(
        &self,
    ) -> Option<std::iter::Copied<std::slice::Iter<'a, &'a [u8]>>> {
        self.params.map(|p| p.iter().copied())
    }

    #[must_use]
    #[inline]
    pub fn iter_static_params(
        &self,
    ) -> Option<std::iter::Copied<std::slice::Iter<'a, &'a [u8]>>> {
        self.static_params.map(|p| p.iter().copied())
    }

    #[must_use]
    #[inline]
    pub fn get_data(&self) -> Option<&'a [u8]> {
        self.data
    }

    #[must_use]
    #[inline]
    pub fn get_finalbyte(&self) -> Option<&'a [u8]> {
        self.finalbyte
    }
}

pub type Handler = for<'a, 'b, 'c> fn(
    input: &'b AnsiEventData<'a>,
    emit: &mut AnsiEmitFn<'c>,
) -> Result<(), ParseError>;

#[derive(Copy, Clone, Debug)]
pub struct AnsiControlFunctionMatchEntry {
    /// Control function name (usually the type name).
    pub name: &'static str,
    /// A key used to match this control function (usually includes the
    /// introducer, the fixed prefix and the final byte, plus a param count marker).
    pub key: &'static [u8],
    /// The kind of this control function.
    pub kind: AnsiControlFunctionKind,
    /// The fixed bytes following the introducer, e.g. this would include
    /// the private byte and any static params in a multi-byte escape sequence.
    pub prefix: &'static [u8],
    /// Final byte of a multi-byte escape sequence if any.
    pub final_byte: Option<u8>,
    /// Handler function to call on match. The handler receives a slice of byte
    /// slices which represent non-static portion of the function. params, and
    /// a callback used to emit the parse result.
    pub handler: Handler,
}

type AnsiControlFunctionTrie = ByteTrie<Handler>;

#[distributed_slice]
pub static ANSI_CONTROL_OUTPUT_FUNCTION_REGISTRY:
    [AnsiControlFunctionMatchEntry] = [..];
pub static ANSI_CONTROL_OUTPUT_FUNCTION_TRIE: LazyLock<
    AnsiControlFunctionTrie,
> = LazyLock::new(|| {
    build_trie_from_registry(&ANSI_CONTROL_OUTPUT_FUNCTION_REGISTRY)
});

#[distributed_slice]
pub static ANSI_CONTROL_INPUT_FUNCTION_REGISTRY:
    [AnsiControlFunctionMatchEntry] = [..];
pub static ANSI_CONTROL_INPUT_FUNCTION_TRIE: LazyLock<AnsiControlFunctionTrie> =
    LazyLock::new(|| {
        build_trie_from_registry(&ANSI_CONTROL_INPUT_FUNCTION_REGISTRY)
    });

pub fn build_trie_from_static_items<I, V, KF, VF>(
    items: &'static [I],
    mut key_fn: KF,
    mut value_fn: VF,
) -> ByteTrie<V>
where
    KF: FnMut(&'static I) -> &'static [u8],
    VF: FnMut(&'static I) -> V,
{
    let mut builder: ByteTrieBuilder<V> = ByteTrieBuilder::new();

    for item in items {
        builder.insert(key_fn(item), value_fn(item));
    }

    builder.build()
}

/// Formats a byte sequence as a human-readable string for error messages.
fn format_key_for_display(key: &[u8]) -> String {
    let mut result = String::new();
    for &b in key {
        if b.is_ascii_graphic() || b == b' ' {
            result.push(b as char);
        } else {
            write!(result, "\\x{b:02x}").unwrap();
        }
    }
    result
}

/// Build a trie from registry entries, detecting and reporting duplicate keys.
///
/// This function panics if two entries have the same trie key, providing
/// detailed error messages about which entries conflict.
///
/// # Panics
///
/// Panics if two or more entries have the same trie key, providing detailed
/// error messages about which entries conflict.
#[must_use]
pub fn build_trie_from_registry(
    items: &'static [AnsiControlFunctionMatchEntry],
) -> ByteTrie<Handler> {
    let mut builder: ByteTrieBuilder<Handler> = ByteTrieBuilder::new();
    let mut seen_keys: HashMap<&[u8], &str> = HashMap::new();
    let mut conflicts: Vec<(&[u8], &str, &str)> = Vec::new();

    for item in items {
        let key = item.key;
        let name = item.name;

        if let Some(prev_name) = seen_keys.get(key) {
            conflicts.push((key, prev_name, name));
        } else {
            seen_keys.insert(key, name);
            builder.insert(key, item.handler);
        }
    }

    if !conflicts.is_empty() {
        let mut msg = String::from("Duplicate trie keys detected!\n\n");
        for (key, prev_name, name) in &conflicts {
            write!(
                msg,
                "Key: {key:?} ({})\n  First: {prev_name}\n  Conflict: {name}\n\n",
                format_key_for_display(key),
            )
            .unwrap();
        }
        msg.push_str(
            "Each pair of types uses the same trie key, which means one will shadow the other.\n\
             To fix this, ensure the types have different:\n\
             - Final bytes, OR\n\
             - Static parameter prefixes, OR\n\
             - Private marker bytes\n\
             \n\
             For types that legitimately share the same final byte but differ in \n\
             parameter count (e.g., CursorPositionReport vs FnKeySeq with 'R'),\n\
             the type with NO parameters should use a fallback mechanism rather\n\
             than being registered with the same key."
        );
        panic!("{}", msg);
    }

    builder.build()
}

#[derive(Clone, Copy, Debug)]
pub struct AnsiControlFunctionTrieCursor(ByteTrieCursor<'static, Handler>);

impl AnsiControlFunctionTrieCursor {
    #[must_use]
    #[inline]
    pub fn new(trie: &'static AnsiControlFunctionTrie) -> Self {
        Self(trie.cursor())
    }

    /// Feed one byte and update the position; return the incremental `Answer`.
    #[inline]
    pub fn advance(&mut self, byte: u8) -> Answer<'static, Handler> {
        self.0.advance(byte)
    }

    #[must_use]
    #[inline]
    pub fn deref(&self) -> Answer<'static, Handler> {
        self.0.deref()
    }

    /// Feed multiple bytes in sequence and update the position.
    ///
    /// This is more efficient than calling `advance` in a loop as it reduces
    /// the overhead of repeatedly resuming and storing the search position.
    ///
    /// Returns the final `Answer` after processing all bytes, or `None` if
    /// any byte query fails (in which case the cursor position is updated to
    /// where the failure occurred).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut cursor = ansi_control_input_function_trie_cursor();
    /// if let Some(answer) = cursor.advance_slice(b"\x1B[") {
    ///     // Successfully advanced through ESC [
    /// }
    /// ```
    #[inline]
    pub fn advance_slice(&mut self, bytes: &[u8]) -> Answer<'static, Handler> {
        self.0.advance_slice(bytes)
    }

    /// Advances through bytes finding the longest matching prefix.
    ///
    /// Walks through the trie as far as possible, tracking the last position
    /// where a match was found. Returns a tuple of `(Answer, bytes_consumed)`:
    /// - `(Answer::DeadEnd, 0)` if no progress could be made
    /// - `(Answer::Prefix, n)` if we advanced but found no match
    /// - `(Answer::Match(v), n)` or `(Answer::PrefixAndMatch(v), n)` for the
    ///   longest match found
    ///
    /// The cursor position is updated to the end of the longest match.
    #[inline]
    pub fn advance_longest_match(
        &mut self,
        bytes: &[u8],
    ) -> (Answer<'static, Handler>, usize) {
        self.0.advance_longest_match(bytes)
    }
}

#[must_use]
#[inline]
pub fn ansi_control_input_function_trie_cursor() -> AnsiControlFunctionTrieCursor
{
    AnsiControlFunctionTrieCursor::new(&ANSI_CONTROL_INPUT_FUNCTION_TRIE)
}

static ANSI_INPUT_C0_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_input_function_trie_cursor();
        cursor.advance(0);
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_input_c0_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_INPUT_C0_TRIE_CURSOR
}

static ANSI_INPUT_ESC_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_input_function_trie_cursor();
        cursor.advance_slice(b"\x1B");
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_input_esc_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_INPUT_ESC_TRIE_CURSOR
}

static ANSI_INPUT_SS3_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_input_function_trie_cursor();
        cursor.advance_slice(b"\x1BO");
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_input_ss3_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_INPUT_SS3_TRIE_CURSOR
}

static ANSI_INPUT_CSI_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_input_function_trie_cursor();
        cursor.advance_slice(b"\x1B[");
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_input_csi_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_INPUT_CSI_TRIE_CURSOR
}

static ANSI_INPUT_OSC_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_input_function_trie_cursor();
        cursor.advance_slice(b"\x1B]");
        // Advance past the \0 final byte placeholder (OSC has no final byte)
        cursor.advance(0);
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_input_osc_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_INPUT_OSC_TRIE_CURSOR
}

static ANSI_INPUT_DCS_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_input_function_trie_cursor();
        cursor.advance_slice(b"\x1BP");
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_input_dcs_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_INPUT_DCS_TRIE_CURSOR
}

#[must_use]
#[inline]
pub fn ansi_control_output_function_trie_cursor()
-> AnsiControlFunctionTrieCursor {
    AnsiControlFunctionTrieCursor::new(&ANSI_CONTROL_OUTPUT_FUNCTION_TRIE)
}

static ANSI_OUTPUT_C0_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_output_function_trie_cursor();
        cursor.advance(0);
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_output_c0_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_OUTPUT_C0_TRIE_CURSOR
}

static ANSI_OUTPUT_ESC_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_output_function_trie_cursor();
        cursor.advance_slice(b"\x1B");
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_output_esc_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_OUTPUT_ESC_TRIE_CURSOR
}

static ANSI_OUTPUT_CSI_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_output_function_trie_cursor();
        cursor.advance_slice(b"\x1B[");
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_output_csi_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_OUTPUT_CSI_TRIE_CURSOR
}

static ANSI_OUTPUT_OSC_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_output_function_trie_cursor();
        cursor.advance_slice(b"\x1B]");
        // Advance past the \0 final byte placeholder (OSC has no final byte)
        cursor.advance(0);
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_output_osc_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_OUTPUT_OSC_TRIE_CURSOR
}

static ANSI_OUTPUT_DCS_TRIE_CURSOR: LazyLock<AnsiControlFunctionTrieCursor> =
    LazyLock::new(|| {
        let mut cursor = ansi_control_output_function_trie_cursor();
        cursor.advance_slice(b"\x1BP");
        cursor
    });

#[must_use]
#[inline]
pub fn ansi_output_dcs_trie_cursor() -> AnsiControlFunctionTrieCursor {
    *ANSI_OUTPUT_DCS_TRIE_CURSOR
}
