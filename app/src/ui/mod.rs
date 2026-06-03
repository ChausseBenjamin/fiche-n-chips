use std::fmt::{self};

const ESC_START: &str = "\x1b[";
const ESC_END: &str = "\x1b[m";

pub use ansi::*;
#[rustfmt::skip]
#[allow(dead_code)]
pub mod ansi {
  type AnsiCode = &'static str;

	pub const RESET: AnsiCode = "0";

	pub const BOLD:              AnsiCode = "1";
	pub const BOLD_DEL:          AnsiCode = "21";
	pub const FAINT:             AnsiCode = "2";
	pub const FAINT_DEL:         AnsiCode = "22";
	pub const STRIKETHROUGH:     AnsiCode = "9";
	pub const STRIKETHROUGH_DEL: AnsiCode = "29";
	pub const ITALIC:            AnsiCode = "3";
	pub const ITALIC_DEL:        AnsiCode = "23";
	pub const UNDERLINE:         AnsiCode = "4";
	pub const UNDERLINE_DEL:     AnsiCode = "24";
	pub const SLOW_BLINK:        AnsiCode = "5";
	pub const BLINK_DEL:         AnsiCode = "25";
	pub const RAPID_BLINK:       AnsiCode = "6";
	pub const REVERSE:           AnsiCode = "7";
	pub const REVERSE_DEL:       AnsiCode = "27";
	pub const CONCEAL:           AnsiCode = "8";
	pub const CONCEAL_DEL:       AnsiCode = "28";

	pub const BLACK_FG:    AnsiCode = "30";
	pub const BLACK_BG:    AnsiCode = "40";
	pub const RED_FG:      AnsiCode = "31";
	pub const RED_BG:      AnsiCode = "41";
	pub const GREEN_FG:    AnsiCode = "32";
	pub const GREEN_BG:    AnsiCode = "42";
	pub const YELLOW_FG:   AnsiCode = "33";
	pub const YELLOW_BG:   AnsiCode = "43";
	pub const BLUE_FG:     AnsiCode = "34";
	pub const BLUE_BG:     AnsiCode = "44";
	pub const MAGENTA_FG:  AnsiCode = "35";
	pub const MAGENTA_BG:  AnsiCode = "45";
	pub const CYAN_FG:     AnsiCode = "36";
	pub const CYAN_BG:     AnsiCode = "46";
	pub const WHITE_FG:    AnsiCode = "37";
	pub const WHITE_BG:    AnsiCode = "47";
	pub const EXTENDED_FG: AnsiCode = "38";
	pub const DEFAULT_FG:  AnsiCode = "39";
}

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
	#[cfg(test)]
	pub fn raw(&self) -> &str { &self.inner }

	#[cfg(test)]
	pub fn styled(&self) -> String {
		use std::fmt::Write;
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

pub trait PlainFormat {
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
        use $crate::ui::__prefer_plain::{ResolveFallback as _, Wrap};
        let __fmt = $fmt.to_string();
        let __args: Vec<String> = vec![$( Wrap($arg).resolve().to_string() ),*];
        let mut __parts = __fmt.split("{}");
        let mut __result = __parts.next().unwrap_or("").to_string();
        for (__arg, __part) in __args.iter().zip(__parts) {
            __result.push_str(__arg);
            __result.push_str(__part);
        }
        __result
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
		let s = ui::format!(true, "{}", us);
		print("ui::format!(true, \"{}\", ui_string)", &s);
		assert_eq!(s, "\x1b[31mhi\x1b[m");
	}

	#[test]
	fn test_styled_then_plain() {
		let name = ui::string("Bob", &[BOLD, YELLOW_FG]);
		let styled = ui::format!(true, "{}", name.clone());
		print("ui::format!(true, \"{}\", name)", &styled);
		assert_eq!(styled, "\x1b[1;33mBob\x1b[m");

		let plain = plain!("{}", name);
		print("plain!(\"{}\", name)", &plain);
		assert_eq!(plain, "Bob");
	}
}
