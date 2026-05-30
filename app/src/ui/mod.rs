use std::fmt::{self, Write};

const ESC_START: &str = "\x1b[";
const ESC_END: &str = "\x1b[m";

#[rustfmt::skip]
#[allow(dead_code)]
pub mod ansi {
	pub const RESET: &str = "0";

	pub const BOLD:              &str = "1";
	pub const BOLD_DEL:          &str = "21";
	pub const FAINT:             &str = "2";
	pub const FAINT_DEL:         &str = "22";
	pub const STRIKETHROUGH:     &str = "9";
	pub const STRIKETHROUGH_DEL: &str = "29";
	pub const ITALIC:            &str = "3";
	pub const ITALIC_DEL:        &str = "23";
	pub const UNDERLINE:         &str = "4";
	pub const UNDERLINE_DEL:     &str = "24";
	pub const SLOW_BLINK:        &str = "5";
	pub const BLINK_DEL:         &str = "25";
	pub const RAPID_BLINK:       &str = "6";
	pub const REVERSE:           &str = "7";
	pub const REVERSE_DEL:       &str = "27";
	pub const CONCEAL:           &str = "8";
	pub const CONCEAL_DEL:       &str = "28";

	pub const BLACK_FG:    &str = "30";
	pub const BLACK_BG:    &str = "40";
	pub const RED_FG:      &str = "31";
	pub const RED_BG:      &str = "41";
	pub const GREEN_FG:    &str = "32";
	pub const GREEN_BG:    &str = "42";
	pub const YELLOW_FG:   &str = "33";
	pub const YELLOW_BG:   &str = "43";
	pub const BLUE_FG:     &str = "34";
	pub const BLUE_BG:     &str = "44";
	pub const MAGENTA_FG:  &str = "35";
	pub const MAGENTA_BG:  &str = "45";
	pub const CYAN_FG:     &str = "36";
	pub const CYAN_BG:     &str = "46";
	pub const WHITE_FG:    &str = "37";
	pub const WHITE_BG:    &str = "47";
	pub const EXTENDED_FG: &str = "38";
	pub const DEFAULT_FG:  &str = "39";
}

pub use ansi::*;

fn seq(attrs: &[&str]) -> String {
	if attrs.is_empty() {
		return ESC_END.to_string();
	}
	let mut out = String::from(ESC_START);
	out.push_str(attrs[0]);
	for a in &attrs[1..] {
		out.push(';');
		out.push_str(a);
	}
	out.push('m');
	out
}

pub fn string(inner: impl fmt::Display, styles: &[&'static str]) -> UiString {
	UiString {
		inner:  inner.to_string(),
		styles: styles.to_vec(),
	}
}

#[derive(Debug, Clone)]
pub struct UiString {
	inner:  String,
	styles: Vec<&'static str>,
}

impl UiString {
	pub fn raw(&self) -> &str { &self.inner }

	pub fn styled(&self) -> String {
		let mut out = String::new();
		if !self.styles.is_empty() {
			let _ = write!(out, "{}", seq(&self.styles));
		}
		out.push_str(&self.inner);
		out.push_str(ESC_END);
		out
	}
}

impl fmt::Display for UiString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if !self.styles.is_empty() {
			f.write_str(&seq(&self.styles))?;
		}
		f.write_str(&self.inner)?;
		f.write_str(ESC_END)
	}
}

trait PlainFormat {
	fn fmt_plain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

impl PlainFormat for UiString {
	fn fmt_plain(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.inner)
	}
}

#[doc(hidden)]
pub mod __prefer_plain {
	use std::fmt::{Display, Formatter, Result};

	use super::PlainFormat;

	pub struct Wrap<T>(pub T);

	impl<T: PlainFormat> Wrap<T> {
		pub fn resolve(self) -> PlainValue<T> {
			PlainValue {
				inner:      self.0,
				display_fn: |v, f| v.fmt_plain(f),
			}
		}
	}

	pub trait ResolveFallback {
		type Inner: Display;
		fn resolve(self) -> PlainValue<Self::Inner>;
	}

	impl<T: Display> ResolveFallback for Wrap<T> {
		type Inner = T;

		fn resolve(self) -> PlainValue<T> {
			PlainValue {
				inner:      self.0,
				display_fn: |v, f| Display::fmt(v, f),
			}
		}
	}

	pub struct PlainValue<T> {
		inner:      T,
		display_fn: fn(&T, &mut Formatter<'_>) -> Result,
	}

	impl<T> Display for PlainValue<T> {
		fn fmt(&self, f: &mut Formatter<'_>) -> Result {
			(self.display_fn)(&self.inner, f)
		}
	}
}

#[macro_export]
macro_rules! plain {
    ($fmt:expr $(, $arg:expr)* $(,)?) => {{
        #[allow(unused_imports)]
        use $crate::ui::__prefer_plain::ResolveFallback as _;
        format!($fmt $(, $crate::ui::__prefer_plain::Wrap($arg).resolve())*)
    }};
}

macro_rules! __format_for {
    ($styled:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {{
        if $styled {
            std::format!($fmt $(, $arg)*)
        } else {
            $crate::plain!($fmt $(, $arg)*)
        }
    }};
}
pub(crate) use __format_for as format;

#[cfg(test)]
mod tests {
	use super::{self as ui, *};

	fn print(label: &str, s: &str) {
		println!("{label}: \"{s}\"");
	}

	#[test]
	fn test_seq_empty() {
		let s = seq(&[]);
		print("seq([])", &s);
		assert_eq!(s, "\x1b[m");
	}

	#[test]
	fn test_seq_single() {
		let s = seq(&[BOLD]);
		print("seq([BOLD])", &s);
		assert_eq!(s, "\x1b[1m");
	}

	#[test]
	fn test_seq_multiple() {
		let s = seq(&[BOLD, RED_FG]);
		print("seq([BOLD, RED_FG])", &s);
		assert_eq!(s, "\x1b[1;31m");
	}

	#[test]
	fn test_ui_string_display() {
		let us = ui::string("hello", &[BOLD, RED_FG]);
		let s = us.to_string();
		print("UiString::to_string()", &s);
		assert_eq!(s, "\x1b[1;31mhello\x1b[m");
	}

	#[test]
	fn test_ui_string_no_styles() {
		let us = ui::string("hello", &[]);
		let s = us.to_string();
		print("UiString::to_string() (no styles)", &s);
		assert_eq!(s, "hello\x1b[m");
	}

	#[test]
	fn test_ui_string_raw() {
		let us = ui::string("hello", &[BOLD, RED_FG]);
		let s = us.raw();
		print("UiString::raw()", s);
		assert_eq!(s, "hello");
	}

	#[test]
	fn test_ui_string_styled() {
		let us = ui::string("hello", &[BOLD]);
		let s = us.styled();
		print("UiString::styled()", &s);
		assert_eq!(s, "\x1b[1mhello\x1b[m");
	}

	#[test]
	fn test_plain_macro_plain_text_args() {
		let result = plain!("hello {}", "world");
		print("plain!(\"hello {}\", \"world\")", &result);
		assert_eq!(result, "hello world");
	}

	#[test]
	fn test_plain_macro_with_ui_string() {
		let us = ui::string("world", &[BOLD]);
		let result = plain!("hello {}", us);
		print("plain!(\"hello {}\", ui_string)", &result);
		assert_eq!(result, "hello world");
	}

	#[test]
	fn test_plain_macro_mixed_args() {
		let action = ui::string("danced", &[BOLD, RED_FG]);
		let result = plain!("Katy {} all night", action);
		print("plain!(\"Katy {} all night\", styled_action)", &result);
		assert_eq!(result, "Katy danced all night");
	}

	#[test]
	fn test_plain_macro_multiple_args() {
		let name = "Bob".to_string();
		let action = ui::string("ran", &[BOLD]);
		let location = "park";
		let result = plain!("{} {} to the {}", name, action, location);
		print("plain!(\"{} {} to the {}\", ...)", &result);
		assert_eq!(result, "Bob ran to the park");
	}

	#[test]
	fn test_plain_macro_no_args() {
		let result = plain!("hello world");
		print("plain!(\"hello world\")", &result);
		assert_eq!(result, "hello world");
	}

	#[test]
	fn test_format_macro_styled() {
		let us = ui::string("hi", &[RED_FG]);
		let s = format!("{}", us);
		print("format!(\"{}\", ui_string)", &s);
		assert_eq!(s, "\x1b[31mhi\x1b[m");
	}

	#[test]
	fn test_styled_then_plain() {
		let name = ui::string("Bob", &[BOLD, YELLOW_FG]);
		let styled = format!("{}", name);
		print("format!(\"{}\", name)", &styled);
		assert_eq!(styled, "\x1b[1;33mBob\x1b[m");

		let plain = plain!("{}", name);
		print("plain!(\"{}\", name)", &plain);
		assert_eq!(plain, "Bob");
	}
}
